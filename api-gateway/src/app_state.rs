use reqwest::Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub order_service_url: String,
    pub jwt_secret: String,
    pub client: Client,
    pub db_pool: PgPool,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_url: String,
}