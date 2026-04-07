mod app_state;
mod config;
mod consumer;
mod db;
mod handlers;
mod middleware;
mod models;
mod publisher;
mod repository;
mod routes;
mod services;
mod logistics;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    serve,
};
use reqwest::Client;
use common::amqp::{create_channel, create_connection, declare_topic_exchange};
use config::OrderServiceConfig;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app_state::AppState;

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
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

    let config = OrderServiceConfig::from_env();

    let db = db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let amqp_conn = create_connection(&config.rabbitmq_url)
        .await
        .expect("Failed to connect to RabbitMQ");

    let amqp_channel = create_channel(&amqp_conn)
        .await
        .expect("Failed to create AMQP channel");

    declare_topic_exchange(&amqp_channel, &config.exchange)
        .await
        .expect("Failed to declare exchange");

    let state = AppState {
        db,
        amqp_channel,
        exchange: config.exchange.clone(),
        jwt_secret: config.jwt_secret.clone(),
        http_client: Client::new(),
        logistics_provider: config.logistics_provider.clone(),
        logistics_api_base_url: config.logistics_api_base_url.clone(),
        logistics_api_key: config.logistics_api_key.clone(),
    };


    let consumer_state = state.clone();

    tokio::spawn(async move {
        if let Err(err) = consumer::start_consumer(&amqp_conn, consumer_state).await {
            tracing::error!("order-service consumer crashed: {err}");
        }
    });

    let app = routes::create_router(state)
        .layer(build_cors_layer(&config.cors_allowed_origins))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    tracing::info!("Order Service running on {}", addr);
    serve(listener, app).await.expect("Server error");
}