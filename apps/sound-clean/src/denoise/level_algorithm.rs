/// How loud a stretch of sound is, in dB relative to full scale (0 is the loudest; silence is
/// -120).
pub fn level_db(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return -120.0;
    }
    let mean_square: f64 = samples
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    (10.0 * mean_square.max(1e-12).log10()).max(-120.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_scale_is_zero_decibels_half_is_six_less_and_silence_is_the_floor() {
        assert!((level_db(&[1.0, -1.0, 1.0]) - 0.0).abs() < 1e-9);
        assert!((level_db(&[0.5, -0.5]) + 6.02).abs() < 0.01);
        assert_eq!(level_db(&[0.0; 4]), -120.0);
        assert_eq!(level_db(&[]), -120.0);
    }
}
