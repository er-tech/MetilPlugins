use serde_json::{json, Value};

/// Pretends to find something every `every_ms` that lasts `length_ms`, in a video of
/// `duration_ms`. The last hit is cut at the end of the video.
pub fn find_hits(duration_ms: u64, every_ms: u64, length_ms: u64, label: &str) -> Vec<Value> {
    let every = every_ms.max(1);
    let mut hits = Vec::new();
    let mut start = 0;
    while start < duration_ms {
        let end = (start + length_ms).min(duration_ms);
        hits.push(json!({ "startMs": start, "endMs": end, "label": label }));
        start += every;
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starts_and_ends(hits: &[Value]) -> Vec<(u64, u64)> {
        hits.iter()
            .map(|hit| {
                (
                    hit["startMs"].as_u64().unwrap(),
                    hit["endMs"].as_u64().unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn finds_one_hit_per_interval_and_cuts_the_last_at_the_end() {
        let hits = find_hits(2_200, 1_000, 500, "cat");
        assert_eq!(
            starts_and_ends(&hits),
            [(0, 500), (1_000, 1_500), (2_000, 2_200)]
        );
        assert_eq!(hits[0]["label"], "cat");
    }

    #[test]
    fn an_interval_of_zero_does_not_loop_forever_and_an_empty_video_has_no_hits() {
        assert_eq!(find_hits(3, 0, 1, "x").len(), 3);
        assert!(find_hits(0, 1_000, 500, "x").is_empty());
    }
}
