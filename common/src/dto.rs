use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub amount: f64,
    pub product_id: Option<i64>,
    pub product_slug: Option<String>,
    pub product_name: Option<String>,
    pub product_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub id: i64,
    pub user_id: i64,
    pub amount: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub product_id: Option<i64>,
    pub product_slug: Option<String>,
    pub product_name: Option<String>,
    pub product_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: i64,
    pub user_id: i64,
    pub amount: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub product_id: Option<i64>,
    pub product_slug: Option<String>,
    pub product_name: Option<String>,
    pub product_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
}
