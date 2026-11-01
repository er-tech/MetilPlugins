/// The steps a time axis may move in, in milliseconds: from a tenth of a second to an hour.
const STEPS_MS: [u64; 18] = [
    100, 200, 500, 1_000, 2_000, 5_000, 10_000, 15_000, 30_000, 60_000, 120_000, 300_000, 600_000,
    900_000, 1_800_000, 3_600_000, 7_200_000, 21_600_000,
];

/// A round step (1 s, 5 s, 15 s, 1 min ...) that gives about `target` marks on an axis that lasts
/// `duration_ms`.
pub fn nice_step_ms(duration_ms: u64, target: usize) -> u64 {
    let wanted = duration_ms / target.max(1) as u64;
    STEPS_MS
        .iter()
        .copied()
        .find(|step| *step >= wanted)
        .unwrap_or(STEPS_MS[STEPS_MS.len() - 1])
}

/// The marks of the axis: every multiple of the step inside the duration, from 0.
pub fn ticks_ms(duration_ms: u64, target: usize) -> Vec<u64> {
    let step = nice_step_ms(duration_ms, target);
    (0..)
        .map(|n| n * step)
        .take_while(|time| *time <= duration_ms)
        .collect()
}

/// A time for a label: `0:05`, `2:30`, `1:02:03`; tenths of a second are shown for steps below a second.
pub fn format_time(ms: u64, step_ms: u64) -> String {
    let (hours, minutes, seconds) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60);
    let fraction = if step_ms < 1000 {
        format!(".{}", ms % 1000 / 100)
    } else {
        String::new()
    };
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}{fraction}")
    } else {
        format!("{minutes}:{seconds:02}{fraction}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_step_is_a_round_time_that_gives_about_the_number_of_marks_asked_for() {
        assert_eq!(nice_step_ms(12_000, 12), 1_000);
        assert_eq!(nice_step_ms(125_000, 12), 15_000);
        assert_eq!(nice_step_ms(3_600_000, 12), 300_000);
        assert_eq!(nice_step_ms(500, 10), 100);
        assert_eq!(nice_step_ms(1_000_000_000, 10), 21_600_000);
        assert_eq!(nice_step_ms(0, 0), 100);
    }

    #[test]
    fn the_marks_start_at_zero_and_stay_inside_the_duration() {
        assert_eq!(ticks_ms(5_000, 5), [0, 1_000, 2_000, 3_000, 4_000, 5_000]);
        let long = ticks_ms(125_000, 12);
        assert_eq!(long.first(), Some(&0));
        assert!(long.last().is_some_and(|last| *last <= 125_000));
        assert_eq!(ticks_ms(0, 5), [0]);
    }

    #[test]
    fn times_read_as_minutes_and_seconds_with_hours_when_needed() {
        assert_eq!(format_time(0, 1_000), "0:00");
        assert_eq!(format_time(65_000, 5_000), "1:05");
        assert_eq!(format_time(3_723_000, 60_000), "1:02:03");
        assert_eq!(format_time(1_500, 100), "0:01.5");
    }
}
