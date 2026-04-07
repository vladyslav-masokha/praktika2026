use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Payment {
    pub id: i64,
    pub order_id: i64,
    pub status: String,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
}