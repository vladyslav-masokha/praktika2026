use axum::{body::Bytes, http::StatusCode};
use common::errors::AppError;

use crate::app_state::AppState;

pub async fn forward_post_orders(
    state: &AppState,
    auth_header: &str,
    body: Bytes,
) -> Result<(StatusCode, String), AppError> {
    let url = format!("{}/orders", state.order_service_url);

    tracing::info!("Proxy POST -> {}", url);

    let response = state
        .client
        .post(&url)
        .header("authorization", auth_header)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("POST {} failed: {}", url, e)))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let text = response
        .text()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!("Proxy POST <- {} status={}", url, status);

    Ok((status, text))
}

pub async fn forward_get_orders(
    state: &AppState,
    auth_header: &str,
) -> Result<(StatusCode, String), AppError> {
    let url = format!("{}/orders", state.order_service_url);

    tracing::info!("Proxy GET -> {}", url);

    let response = state
        .client
        .get(&url)
        .header("authorization", auth_header)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GET {} failed: {}", url, e)))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let text = response
        .text()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!("Proxy GET <- {} status={} body={}", url, status, text);

    Ok((status, text))
}

pub async fn forward_get_order_by_id(
    state: &AppState,
    auth_header: &str,
    order_id: i64,
) -> Result<(StatusCode, String), AppError> {
    let url = format!("{}/orders/{}", state.order_service_url, order_id);

    tracing::info!("Proxy GET by ID -> {}", url);

    let response = state
        .client
        .get(&url)
        .header("authorization", auth_header)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GET {} failed: {}", url, e)))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let text = response
        .text()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!("Proxy GET by ID <- {} status={}", url, status);

    Ok((status, text))
}