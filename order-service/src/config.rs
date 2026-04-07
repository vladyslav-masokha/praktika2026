use common::config::{get_csv_env, get_env, get_env_or, get_u16_env};

#[derive(Clone)]
pub struct OrderServiceConfig {
    pub port: u16,
    pub database_url: String,
    pub rabbitmq_url: String,
    pub exchange: String,
    pub jwt_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub logistics_provider: String,
    pub logistics_api_base_url: Option<String>,
    pub logistics_api_key: Option<String>,
}

impl OrderServiceConfig {
    pub fn from_env() -> Self {
        let logistics_api_base_url = get_env_or("LOGISTICS_API_BASE_URL", "");
        let logistics_api_key = get_env_or("LOGISTICS_API_KEY", "");

        Self {
            port: get_u16_env("ORDER_SERVICE_PORT", 8081),
            database_url: get_env("ORDER_DATABASE_URL"),
            rabbitmq_url: get_env("RABBITMQ_URL"),
            exchange: get_env("AMQP_EXCHANGE"),
            jwt_secret: get_env("JWT_SECRET"),
            cors_allowed_origins: get_csv_env(
                "CORS_ALLOWED_ORIGINS",
                "http://localhost:5173,http://127.0.0.1:5173",
            ),
            logistics_provider: get_env_or("LOGISTICS_PROVIDER", "mock").to_ascii_lowercase(),
            logistics_api_base_url: (!logistics_api_base_url.is_empty()).then_some(logistics_api_base_url),
            logistics_api_key: (!logistics_api_key.is_empty()).then_some(logistics_api_key),
        }
    }
}
