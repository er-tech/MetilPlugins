use super::signal_contract::{Signal, Source};
use super::timed_item_contract::TimedItem;

/// The reason written for a window that is much louder than the rest.
const REASON: &str = "sudden loud sound";

/// The middle of the values. For an even count it is the mean of the two in the middle.
fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

/// Flags each window whose level is at least `jump_db` above the usual level of the whole video.
///
/// The usual level is the median of all windows, so a few loud windows do not raise it. The strength grows with the excess:
/// it is 1 at twice the jump and never below 0.3. Items without `value` are skipped.
pub fn volume_signals(windows: &[TimedItem], jump_db: f64) -> Vec<Signal> {
    let levels: Vec<f64> = windows.iter().filter_map(|window| window.value).collect();
    if levels.is_empty() {
        return Vec::new();
    }
    let usual = median(&levels);

    windows
        .iter()
        .filter_map(|window| {
            let excess = window.value? - usual;
            (excess >= jump_db).then(|| Signal {
                start_ms: window.start_ms,
                end_ms: window.end_ms,
                source: Source::Volume,
                reason: REASON.to_owned(),
                weight: (excess / (2.0 * jump_db)).clamp(0.3, 1.0),
                detail: format!("{excess:.0} dB louder than usual"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(start_ms: u64, db: f64) -> TimedItem {
        TimedItem {
            start_ms,
            end_ms: start_ms + 1_000,
            text: None,
            label: None,
            confidence: None,
            value: Some(db),
        }
    }

    #[test]
    fn a_window_far_above_the_usual_level_is_flagged() {
        let windows = [
            window(0, -40.0),
            window(1_000, -40.0),
            window(2_000, -20.0),
            window(3_000, -40.0),
        ];
        let signals = volume_signals(&windows, 15.0);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].start_ms, 2_000);
        assert_eq!(signals[0].detail, "20 dB louder than usual");
        assert!((signals[0].weight - 0.667).abs() < 0.01);
    }

    #[test]
    fn a_steady_sound_is_never_flagged() {
        let windows = [window(0, -30.0), window(1_000, -31.0), window(2_000, -29.0)];
        assert!(volume_signals(&windows, 15.0).is_empty());
    }

    #[test]
    fn the_strength_is_one_at_twice_the_jump_and_at_least_a_third() {
        let windows = [
            window(0, -60.0),
            window(1_000, -60.0),
            window(2_000, -60.0),
            window(3_000, -20.0),
            window(4_000, -44.0),
        ];
        let signals = volume_signals(&windows, 15.0);
        assert!((signals[0].weight - 1.0).abs() < 1e-9);
        assert!((signals[1].weight - 0.533).abs() < 0.01);
    }

    #[test]
    fn no_values_means_no_signals() {
        let item = TimedItem {
            value: None,
            ..window(0, 0.0)
        };
        assert!(volume_signals(&[item], 15.0).is_empty());
    }
}
