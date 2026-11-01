/// The samples played `speed` times as slow (`speed` 1.05 is 5% longer), by linear interpolation.
/// The start stays where it is, so a delay measured on the result is a delay of the original.
pub(super) fn stretch(samples: &[f32], speed: f64) -> Vec<f32> {
    let length = (samples.len() as f64 * speed) as usize;
    (0..length)
        .map(|n| {
            let at = n as f64 / speed;
            let index = at as usize;
            let next = samples
                .get(index + 1)
                .copied()
                .unwrap_or(samples[samples.len() - 1]);
            let fraction = (at - index as f64) as f32;
            samples[index.min(samples.len() - 1)] * (1.0 - fraction) + next * fraction
        })
        .collect()
}

/// Every fourth sample, after averaging four: a quarter of the rate for a cheap first look.
pub(super) fn decimate(samples: &[f32]) -> Vec<f32> {
    samples
        .as_chunks::<4>()
        .0
        .iter()
        .map(|four| four.iter().sum::<f32>() / 4.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slower_play_is_longer_and_keeps_its_start() {
        let samples: Vec<f32> = (0..100).map(|n| n as f32).collect();
        let slow = stretch(&samples, 2.0);
        assert_eq!(slow.len(), 200);
        assert_eq!(slow[0], 0.0);
        assert!((slow[100] - 50.0).abs() < 1e-4);
        assert!((slow[1] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn decimating_averages_four_samples_into_one() {
        assert_eq!(decimate(&[1.0, 2.0, 3.0, 6.0, 9.0]), vec![3.0]);
    }
}
