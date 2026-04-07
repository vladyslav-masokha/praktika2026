use sqlx::PgPool;

use crate::{hub::WsHub, providers::NotificationRuntime};

#[derive(Clone)]
pub struct AppState {
    pub hub: WsHub,
    pub gateway_db: PgPool,
    pub runtime: NotificationRuntime,
}
