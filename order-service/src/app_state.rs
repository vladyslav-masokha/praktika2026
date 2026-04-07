use lapin::Channel;
use reqwest::Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub amqp_channel: Channel,
    pub exchange: String,
    pub jwt_secret: String,
    pub http_client: Client,
    pub logistics_provider: String,
    pub logistics_api_base_url: Option<String>,
    pub logistics_api_key: Option<String>,
}
