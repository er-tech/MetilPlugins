/// The voice and the rest of a mix, given the voice the network made of it (`enhanced`). The voice
/// is that, with `keep` of the original sound added back (0 for the voice as it is, a little more
/// for a voice with fewer artifacts and some of the noise); the rest is what the network took out:
/// the mix less the voice it made. The rest is not clean: some of the voice is always left in it.
pub fn split_voice(mix: &[f32], enhanced: &[f32], keep: f32) -> (Vec<f32>, Vec<f32>) {
    let voice = enhanced
        .iter()
        .zip(mix)
        .map(|(voice, mix)| voice + keep * mix)
        .collect();
    let rest = mix
        .iter()
        .zip(enhanced)
        .map(|(mix, voice)| mix - voice)
        .collect();
    (voice, rest)
}

/// How loud a sound is, in decibels (the root mean square against full scale), -120 for silence.
pub fn level_db(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return -120.0;
    }
    let mean_square = samples
        .iter()
        .map(|s| f64::from(*s) * f64::from(*s))
        .sum::<f64>()
        / samples.len() as f64;
    (10.0 * mean_square.max(1e-12).log10()).max(-120.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rest_is_what_the_voice_does_not_have_and_keep_adds_some_of_the_mix_to_the_voice() {
        let mix = [1.0, 0.5, -0.5];
        let enhanced = [0.6, 0.5, -0.1];
        let (voice, rest) = split_voice(&mix, &enhanced, 0.0);
        let close = |a: &[f32], b: &[f32]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        assert_eq!(voice, enhanced);
        assert!(close(&rest, &[0.4, 0.0, -0.4]), "{rest:?}");
        let (blended, same_rest) = split_voice(&mix, &enhanced, 0.5);
        assert!(close(&blended, &[1.1, 0.75, -0.35]), "{blended:?}");
        assert_eq!(same_rest, rest);
    }

    #[test]
    fn a_full_scale_square_wave_is_zero_decibels_and_silence_is_the_floor() {
        assert!((level_db(&[1.0, -1.0, 1.0, -1.0])).abs() < 1e-9);
        assert!((level_db(&[0.1; 100]) + 20.0).abs() < 1e-6);
        assert_eq!(level_db(&[0.0; 10]), -120.0);
        assert_eq!(level_db(&[]), -120.0);
    }
}
