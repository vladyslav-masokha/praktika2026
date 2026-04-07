use common::config::{get_csv_env, get_env, get_u16_env};

#[derive(Clone)]
pub struct GatewayConfig {
    pub port: u16,
    pub order_service_url: String,
    pub jwt_secret: String,
    pub cors_allowed_origins: Vec<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        Self {
            port: get_u16_env("API_GATEWAY_PORT", 8080),
            order_service_url: get_env("ORDER_SERVICE_URL"),
            jwt_secret: get_env("JWT_SECRET"),
            cors_allowed_origins: get_csv_env(
                "CORS_ALLOWED_ORIGINS",
                "http://localhost:5173,http://127.0.0.1:5173",
            ),
        }
    }
}