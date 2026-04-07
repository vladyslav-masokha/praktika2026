use chrono::Utc;
use serde::{Deserialize, Serialize};

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/events.rs"));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreatedEvent {
    pub order_id: i64,
    pub user_id: i64,
    pub amount: f64,
    pub status: String,
    pub sent_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSuccessEvent {
    pub order_id: i64,
    pub user_id: i64,
    pub provider: String,
    pub transaction_id: String,
    pub sent_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFailedEvent {
    pub order_id: i64,
    pub user_id: i64,
    pub provider: String,
    pub reason: String,
    pub sent_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfirmedEvent {
    pub order_id: i64,
    pub user_id: i64,
    pub status: String,
    pub logistics_provider: Option<String>,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub sent_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkEvent {
    pub event_id: String,
    pub sequence: i64,
    pub sent_at_ms: i64,
    pub payload: String,
}

impl OrderCreatedEvent {
    pub fn now(order_id: i64, user_id: i64, amount: f64, status: String) -> Self {
        Self {
            order_id,
            user_id,
            amount,
            status,
            sent_at_ms: Utc::now().timestamp_millis(),
        }
    }
}

impl BenchmarkEvent {
    pub fn new(event_id: String, sequence: i64, payload: String) -> Self {
        Self {
            event_id,
            sequence,
            payload,
            sent_at_ms: Utc::now().timestamp_millis(),
        }
    }
}
