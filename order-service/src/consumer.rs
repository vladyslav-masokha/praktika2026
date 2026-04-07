use futures_util::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ExchangeKind,
};

use common::{
    errors::AppError,
    events::{PaymentFailedEvent, PaymentSuccessEvent},
};

use crate::{app_state::AppState, services};

pub async fn start_consumer(
    conn: &Connection,
    state: AppState,
) -> Result<(), AppError> {
    let channel: Channel = conn
        .create_channel()
        .await
        .map_err(|e| AppError::Broker(format!("Create order consumer channel failed: {e}")))?;

    channel
        .exchange_declare(
            &state.exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare exchange failed: {e}")))?;

    let queue = channel
        .queue_declare(
            "order-service.payment-events",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare queue failed: {e}")))?;

    channel
        .queue_bind(
            queue.name().as_str(),
            &state.exchange,
            "payment.success",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Bind payment.success queue failed: {e}")))?;

    channel
        .queue_bind(
            queue.name().as_str(),
            &state.exchange,
            "payment.failed",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Bind payment.failed queue failed: {e}")))?;

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "order-service-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Start consumer failed: {e}")))?;

    tracing::info!("order-service consumer started");

    while let Some(message) = consumer.next().await {
        let delivery = match message {
            Ok(delivery) => delivery,
            Err(err) => {
                tracing::error!("order-service consumer delivery error: {err}");
                continue;
            }
        };

        let routing_key = delivery.routing_key.to_string();

        let result = match routing_key.as_str() {
            "payment.success" => {
                let event: PaymentSuccessEvent = match serde_json::from_slice(&delivery.data) {
                    Ok(event) => event,
                    Err(err) => {
                        tracing::error!("Failed to parse PaymentSuccessEvent: {err}");
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                        continue;
                    }
                };

                tracing::info!(
                    "order-service consumer received payment.success for order_id={}",
                    event.order_id
                );

                services::handle_payment_success(&state, event).await
            }

            "payment.failed" => {
                let event: PaymentFailedEvent = match serde_json::from_slice(&delivery.data) {
                    Ok(event) => event,
                    Err(err) => {
                        tracing::error!("Failed to parse PaymentFailedEvent: {err}");
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                        continue;
                    }
                };

                tracing::info!(
                    "order-service consumer received payment.failed for order_id={}",
                    event.order_id
                );

                services::handle_payment_failed(&state, event).await
            }

            other => {
                tracing::warn!("Unexpected routing key: {other}");
                let _ = delivery.ack(BasicAckOptions::default()).await;
                continue;
            }
        };

        match result {
            Ok(()) => {
                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
            Err(err) => {
                tracing::error!(
                    "Application service failed for routing_key={}: {}. Requeueing.",
                    routing_key,
                    err
                );

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