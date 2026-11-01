/// The frame rate of a stretch of `count` pictures that runs from `start_ms` to `end_ms`. A video
/// with a changing rate gets its average. 25 when the stretch has no length to measure.
pub fn frame_rate(count: usize, start_ms: u64, end_ms: u64) -> f64 {
    let length = end_ms.saturating_sub(start_ms);
    if count == 0 || length == 0 {
        return 25.0;
    }
    count as f64 * 1000.0 / length as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rate_is_the_pictures_over_the_length_and_has_a_default() {
        assert_eq!(frame_rate(50, 0, 2000), 25.0);
        assert_eq!(frame_rate(30, 1000, 2000), 30.0);
        assert_eq!(frame_rate(0, 0, 2000), 25.0);
        assert_eq!(frame_rate(10, 500, 500), 25.0);
    }
}
