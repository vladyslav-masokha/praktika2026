use common::{
    errors::AppError,
    events::{OrderCreatedEvent, PaymentFailedEvent, PaymentSuccessEvent},
    utils::now_millis,
};

use crate::providers::{PaymentProviderChargeOutcome, PaymentProviderRuntime};

pub enum PaymentProcessingResult {
    Success(PaymentSuccessEvent),
    Failed(PaymentFailedEvent),
}

pub async fn process_payment(
    runtime: &PaymentProviderRuntime,
    event: &OrderCreatedEvent,
) -> Result<PaymentProcessingResult, AppError> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let outcome = runtime.charge(event).await?;

    Ok(match outcome {
        PaymentProviderChargeOutcome::Success {
            provider,
            transaction_id,
        } => PaymentProcessingResult::Success(PaymentSuccessEvent {
            order_id: event.order_id,
            user_id: event.user_id,
            provider,
            transaction_id,
            sent_at_ms: now_millis(),
        }),
        PaymentProviderChargeOutcome::Failed { provider, reason } => {
            PaymentProcessingResult::Failed(PaymentFailedEvent {
                order_id: event.order_id,
                user_id: event.user_id,
                provider,
                reason,
                sent_at_ms: now_millis(),
            })
        }
    })
}
