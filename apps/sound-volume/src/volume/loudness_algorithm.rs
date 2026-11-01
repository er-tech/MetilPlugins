/// The quietest level reported: anything below it, including silence, is -100 dB.
pub const FLOOR_DB: f64 = -100.0;

/// A level in decibels relative to full scale (0 dB is the loudest a sound can be) from a mean
/// square, which is the power of the samples. Silence and tiny values give [`FLOOR_DB`].
pub fn power_to_db(mean_square: f64) -> f64 {
    if mean_square <= 0.0 {
        return FLOOR_DB;
    }
    (10.0 * mean_square.log10()).max(FLOOR_DB)
}

/// The root mean square level of samples in dB: how loud they sound on average.
pub fn rms_db(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return FLOOR_DB;
    }
    let sum: f64 = samples
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum();
    power_to_db(sum / samples.len() as f64)
}

/// The highest single sample in dB: how loud the loudest instant is.
pub fn peak_db(samples: &[f32]) -> f64 {
    let peak = samples
        .iter()
        .fold(0.0_f64, |peak, sample| peak.max(f64::from(sample.abs())));
    if peak <= 0.0 {
        return FLOOR_DB;
    }
    (20.0 * peak.log10()).max(FLOOR_DB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_scale_is_zero_decibels_and_half_the_amplitude_is_six_less() {
        assert!(rms_db(&[1.0, -1.0, 1.0, -1.0]).abs() < 1e-9);
        assert!((rms_db(&[0.5, -0.5]) + 6.0206).abs() < 1e-3);
        assert!((peak_db(&[0.1, -0.5, 0.2]) + 6.0206).abs() < 1e-3);
    }

    #[test]
    fn silence_and_nothing_are_the_floor() {
        assert_eq!(rms_db(&[0.0; 100]), FLOOR_DB);
        assert_eq!(rms_db(&[]), FLOOR_DB);
        assert_eq!(peak_db(&[]), FLOOR_DB);
        assert_eq!(power_to_db(1e-30), FLOOR_DB);
    }

    #[test]
    fn a_sine_of_amplitude_one_half_is_about_minus_nine_decibels() {
        let sine: Vec<f32> = (0..8_000)
            .map(|n| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * n as f32 / 8_000.0).sin())
            .collect();
        assert!((rms_db(&sine) + 9.03).abs() < 0.05, "{}", rms_db(&sine));
        assert!((peak_db(&sine) + 6.02).abs() < 0.05);
    }
}
