use common::config::{get_env, get_env_or, get_u16_env};

#[derive(Clone)]
pub struct PaymentServiceConfig {
    pub port: u16,
    pub database_url: String,
    pub rabbitmq_url: String,
    pub exchange: String,
    pub payment_provider: String,
    pub payment_api_base_url: Option<String>,
    pub payment_api_key: Option<String>,
}

impl PaymentServiceConfig {
    pub fn from_env() -> Self {
        let payment_api_base_url = get_env_or("PAYMENT_API_BASE_URL", "");
        let payment_api_key = get_env_or("PAYMENT_API_KEY", "");

        Self {
            port: get_u16_env("PAYMENT_SERVICE_PORT", 8082),
            database_url: get_env("PAYMENT_DATABASE_URL"),
            rabbitmq_url: get_env("RABBITMQ_URL"),
            exchange: get_env("AMQP_EXCHANGE"),
            payment_provider: get_env_or("PAYMENT_PROVIDER", "mock").to_ascii_lowercase(),
            payment_api_base_url: (!payment_api_base_url.is_empty()).then_some(payment_api_base_url),
            payment_api_key: (!payment_api_key.is_empty()).then_some(payment_api_key),
        }
    }
}
