use common::{
    amqp::publish_json,
    errors::AppError,
    events::{OrderConfirmedEvent, OrderCreatedEvent},
};
use lapin::Channel;

pub async fn publish_order_created(
    channel: &Channel,
    exchange: &str,
    event: &OrderCreatedEvent,
) -> Result<(), AppError> {
    let payload =
        serde_json::to_vec(event).map_err(|e| AppError::Internal(e.to_string()))?;

    publish_json(channel, exchange, "order.created", &payload).await
}

pub async fn publish_order_confirmed(
    channel: &Channel,
    exchange: &str,
    event: &OrderConfirmedEvent,
) -> Result<(), AppError> {
    let payload =
        serde_json::to_vec(event).map_err(|e| AppError::Internal(e.to_string()))?;

    publish_json(channel, exchange, "order.confirmed", &payload).await
}