mod config;
mod consumer;
mod db;
mod models;
mod providers;
mod publisher;
mod repository;
mod service;

use axum::{routing::get, Router};
use common::amqp::create_connection;
use config::PaymentServiceConfig;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = PaymentServiceConfig::from_env();

    let db = db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect payment database");

    let amqp_conn = create_connection(&config.rabbitmq_url)
        .await
        .expect("Failed to connect to RabbitMQ");

    let consumer_db = db.clone();
    let consumer_config = config.clone();

    tokio::spawn(async move {
        if let Err(err) = consumer::start_consumer(&amqp_conn, consumer_db, consumer_config).await {
            tracing::error!("payment-service consumer crashed: {err}");
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "payment-service-ok" }))
        .route("/", get(|| async { "payment-service-running" }))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    tracing::info!("payment-service running on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
