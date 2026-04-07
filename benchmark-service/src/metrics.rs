#[derive(Debug, Clone)]
pub struct BenchmarkObservation {
    pub latency_ms: i64,
    pub payload_size_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub label: String,
    pub total_events: usize,
    pub avg_latency_ms: f64,
    pub min_latency_ms: i64,
    pub max_latency_ms: i64,
    pub avg_payload_size_bytes: f64,
}

pub fn summarize(label: &str, observations: &[BenchmarkObservation]) -> BenchmarkSummary {
    if observations.is_empty() {
        return BenchmarkSummary {
            label: label.to_string(),
            total_events: 0,
            avg_latency_ms: 0.0,
            min_latency_ms: 0,
            max_latency_ms: 0,
            avg_payload_size_bytes: 0.0,
        };
    }

    let total_events = observations.len();
    let latency_sum: i64 = observations.iter().map(|item| item.latency_ms).sum();
    let payload_sum: usize = observations.iter().map(|item| item.payload_size_bytes).sum();
    let min_latency_ms = observations.iter().map(|item| item.latency_ms).min().unwrap_or(0);
    let max_latency_ms = observations.iter().map(|item| item.latency_ms).max().unwrap_or(0);

    BenchmarkSummary {
        label: label.to_string(),
        total_events,
        avg_latency_ms: latency_sum as f64 / total_events as f64,
        min_latency_ms,
        max_latency_ms,
        avg_payload_size_bytes: payload_sum as f64 / total_events as f64,
    }
}
