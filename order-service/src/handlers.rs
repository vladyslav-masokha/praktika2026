use axum::{
    extract::{Extension, Path, State},
    Json,
};

use common::{
    dto::{CreateOrderRequest, CreateOrderResponse, OrderResponse},
    errors::AppError,
    jwt::Claims,
};

use crate::{app_state::AppState, services};

pub async fn create_order_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<CreateOrderResponse>, AppError> {
    let response = services::create_order(&state, claims.sub, payload).await?;
    Ok(Json(response))
}

pub async fn list_orders_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<OrderResponse>>, AppError> {
    let response = services::list_orders(&state, claims.sub).await?;
    Ok(Json(response))
}

pub async fn get_order_by_id_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(order_id): Path<i64>,
) -> Result<Json<OrderResponse>, AppError> {
    let response = services::get_order_by_id(&state, claims.sub, order_id).await?;
    Ok(Json(response))
}

pub async fn admin_summary_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<services::AdminOrderSummaryResponse>, AppError> {
    let response = services::get_admin_summary(&state).await?;
    Ok(Json(response))
}

pub async fn health_handler() -> &'static str {
    "order-service-ok"
}