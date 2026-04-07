mod app_state;
mod config;
mod consumer;
mod dto;
mod hub;
mod providers;
mod ws;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::get,
    Router,
};
use common::amqp::create_connection;
use config::NotificationServiceConfig;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app_state::AppState;
use hub::WsHub;
use providers::NotificationRuntime;

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

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
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = NotificationServiceConfig::from_env();

    let hub = WsHub::default();
    let runtime = NotificationRuntime::new(&config);

    let gateway_db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.gateway_database_url)
        .await
        .expect("Failed to connect gateway database");

    let amqp_conn = create_connection(&config.rabbitmq_url)
        .await
        .expect("Failed to connect to RabbitMQ");

    let state = AppState {
        hub: hub.clone(),
        gateway_db,
        runtime: runtime.clone(),
    };

    let consumer_state = state.clone();
    let consumer_exchange = config.exchange.clone();

    tokio::spawn(async move {
        if let Err(err) = consumer::start_consumer(&amqp_conn, consumer_state, consumer_exchange).await {
            tracing::error!("notification consumer crashed: {err}");
        }
    });

    let app = Router::new()
        .route("/health", get(|| async { "notification-service-ok" }))
        .route("/ws", get(ws::ws_handler))
        .with_state(state)
        .layer(build_cors_layer(&config.cors_allowed_origins))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    tracing::info!("notification-service running on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
