/// When an event starts and ends, in milliseconds from the start of the sound.
///
/// A frame is `window_ms` long and a new one starts every `hop_ms`, so a frame "hears" a sound that
/// is only in its first or last part. Taking the whole frame would start an event up to a window
/// early and end it as much late. Each frame is taken as the `hop_ms` round its middle instead, which
/// keeps the error to about half a hop either way. The first frame starts at the start of the sound
/// and the last ends no later than the end of it (`duration_ms`).
pub fn event_times(
    first_frame: usize,
    last_frame: usize,
    hop_ms: u64,
    window_ms: u64,
    duration_ms: u64,
) -> (u64, u64) {
    let margin = window_ms.saturating_sub(hop_ms) / 2;
    let start = if first_frame == 0 {
        0
    } else {
        (first_frame as u64 * hop_ms + margin).min(duration_ms)
    };
    let end = (last_frame as u64 * hop_ms + margin + hop_ms).min(duration_ms);
    (start, end.max(start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_counts_as_the_step_round_its_middle() {
        // Frames every 480 ms, 975 ms long: the middle of frame 9 is at 4320 + 487 = 4807, so its
        // step is 4567 to 5047; frame 12's is 6007 to 6487.
        assert_eq!(event_times(9, 12, 480, 975, 60_000), (4567, 6487));
    }

    #[test]
    fn the_first_frame_starts_at_zero_and_the_last_is_held_at_the_end_of_the_sound() {
        assert_eq!(event_times(0, 3, 480, 975, 1_000), (0, 1_000));
        assert_eq!(event_times(5, 5, 480, 975, 100), (100, 100));
    }
}
