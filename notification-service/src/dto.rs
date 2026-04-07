use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsOrderStatusMessage {
    pub event_type: String,
    pub order_id: i64,
    pub status: String,
    pub message: String,
}