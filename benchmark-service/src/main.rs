mod consumer;
mod generator;
mod json_benchmark;
mod metrics;
mod protobuf_benchmark;
mod report;

use common::{
    amqp::{create_channel, create_connection, declare_topic_exchange},
    config::{get_env, get_env_or},
    errors::AppError,
};
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(err) = run().await {
        tracing::error!("benchmark-service failed: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let rabbitmq_url = get_env("RABBITMQ_URL");
    let exchange = get_env("AMQP_EXCHANGE");
    let total_events = get_env_or("BENCHMARK_TOTAL_EVENTS", "1000")
        .parse::<usize>()
        .unwrap_or(1000);
    let payload_size = get_env_or("BENCHMARK_PAYLOAD_SIZE", "128")
        .parse::<usize>()
        .unwrap_or(128);

    let connection = create_connection(&rabbitmq_url).await?;
    let channel = create_channel(&connection).await?;
    declare_topic_exchange(&channel, &exchange).await?;

    let json_summary = json_benchmark::run_json_benchmark(
        &connection,
        &channel,
        &exchange,
        total_events,
        payload_size,
    )
    .await?;

    let protobuf_summary = protobuf_benchmark::run_protobuf_benchmark(
        &connection,
        &channel,
        &exchange,
        total_events,
        payload_size,
    )
    .await?;

    println!("{}", report::render_report(&[json_summary, protobuf_summary]));
    Ok(())
}
