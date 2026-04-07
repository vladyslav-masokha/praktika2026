use common::errors::AppError;
use common::events::OrderCreatedEvent;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::PaymentServiceConfig;

#[derive(Debug)]
pub enum PaymentProviderChargeOutcome {
    Success {
        provider: String,
        transaction_id: String,
    },
    Failed {
        provider: String,
        reason: String,
    },
}

#[derive(Clone)]
pub struct PaymentProviderRuntime {
    provider: String,
    api_base_url: Option<String>,
    api_key: Option<String>,
    client: Client,
}

#[derive(Debug, Serialize)]
struct ExternalChargeRequest {
    order_id: i64,
    user_id: i64,
    amount: f64,
    currency: &'static str,
}

#[derive(Debug, Deserialize)]
struct ExternalChargeResponse {
    approved: bool,
    transaction_id: Option<String>,
    reason: Option<String>,
}

impl PaymentProviderRuntime {
    pub fn new(config: &PaymentServiceConfig) -> Self {
        Self {
            provider: config.payment_provider.clone(),
            api_base_url: config.payment_api_base_url.clone(),
            api_key: config.payment_api_key.clone(),
            client: Client::new(),
        }
    }

    pub async fn charge(
        &self,
        event: &OrderCreatedEvent,
    ) -> Result<PaymentProviderChargeOutcome, AppError> {
        match self.provider.as_str() {
            "stripe" => self.charge_via_http("stripe", event).await,
            "paypal" => self.charge_via_http("paypal", event).await,
            _ => self.charge_mock(event).await,
        }
    }

    async fn charge_mock(
        &self,
        event: &OrderCreatedEvent,
    ) -> Result<PaymentProviderChargeOutcome, AppError> {
        if event.amount <= 36_000.0 {
            Ok(PaymentProviderChargeOutcome::Success {
                provider: "mock".to_string(),
                transaction_id: format!("MOCK-TXN-{}", event.order_id),
            })
        } else {
            Ok(PaymentProviderChargeOutcome::Failed {
                provider: "mock".to_string(),
                reason: "Amount exceeds configured payment limit".to_string(),
            })
        }
    }

    async fn charge_via_http(
        &self,
        provider: &str,
        event: &OrderCreatedEvent,
    ) -> Result<PaymentProviderChargeOutcome, AppError> {
        let Some(base_url) = &self.api_base_url else {
            return Ok(if event.amount <= 36_000.0 {
                PaymentProviderChargeOutcome::Success {
                    provider: provider.to_string(),
                    transaction_id: format!("{}-TXN-{}", provider.to_ascii_uppercase(), event.order_id),
                }
            } else {
                PaymentProviderChargeOutcome::Failed {
                    provider: provider.to_string(),
                    reason: format!("{} adapter rejected amount", provider),
                }
            });
        };

        let request = ExternalChargeRequest {
            order_id: event.order_id,
            user_id: event.user_id,
            amount: event.amount,
            currency: "USD",
        };

        let url = format!("{}/charge", base_url.trim_end_matches('/'));
        let mut call = self.client.post(url).json(&request);

        if let Some(api_key) = &self.api_key {
            call = call.bearer_auth(api_key);
        }

        let response = call
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("{} adapter call failed: {}", provider, e)))?;

        if !response.status().is_success() {
            return Ok(PaymentProviderChargeOutcome::Failed {
                provider: provider.to_string(),
                reason: format!("{} adapter returned HTTP {}", provider, response.status()),
            });
        }

        let payload: ExternalChargeResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("{} adapter response parse failed: {}", provider, e)))?;


        if payload.approved {
            Ok(PaymentProviderChargeOutcome::Success {
                provider: provider.to_string(),
                transaction_id: payload
                    .transaction_id
                    .unwrap_or_else(|| format!("{}-TXN-{}", provider.to_ascii_uppercase(), event.order_id)),
            })
        } else {
            Ok(PaymentProviderChargeOutcome::Failed {
                provider: provider.to_string(),
                reason: payload.reason.unwrap_or_else(|| "Payment rejected by external provider".to_string()),
            })
        }
    }
}
