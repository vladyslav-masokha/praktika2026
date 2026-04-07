use lapin::{
    options::BasicPublishOptions,
    BasicProperties,
    Channel,
};
use prost::Message;

use common::{
    errors::AppError,
    events::{proto::BenchmarkEvent as ProtoBenchmarkEvent, BenchmarkEvent},
};

use crate::{consumer, generator, metrics};

pub async fn run_protobuf_benchmark(
    connection: &lapin::Connection,
    publish_channel: &Channel,
    exchange: &str,
    total_events: usize,
    payload_size: usize,
) -> Result<metrics::BenchmarkSummary, AppError> {
    let routing_key = "benchmark.protobuf";
    let queue_name = "benchmark-service.protobuf";
    
    let events = generator::build_events(total_events, payload_size);

    let consumer_channel = connection
        .create_channel()
        .await
        .map_err(|e| AppError::Broker(format!("Create Protobuf consumer channel failed: {e}")))?;

    let exchange_name = exchange.to_string();

    let consume_task = tokio::spawn(async move {
        consumer::consume_benchmark_events(
            &consumer_channel,
            &exchange_name,
            queue_name,
            routing_key,
            total_events,
            |payload| {
                let event = ProtoBenchmarkEvent::decode(payload)
                    .map_err(|e| AppError::Internal(format!("Decode Protobuf payload failed: {e}")))?;
                Ok(event.sent_at_ms)
            },
        )
        .await
    });

    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    for mut event in events {
        event.sent_at_ms = common::utils::now_millis();

        let mut payload = Vec::new();
        map_to_proto(&event)
            .encode(&mut payload)
            .map_err(|e| AppError::Internal(format!("Encode Protobuf payload failed: {e}")))?;

        let properties = BasicProperties::default().with_content_type("application/x-protobuf".into());

        publish_channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await
            .map_err(|e| AppError::Broker(format!("Publish Protobuf failed: {e}")))?;
    }

    let observations = consume_task
        .await
        .map_err(|e| AppError::Internal(format!("Join Protobuf benchmark task failed: {e}")))??;

    Ok(metrics::summarize("PROTOBUF", &observations))
}

fn map_to_proto(event: &BenchmarkEvent) -> ProtoBenchmarkEvent {
    ProtoBenchmarkEvent {
        event_id: event.event_id.clone(),
        sequence: event.sequence,
        sent_at_ms: event.sent_at_ms,
        payload: event.payload.clone(),
    }
}