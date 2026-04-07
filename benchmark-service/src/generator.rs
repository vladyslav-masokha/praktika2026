use common::events::BenchmarkEvent;
use uuid::Uuid;

pub fn build_events(total_events: usize, payload_size: usize) -> Vec<BenchmarkEvent> {
    let payload = "X".repeat(payload_size.max(1));

    (0..total_events)
        .map(|index| {
            BenchmarkEvent::new(
                Uuid::new_v4().to_string(),
                index as i64 + 1,
                payload.clone(),
            )
        })
        .collect()
}