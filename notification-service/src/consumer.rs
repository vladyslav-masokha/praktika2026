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
    events::{OrderConfirmedEvent, PaymentFailedEvent},
};

use crate::{
    app_state::AppState,
    dto::WsOrderStatusMessage,
    providers::UserNotificationContext,
};

#[derive(sqlx::FromRow)]
struct UserRow {
    email: String,
    full_name: Option<String>,
}

async fn load_user_context(
    state: &AppState,
    user_id: i64,
) -> Result<UserNotificationContext, AppError> {
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT email, full_name
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.gateway_db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    Ok(UserNotificationContext {
        user_id,
        email: user.email,
        full_name: user.full_name,
    })
}

pub async fn start_consumer(
    conn: &Connection,
    state: AppState,
    exchange: String,
) -> Result<(), AppError> {
    let channel: Channel = conn
        .create_channel()
        .await
        .map_err(|e| AppError::Broker(format!("Create notification consumer channel failed: {e}")))?;

    channel
        .exchange_declare(
            &exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare exchange failed: {e}")))?;

    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare queue failed: {e}")))?;

    channel
        .queue_bind(
            queue.name().as_str(),
            &exchange,
            "order.confirmed",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Bind order.confirmed failed: {e}")))?;

    channel
        .queue_bind(
            queue.name().as_str(),
            &exchange,
            "payment.failed",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Bind payment.failed failed: {e}")))?;

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "notification-service-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Start consumer failed: {e}")))?;

    while let Some(message) = consumer.next().await {
        let delivery = match message {
            Ok(delivery) => delivery,
            Err(err) => {
                tracing::error!("notification delivery error: {err}");
                continue;
            }
        };

        let routing_key = delivery.routing_key.to_string();

        let result: Result<(), AppError> = match routing_key.as_str() {
            "order.confirmed" => {
                let event: OrderConfirmedEvent = match serde_json::from_slice(&delivery.data) {
                    Ok(event) => event,
                    Err(err) => {
                        tracing::error!("Failed to parse OrderConfirmedEvent: {err}");
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                        continue;
                    }
                };


                let user = match load_user_context(&state, event.user_id).await {
                    Ok(user) => user,
                    Err(err) => {
                        tracing::error!("User lookup failed: {err}");
                        let _ = delivery
                            .nack(BasicNackOptions {
                                multiple: false,
                                requeue: true,
                            })
                            .await;
                        continue;
                    }
                };

                let ws_message = WsOrderStatusMessage {
                    event_type: "order.confirmed".to_string(),
                    order_id: event.order_id,
                    status: event.status.clone(),
                    message: match &event.tracking_number {
                        Some(ttn) => format!("Замовлення #{} підтверджено. Трекінг: {}", event.order_id, ttn),
                        None => format!("Замовлення #{} підтверджено", event.order_id),
                    },
                };

                state
                    .runtime
                    .send_order_confirmed(&user, &event)
                    .await?;

                let payload = serde_json::to_string(&ws_message)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                state.hub.notify_order(event.order_id, payload).await;
                Ok(())
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

                let user = match load_user_context(&state, event.user_id).await {
                    Ok(user) => user,
                    Err(err) => {
                        tracing::error!("User lookup failed: {err}");
                        let _ = delivery
                            .nack(BasicNackOptions {
                                multiple: false,
                                requeue: true,
                            })
                            .await;
                        continue;
                    }
                };

                let ws_message = WsOrderStatusMessage {
                    event_type: "payment.failed".to_string(),
                    order_id: event.order_id,
                    status: "FAILED".to_string(),
                    message: format!(
                        "Оплата замовлення #{} неуспішна через {}: {}",
                        event.order_id, event.provider, event.reason
                    ),
                };

                state.runtime.send_payment_failed(&user, &event).await?;

                let payload = serde_json::to_string(&ws_message)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                state.hub.notify_order(event.order_id, payload).await;
                Ok(())
            }
            _ => {
                let _ = delivery.ack(BasicAckOptions::default()).await;
                continue;
            }
        };

        match result {
            Ok(()) => {
                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
            Err(err) => {
                tracing::error!("notification processing failed: {err}");
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
