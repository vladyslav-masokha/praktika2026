use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};

use crate::{app_state::AppState, handlers, middleware::auth_middleware};

pub fn create_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/orders/admin/summary", get(handlers::admin_summary_handler))
        .route(
            "/orders",
            post(handlers::create_order_handler).get(handlers::list_orders_handler),
        )
        .route("/orders/:id", get(handlers::get_order_by_id_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .route("/health", get(handlers::health_handler))
        .merge(protected_routes)
        .with_state(state)
}