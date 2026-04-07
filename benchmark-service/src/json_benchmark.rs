use lapin::{
    options::BasicPublishOptions,
    BasicProperties,
    Channel,
};

use common::{errors::AppError, events::BenchmarkEvent};

use crate::{consumer, generator, metrics};

pub async fn run_json_benchmark(
    connection: &lapin::Connection,
    publish_channel: &Channel,
    exchange: &str,
    total_events: usize,
    payload_size: usize,
) -> Result<metrics::BenchmarkSummary, AppError> {
    let routing_key = "benchmark.json";
    let queue_name = "benchmark-service.json";
    
    let events = generator::build_events(total_events, payload_size);

    let consumer_channel = connection
        .create_channel()
        .await
        .map_err(|e| AppError::Broker(format!("Create JSON consumer channel failed: {e}")))?;

    let exchange_name = exchange.to_string();

    let consume_task = tokio::spawn(async move {
        consumer::consume_benchmark_events(
            &consumer_channel,
            &exchange_name,
            queue_name,
            routing_key,
            total_events,
            |payload| {
                let event: BenchmarkEvent = serde_json::from_slice(payload)
                    .map_err(|e| AppError::Internal(format!("Decode JSON benchmark payload failed: {e}")))?;
                Ok(event.sent_at_ms)
            },
        )
        .await
    });

    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    for mut event in events {
        event.sent_at_ms = common::utils::now_millis();

        let payload = serde_json::to_vec(&event)
            .map_err(|e| AppError::Internal(format!("Encode JSON benchmark payload failed: {e}")))?;

        let properties = BasicProperties::default().with_content_type("application/json".into());

        publish_channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await
            .map_err(|e| AppError::Broker(format!("Publish JSON failed: {e}")))?;
    }

    let observations = consume_task
        .await
        .map_err(|e| AppError::Internal(format!("Join JSON benchmark task failed: {e}")))??;

    Ok(metrics::summarize("JSON", &observations))
}