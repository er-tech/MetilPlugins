use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

/// How often a wait looks at whether it was cancelled.
const STEP: Duration = Duration::from_millis(10);

/// Waits `params.ms` milliseconds, then says how long it waited. It stops early with an error if
/// the call is cancelled, which is how a host can see that cancelling reaches a plugin.
pub fn wait(params: &Value, is_cancelled: &dyn Fn() -> bool) -> Result<Value, String> {
    let wanted = Duration::from_millis(params["ms"].as_u64().unwrap_or(0));
    let started = Instant::now();
    while started.elapsed() < wanted {
        if is_cancelled() {
            return Err("cancelled".to_string());
        }
        thread::sleep(STEP.min(wanted - started.elapsed()));
    }
    Ok(json!({ "waitedMs": u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_the_time_asked_for() {
        let result = wait(&json!({ "ms": 40 }), &|| false).unwrap();
        assert!(result["waitedMs"].as_u64().unwrap() >= 40);
    }

    #[test]
    fn stops_with_an_error_when_cancelled() {
        let error = wait(&json!({ "ms": 30_000 }), &|| true).unwrap_err();
        assert_eq!(error, "cancelled");
    }
}
