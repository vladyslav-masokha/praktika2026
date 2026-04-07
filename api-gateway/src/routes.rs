use axum::{
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};

use crate::{app_state::AppState, handlers, middleware::auth_middleware};

pub fn create_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route(
            "/api/auth/me",
            get(handlers::me_handler).put(handlers::update_me_handler),
        )
        .route(
            "/api/orders",
            get(handlers::list_orders_handler).post(handlers::create_order_handler),
        )
        .route("/api/orders/:id", get(handlers::get_order_by_id_handler))
        .route("/api/admin/stats", get(handlers::admin_stats_handler))
        .route("/api/admin/users", get(handlers::list_users_handler))
        .route(
            "/api/admin/products",
            get(handlers::admin_list_products_handler).post(handlers::create_product_handler),
        )
        .route(
            "/api/admin/products/:id",
            put(handlers::update_product_handler).delete(handlers::delete_product_handler),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let public_routes = Router::new()
        .route("/health", get(handlers::health_handler))
        .route("/api/auth/login", post(handlers::login_handler))
        .route("/api/auth/register", post(handlers::register_handler))
        .route("/api/auth/google", get(handlers::google_login_handler))
        .route("/api/auth/google/callback", get(handlers::google_callback_handler))
        .route("/api/products", get(handlers::list_products_handler))
        .route("/api/products/:slug", get(handlers::get_product_by_slug_handler))
        .route("/api/analytics/visit", post(handlers::record_visit_handler));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}