use futures_util::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ExchangeKind,
};
use sqlx::PgPool;

use common::{errors::AppError, events::OrderCreatedEvent};

use crate::{
    config::PaymentServiceConfig,
    providers::PaymentProviderRuntime,
    publisher,
    repository,
    service::{self, PaymentProcessingResult},
};

pub async fn start_consumer(
    conn: &Connection,
    db: PgPool,
    config: PaymentServiceConfig,
) -> Result<(), AppError> {
    let channel: Channel = conn
        .create_channel()
        .await
        .map_err(|e| AppError::Broker(format!("Create payment consumer channel failed: {e}")))?;

    channel
        .exchange_declare(
            &config.exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare exchange failed: {e}")))?;

    let queue = channel
        .queue_declare(
            "payment-service.order-created",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare queue failed: {e}")))?;

    channel
        .queue_bind(
            queue.name().as_str(),
            &config.exchange,
            "order.created",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Bind queue failed: {e}")))?;

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "payment-service-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Start consumer failed: {e}")))?;


    let provider_runtime = PaymentProviderRuntime::new(&config);

    tracing::info!(
        "payment-service consumer started with provider={} ",
        config.payment_provider
    );

    while let Some(message) = consumer.next().await {
        let delivery = match message {
            Ok(delivery) => delivery,
            Err(err) => {
                tracing::error!("payment-service consumer delivery error: {err}");
                continue;
            }
        };

        let event: OrderCreatedEvent = match serde_json::from_slice(&delivery.data) {
            Ok(event) => event,
            Err(err) => {
                tracing::error!("Failed to parse OrderCreatedEvent: {err}");
                let _ = delivery.ack(BasicAckOptions::default()).await;
                continue;
            }
        };

        match service::process_payment(&provider_runtime, &event).await {
            Ok(PaymentProcessingResult::Success(success_event)) => {
                if let Err(err) = repository::insert_payment(
                    &db,
                    success_event.order_id,
                    "SUCCESS",
                    Some(&success_event.transaction_id),
                )
                .await
                {
                    tracing::error!("Insert payment success record failed, requeueing: {err}");
                    let _ = delivery
                        .nack(BasicNackOptions {
                            multiple: false,
                            requeue: true,
                        })
                        .await;
                    continue;
                }

                if let Err(err) =
                    publisher::publish_payment_success(&channel, &config.exchange, &success_event).await
                {
                    tracing::error!("Publish payment.success failed, requeueing: {err}");
                    let _ = delivery
                        .nack(BasicNackOptions {
                            multiple: false,
                            requeue: true,
                        })
                        .await;
                    continue;
                }

                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
            Ok(PaymentProcessingResult::Failed(failed_event)) => {
                if let Err(err) =
                    repository::insert_payment(&db, failed_event.order_id, "FAILED", None).await
                {
                    tracing::error!("Insert payment failed record failed, requeueing: {err}");
                    let _ = delivery
                        .nack(BasicNackOptions {
                            multiple: false,
                            requeue: true,
                        })
                        .await;
                    continue;
                }

                if let Err(err) =
                    publisher::publish_payment_failed(&channel, &config.exchange, &failed_event).await
                {
                    tracing::error!("Publish payment.failed failed, requeueing: {err}");
                    let _ = delivery
                        .nack(BasicNackOptions {
                            multiple: false,
                            requeue: true,
                        })
                        .await;
                    continue;
                }

                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
            Err(err) => {
                tracing::error!("External payment adapter failed, requeueing: {err}");
                let _ = delivery
                    .nack(BasicNackOptions {
                        multiple: false,
                        requeue: true,
                    })
                    .await;
            }
        }
    }

    Ok(())
}
