use common::config::{get_csv_env, get_env, get_env_or, get_u16_env};

#[derive(Clone)]
pub struct NotificationServiceConfig {
    pub port: u16,
    pub rabbitmq_url: String,
    pub exchange: String,
    pub cors_allowed_origins: Vec<String>,
    pub gateway_database_url: String,
    pub email_provider: String,
    pub email_api_base_url: Option<String>,
    pub email_api_key: Option<String>,
    pub sms_provider: String,
    pub sms_api_base_url: Option<String>,
    pub sms_api_key: Option<String>,
    pub notification_from_email: String,
    pub notification_from_phone: Option<String>,
    pub notification_fallback_phone: Option<String>,
}

impl NotificationServiceConfig {
    pub fn from_env() -> Self {
        let email_api_base_url = get_env_or("EMAIL_API_BASE_URL", "");
        let email_api_key = get_env_or("EMAIL_API_KEY", "");
        let sms_api_base_url = get_env_or("SMS_API_BASE_URL", "");
        let sms_api_key = get_env_or("SMS_API_KEY", "");
        let notification_from_phone = get_env_or("NOTIFICATION_FROM_PHONE", "");
        let notification_fallback_phone = get_env_or("NOTIFICATION_FALLBACK_PHONE", "");


        Self {
            port: get_u16_env("NOTIFICATION_SERVICE_PORT", 8083),
            rabbitmq_url: get_env("RABBITMQ_URL"),
            exchange: get_env("AMQP_EXCHANGE"),
            cors_allowed_origins: get_csv_env(
                "CORS_ALLOWED_ORIGINS",
                "http://localhost:5173,http://127.0.0.1:5173",
            ),
            gateway_database_url: get_env("GATEWAY_DATABASE_URL"),
            email_provider: get_env_or("EMAIL_PROVIDER", "mock").to_ascii_lowercase(),
            email_api_base_url: (!email_api_base_url.is_empty()).then_some(email_api_base_url),
            email_api_key: (!email_api_key.is_empty()).then_some(email_api_key),
            sms_provider: get_env_or("SMS_PROVIDER", "disabled").to_ascii_lowercase(),
            sms_api_base_url: (!sms_api_base_url.is_empty()).then_some(sms_api_base_url),
            sms_api_key: (!sms_api_key.is_empty()).then_some(sms_api_key),
            notification_from_email: get_env_or("NOTIFICATION_FROM_EMAIL", "noreply@pulsecommerce.local"),
            notification_from_phone: (!notification_from_phone.is_empty()).then_some(notification_from_phone),
            notification_fallback_phone: (!notification_fallback_phone.is_empty())
                .then_some(notification_fallback_phone),
        }
    }
}
