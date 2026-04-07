use crate::metrics::BenchmarkSummary;

pub fn render_report(summaries: &[BenchmarkSummary]) -> String {
    let mut lines = Vec::new();
    lines.push("================ Benchmark Report ================".to_string());

    for summary in summaries {
        lines.push(format!("Format: {}", summary.label));
        lines.push(format!("  Total Events: {}", summary.total_events));
        lines.push(format!("  Avg Latency: {:.2} ms", summary.avg_latency_ms));
        lines.push(format!("  Min Latency: {} ms", summary.min_latency_ms));
        lines.push(format!("  Max Latency: {} ms", summary.max_latency_ms));
        lines.push(format!("  Avg Payload Size: {:.2} bytes", summary.avg_payload_size_bytes));
        lines.push(String::new());
    }

    lines.join("
")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::BenchmarkSummary;

    #[test]
    fn test_render_report() {
        let summaries = vec![
            BenchmarkSummary {
                label: "JSON".to_string(),
                total_events: 100,
                avg_latency_ms: 10.0,
                min_latency_ms: 5,
                max_latency_ms: 20,
                avg_payload_size_bytes: 200.0,
            }
        ];

        let report = render_report(&summaries);
        println!("{report}");
    }
}