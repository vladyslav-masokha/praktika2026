use futures_util::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, ExchangeKind,
};

use common::errors::AppError;

use crate::metrics::BenchmarkObservation;

pub async fn consume_benchmark_events<F>(
    channel: &Channel,
    exchange: &str,
    queue_name: &str,
    routing_key: &str,
    expected_messages: usize,
    mut decode_sent_at_ms: F,
) -> Result<Vec<BenchmarkObservation>, AppError>
where
    F: FnMut(&[u8]) -> Result<i64, AppError>,
{
    channel
        .exchange_declare(
            exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare benchmark exchange failed: {e}")))?;

    let queue = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Declare benchmark queue failed: {e}")))?;

    channel
        .queue_bind(
            queue.name().as_str(),
            exchange,
            routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Bind benchmark queue failed: {e}")))?;

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            &format!("{queue_name}-consumer"),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| AppError::Broker(format!("Start benchmark consumer failed: {e}")))?;

    let mut observations = Vec::with_capacity(expected_messages);

    while observations.len() < expected_messages {
        let delivery = consumer
            .next()
            .await
            .ok_or_else(|| AppError::Broker("Benchmark consumer stream ended unexpectedly".to_string()))?
            .map_err(|e| AppError::Broker(format!("Benchmark delivery failed: {e}")))?;

        let sent_at_ms = decode_sent_at_ms(&delivery.data)?;
        let received_at_ms = common::utils::now_millis();
        let latency_ms = (received_at_ms - sent_at_ms).max(0);

        observations.push(BenchmarkObservation {
            latency_ms,
            payload_size_bytes: delivery.data.len(),
        });

        delivery
            .ack(BasicAckOptions::default())
            .await
            .map_err(|e| AppError::Broker(format!("Benchmark ack failed: {e}")))?;
    }

    Ok(observations)
}
