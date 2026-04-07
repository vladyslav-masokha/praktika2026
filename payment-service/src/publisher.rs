use common::{
    amqp::publish_json,
    errors::AppError,
    events::{PaymentFailedEvent, PaymentSuccessEvent},
};
use lapin::Channel;

pub async fn publish_payment_success(
    channel: &Channel,
    exchange: &str,
    event: &PaymentSuccessEvent,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(event)
        .map_err(|e| AppError::Internal(format!("Serialize payment.success failed: {e}")))?;

    publish_json(channel, exchange, "payment.success", &payload).await
}

pub async fn publish_payment_failed(
    channel: &Channel,
    exchange: &str,
    event: &PaymentFailedEvent,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(event)
        .map_err(|e| AppError::Internal(format!("Serialize payment.failed failed: {e}")))?;

    publish_json(channel, exchange, "payment.failed", &payload).await
}