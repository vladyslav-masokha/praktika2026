use serde::Serialize;

use common::{
    dto::{CreateOrderRequest, CreateOrderResponse, OrderResponse},
    errors::AppError,
    events::{OrderConfirmedEvent, OrderCreatedEvent, PaymentFailedEvent, PaymentSuccessEvent},
    utils::now_millis,
};

use crate::{app_state::AppState, logistics, publisher, repository};

#[derive(Debug, Serialize)]
pub struct AdminOrderSummaryResponse {
    pub total_orders: i64,
}

pub async fn create_order(
    state: &AppState,
    user_id: i64,
    request: CreateOrderRequest,
) -> Result<CreateOrderResponse, AppError> {
    if request.amount <= 0.0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".into()));
    }

    let order = repository::insert_order(
        &state.db,
        user_id,
        request.amount,
        "PENDING",
        request.product_id,
        request.product_slug.as_deref(),
        request.product_name.as_deref(),
        request.product_image_url.as_deref(),
    )
    .await?;

    let event = OrderCreatedEvent::now(order.id, order.user_id, order.amount, order.status.clone());
    publisher::publish_order_created(&state.amqp_channel, &state.exchange, &event).await?;

    Ok(CreateOrderResponse {
        id: order.id,
        user_id: order.user_id,
        amount: order.amount,
        status: order.status,
        created_at: order.created_at,
        product_id: order.product_id,
        product_slug: order.product_slug,
        product_name: order.product_name,
        product_image_url: order.product_image_url,
    })
}

pub async fn list_orders(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<OrderResponse>, AppError> {
    let orders = repository::find_orders_by_user_id(&state.db, user_id).await?;

    Ok(orders
        .into_iter()
        .map(|order| OrderResponse {
            id: order.id,
            user_id: order.user_id,
            amount: order.amount,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
            product_id: order.product_id,
            product_slug: order.product_slug,
            product_name: order.product_name,
            product_image_url: order.product_image_url,
        })
        .collect())
}

pub async fn get_order_by_id(
    state: &AppState,
    user_id: i64,
    order_id: i64,
) -> Result<OrderResponse, AppError> {
    let order = repository::find_order_by_id(&state.db, order_id).await?;

    if order.user_id != user_id {
        return Err(AppError::Unauthorized);
    }

    Ok(OrderResponse {
        id: order.id,
        user_id: order.user_id,
        amount: order.amount,
        status: order.status,
        created_at: order.created_at,
        updated_at: order.updated_at,
        product_id: order.product_id,
        product_slug: order.product_slug,
        product_name: order.product_name,
        product_image_url: order.product_image_url,
    })
}

pub async fn get_admin_summary(
    state: &AppState,
) -> Result<AdminOrderSummaryResponse, AppError> {
    let total_orders = repository::count_all_orders(&state.db).await?;
    Ok(AdminOrderSummaryResponse { total_orders })
}

pub async fn handle_payment_success(
    state: &AppState,
    event: PaymentSuccessEvent,
) -> Result<(), AppError> {
    tracing::info!(
        "ApplicationService.handle_payment_success(order_id={}, user_id={}, provider={}, transaction_id={})",
        event.order_id,
        event.user_id,
        event.provider,
        event.transaction_id
    );

    let result = repository::update_order_status(&state.db, event.order_id, "CONFIRMED").await?;

    if !result.changed {
        tracing::info!(
            "Order {} already has status {}, skip duplicate order.confirmed publish",
            result.order.id,
            result.order.status
        );
        return Ok(());
    }


    let shipment = logistics::create_shipment(
        &state.http_client,
        &state.logistics_provider,
        state.logistics_api_base_url.as_deref(),
        state.logistics_api_key.as_deref(),
        result.order.id,
        result.order.user_id,
    )
    .await?;

    let confirmed_event = OrderConfirmedEvent {
        order_id: result.order.id,
        user_id: result.order.user_id,
        status: result.order.status.clone(),
        logistics_provider: Some(shipment.provider),
        tracking_number: Some(shipment.tracking_number),
        tracking_url: shipment.tracking_url,
        sent_at_ms: now_millis(),
    };

    publisher::publish_order_confirmed(&state.amqp_channel, &state.exchange, &confirmed_event)
        .await?;

    Ok(())
}

pub async fn handle_payment_failed(
    state: &AppState,
    event: PaymentFailedEvent,
) -> Result<(), AppError> {
    tracing::info!(
        "ApplicationService.handle_payment_failed(order_id={}, user_id={}, provider={}, reason={})",
        event.order_id,
        event.user_id,
        event.provider,
        event.reason
    );

    let result = repository::update_order_status(&state.db, event.order_id, "FAILED").await?;

    if result.changed {
        tracing::info!("Order {} marked as FAILED", result.order.id);
    }

    Ok(())
}
