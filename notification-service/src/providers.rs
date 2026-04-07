use common::{
    errors::AppError,
    events::{OrderConfirmedEvent, PaymentFailedEvent},
};
use reqwest::Client;
use serde::Serialize;

use crate::config::NotificationServiceConfig;

#[derive(Clone)]
pub struct NotificationRuntime {
    client: Client,
    email_provider: String,
    email_api_base_url: Option<String>,
    email_api_key: Option<String>,
    sms_provider: String,
    sms_api_base_url: Option<String>,
    sms_api_key: Option<String>,
    from_email: String,
    from_phone: Option<String>,
    fallback_phone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserNotificationContext {
    pub user_id: i64,
    pub email: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmailRequest<'a> {
    to: &'a str,
    from: &'a str,
    subject: &'a str,
    body: &'a str,
}

#[derive(Debug, Serialize)]
struct SmsRequest<'a> {
    to: &'a str,
    from: &'a str,
    body: &'a str,
}

impl NotificationRuntime {
    pub fn new(config: &NotificationServiceConfig) -> Self {
        Self {
            client: Client::new(),
            email_provider: config.email_provider.clone(),
            email_api_base_url: config.email_api_base_url.clone(),
            email_api_key: config.email_api_key.clone(),
            sms_provider: config.sms_provider.clone(),
            sms_api_base_url: config.sms_api_base_url.clone(),
            sms_api_key: config.sms_api_key.clone(),
            from_email: config.notification_from_email.clone(),
            from_phone: config.notification_from_phone.clone(),
            fallback_phone: config.notification_fallback_phone.clone(),
        }
    }

    pub async fn send_order_confirmed(
        &self,
        user: &UserNotificationContext,
        event: &OrderConfirmedEvent,
    ) -> Result<(), AppError> {
        let subject = format!("Order #{} confirmed", event.order_id);
        let body = if let Some(tracking_number) = &event.tracking_number {
            format!(
                "Ваше замовлення #{} підтверджено. Трекінг: {}. Посилання: {}",
                event.order_id,
                tracking_number,
                event.tracking_url.clone().unwrap_or_else(|| "n/a".to_string())
            )
        } else {
            format!("Ваше замовлення #{} підтверджено.", event.order_id)
        };

        self.send_email(&user.email, &subject, &body).await?;
        self.send_sms_if_enabled(&body).await?;
        Ok(())
    }


    pub async fn send_payment_failed(
        &self,
        user: &UserNotificationContext,
        event: &PaymentFailedEvent,
    ) -> Result<(), AppError> {
        let subject = format!("Order #{} payment failed", event.order_id);
        let body = format!(
            "Оплата замовлення #{} неуспішна через {}: {}",
            event.order_id, event.provider, event.reason
        );

        self.send_email(&user.email, &subject, &body).await?;
        self.send_sms_if_enabled(&body).await?;
        Ok(())
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
        match self.email_provider.as_str() {
            "sendgrid" => {
                let Some(base_url) = &self.email_api_base_url else {
                    tracing::warn!("sendgrid adapter selected, but EMAIL_API_BASE_URL is missing; fallback to log");
                    tracing::info!("EMAIL -> {} | {} | {}", to, subject, body);
                    return Ok(());
                };

                let url = format!("{}/send-email", base_url.trim_end_matches('/'));
                let mut call = self.client.post(url).json(&EmailRequest {
                    to,
                    from: &self.from_email,
                    subject,
                    body,
                });

                if let Some(key) = &self.email_api_key {
                    call = call.bearer_auth(key);
                }

                let response = call
                    .send()
                    .await
                    .map_err(|e| AppError::Internal(format!("sendgrid adapter call failed: {}", e)))?;

                if !response.status().is_success() {
                    return Err(AppError::Internal(format!(
                        "sendgrid adapter returned HTTP {}",
                        response.status()
                    )));
                }

                Ok(())
            }
            _ => {
                tracing::info!("EMAIL(MOCK) -> {} | {} | {}", to, subject, body);
                Ok(())
            }
        }
    }

    async fn send_sms_if_enabled(&self, body: &str) -> Result<(), AppError> {
        match self.sms_provider.as_str() {
            "disabled" => Ok(()),
            "twilio" => {
                let Some(to) = self.fallback_phone.as_deref() else {
                    tracing::warn!("twilio adapter selected, but NOTIFICATION_FALLBACK_PHONE is missing");
                    return Ok(());
                };

                let Some(from) = self.from_phone.as_deref() else {
                    tracing::warn!("twilio adapter selected, but NOTIFICATION_FROM_PHONE is missing");
                    return Ok(());
                };

                let Some(base_url) = &self.sms_api_base_url else {
                    tracing::warn!("twilio adapter selected, but SMS_API_BASE_URL is missing; fallback to log");
                    tracing::info!("SMS -> {} | {}", to, body);
                    return Ok(());
                };

                let url = format!("{}/send-sms", base_url.trim_end_matches('/'));
                let mut call = self.client.post(url).json(&SmsRequest { to, from, body });

                if let Some(key) = &self.sms_api_key {
                    call = call.bearer_auth(key);
                }

                let response = call
                    .send()
                    .await
                    .map_err(|e| AppError::Internal(format!("twilio adapter call failed: {}", e)))?;

                if !response.status().is_success() {
                    return Err(AppError::Internal(format!(
                        "twilio adapter returned HTTP {}",
                        response.status()
                    )));
                }

                Ok(())
            }
            _ => {
                tracing::info!("SMS(MOCK) -> {}", body);
                Ok(())
            }
        }
    }
}
