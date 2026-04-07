mod app_state;
mod config;
mod handlers;
mod middleware;
mod proxy;
mod routes;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderName, HeaderValue, Method,
    },
    serve,
};
use config::GatewayConfig;
use dotenvy::dotenv;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app_state::AppState;

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-user-id"),
        ]);

    if allowed_origins.iter().any(|origin| origin == "*") {
        base.allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> = allowed_origins
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect();
        base.allow_origin(origins)
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = GatewayConfig::from_env();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run api-gateway migrations");

    let google_client_id =
        std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let google_client_secret =
        std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
    let google_redirect_url =
        std::env::var("GOOGLE_REDIRECT_URL").unwrap_or_default();

    let state = AppState {
        order_service_url: config.order_service_url.clone(),
        jwt_secret: config.jwt_secret.clone(),
        client: Client::new(),
        db_pool,
        google_client_id,
        google_client_secret,
        google_redirect_url,
    };

    let app = routes::create_router(state)
        .layer(build_cors_layer(&config.cors_allowed_origins))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind API Gateway");

    tracing::info!("API Gateway running on {}", addr);

    serve(listener, app).await.expect("Server error");
}