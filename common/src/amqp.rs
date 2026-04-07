use crate::errors::AppError;
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use tokio::time::{sleep, Duration};

pub async fn create_connection(amqp_url: &str) -> Result<Connection, AppError> {
    let mut last_error = None;

    for attempt in 1..=15 {
        match Connection::connect(amqp_url, ConnectionProperties::default()).await {
            Ok(conn) => {
                tracing::info!("Connected to RabbitMQ on attempt {}", attempt);
                return Ok(conn);
            }
            Err(err) => {
                tracing::warn!(
                    "RabbitMQ connection attempt {} failed: {}",
                    attempt,
                    err
                );
                last_error = Some(err.to_string());
                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Err(AppError::Broker(format!(
        "Failed to connect to RabbitMQ after retries: {}",
        last_error.unwrap_or_else(|| "unknown error".to_string())
    )))
}

pub async fn create_channel(conn: &Connection) -> Result<Channel, AppError> {
    conn.create_channel()
        .await
        .map_err(|e| AppError::Broker(format!("Failed to create channel: {e}")))
}

pub async fn declare_topic_exchange(
    channel: &Channel,
    exchange: &str,
) -> Result<(), AppError> {
    channel
        .exchange_declare(
            exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Failed to declare topic exchange: {e}")))?;

    Ok(())
}

pub async fn publish_json(
    channel: &Channel,
    exchange: &str,
    routing_key: &str,
    payload: &[u8],
) -> Result<(), AppError> {
    publish_message(channel, exchange, routing_key, payload, "application/json").await
}

pub async fn publish_message(
    channel: &Channel,
    exchange: &str,
    routing_key: &str,
    payload: &[u8],
    content_type: &str,
) -> Result<(), AppError> {
    channel
        .basic_publish(
            exchange,
            routing_key,
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default()
                .with_content_type(content_type.into())
                .with_delivery_mode(2),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Failed to publish message: {e}")))?
        .await
        .map_err(|e| AppError::Broker(format!("Failed to confirm published message: {e}")))?;

    Ok(())
}