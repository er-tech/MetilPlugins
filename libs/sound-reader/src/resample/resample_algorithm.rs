/// Changes the sample rate of a sound from `from` to `to` hertz.
///
/// Going down, each new sample is the average of the old ones it covers (which also filters out
/// what the lower rate cannot hold, roughly); going up, new samples are read off a straight line
/// between the old ones. Good enough to measure a sound, not meant for music production.
pub fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || samples.is_empty() || from == 0 || to == 0 {
        return samples.to_vec();
    }
    let ratio = f64::from(from) / f64::from(to);
    let length = (samples.len() as f64 / ratio).floor() as usize;
    let last = samples.len() - 1;
    let mut out = Vec::with_capacity(length);
    if ratio > 1.0 {
        for i in 0..length {
            let start = i as f64 * ratio;
            let end = start + ratio;
            let (mut sum, mut position) = (0.0_f64, start);
            while position < end {
                let index = position.floor() as usize;
                let next = (index as f64 + 1.0).min(end);
                sum += f64::from(samples[index.min(last)]) * (next - position);
                position = next;
            }
            out.push((sum / ratio) as f32);
        }
    } else {
        for i in 0..length {
            let position = i as f64 * ratio;
            let index = position.floor() as usize;
            let fraction = (position - index as f64) as f32;
            let (a, b) = (samples[index.min(last)], samples[(index + 1).min(last)]);
            out.push(a + (b - a) * fraction);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    }

    #[test]
    fn the_same_rate_gives_the_same_samples() {
        assert_eq!(resample(&[0.1, 0.2], 8_000, 8_000), [0.1, 0.2]);
        assert!(resample(&[], 8_000, 4_000).is_empty());
    }

    #[test]
    fn going_down_keeps_a_steady_level_and_makes_the_sound_the_right_length() {
        let steady = vec![0.5; 16_000];
        let down = resample(&steady, 16_000, 8_000);
        assert_eq!(down.len(), 8_000);
        assert!(down.iter().all(|s| (s - 0.5).abs() < 1e-6));
        // A rate that does not divide evenly.
        assert_eq!(resample(&vec![0.0; 44_100], 44_100, 16_000).len(), 16_000);
    }

    #[test]
    fn going_up_follows_a_line_between_the_old_samples() {
        let up = resample(&[0.0, 1.0, 0.0], 1_000, 2_000);
        assert_eq!(up.len(), 6);
        assert!(
            (up[1] - 0.5).abs() < 1e-6 && (up[2] - 1.0).abs() < 1e-6 && (up[3] - 0.5).abs() < 1e-6
        );
    }

    #[test]
    fn a_low_tone_keeps_its_level_and_a_tone_above_the_new_rate_is_mostly_removed() {
        let tone = |hertz: f32| -> Vec<f32> {
            (0..32_000)
                .map(|n| (2.0 * std::f32::consts::PI * hertz * n as f32 / 32_000.0).sin())
                .collect()
        };
        let low = resample(&tone(200.0), 32_000, 8_000);
        assert!((rms(&low) - 0.707).abs() < 0.05, "{}", rms(&low));
        // 7000 Hz cannot exist at 8000 Hz (the limit is 4000): it must not come through at full level.
        let high = resample(&tone(7_000.0), 32_000, 8_000);
        assert!(rms(&high) < 0.5, "{}", rms(&high));
    }
}
