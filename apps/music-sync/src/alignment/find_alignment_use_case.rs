use super::correlate_algorithm::{correlate, overlap};

/// What to look for.
#[derive(Debug, Clone, Copy)]
pub struct AlignmentOptions {
    pub sample_rate: u32,
    /// The least the two sounds must have in common at the delay found, in samples.
    pub min_overlap: usize,
}

/// Where the music lies on the video's timeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    /// How late the music starts on the video's timeline, in samples (negative: the video starts
    /// partway into the music).
    pub delay: isize,
    /// How far the best match stands out from the rest, in standard deviations.
    pub score: f64,
}

impl Alignment {
    pub fn offset_ms(&self, sample_rate: u32) -> i64 {
        (self.delay as f64 * 1000.0 / sample_rate as f64).round() as i64
    }
}

/// The delay at which `music` best matches `video`, or `None` when the sounds are too short or
/// too quiet to compare. Delays that leave less than `min_overlap` in common are not considered.
pub fn find_alignment(
    video: &[f32],
    music: &[f32],
    options: AlignmentOptions,
) -> Option<Alignment> {
    let scores = correlate(video, music);
    let first = -(music.len() as isize - 1);
    let wanted = options.min_overlap.min(video.len()).min(music.len());
    let valid: Vec<(isize, f32)> = scores
        .iter()
        .enumerate()
        .map(|(i, &score)| (first + i as isize, score))
        .filter(|&(delay, _)| overlap(video.len(), music.len(), delay) >= wanted)
        .collect();
    let &(delay, peak) = valid.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
    // The rest of the delays, away from the peak, say how high chance alone gets.
    let guard = (options.sample_rate / 10) as isize;
    let rest: Vec<f64> = valid
        .iter()
        .filter(|(other, _)| (other - delay).abs() > guard)
        .map(|&(_, score)| f64::from(score))
        .collect();
    if rest.len() < 2 {
        return None;
    }
    let mean = rest.iter().sum::<f64>() / rest.len() as f64;
    let spread = (rest.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / rest.len() as f64).sqrt();
    if spread <= 0.0 {
        return None;
    }
    Some(Alignment {
        delay,
        score: (f64::from(peak) - mean) / spread,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tune(length: usize, seed: u32) -> Vec<f32> {
        let mut state = seed;
        let mut note = 200.0f32;
        let mut phase = 0.0f32;
        (0..length)
            .map(|i| {
                if i % 2000 == 0 {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    note = 150.0 + (state >> 20) as f32 % 900.0;
                }
                phase += note / 8000.0 * std::f32::consts::TAU;
                phase.sin() * 0.5
            })
            .collect()
    }

    fn noise(length: usize, seed: u32, level: f32) -> Vec<f32> {
        let mut state = seed;
        (0..length)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                ((state >> 8) as f32 / (1u32 << 24) as f32 - 0.5) * 2.0 * level
            })
            .collect()
    }

    const OPTIONS: AlignmentOptions = AlignmentOptions {
        sample_rate: 8000,
        min_overlap: 8000,
    };

    #[test]
    fn a_music_that_starts_late_in_a_noisy_video_is_found_to_the_sample() {
        let music = tune(80_000, 7);
        let mut video = noise(24_000, 1, 0.3);
        // The video carries the music quieter and with noise on top.
        video.extend(
            music
                .iter()
                .zip(noise(80_000, 2, 0.5))
                .map(|(m, n)| m * 0.4 + n * 0.5),
        );
        video.extend(noise(16_000, 3, 0.3));
        let found = find_alignment(&video, &music, OPTIONS).unwrap();
        assert_eq!(found.delay, 24_000);
        assert!(found.score > 8.0, "score {}", found.score);
    }

    #[test]
    fn a_video_that_starts_inside_the_music_gives_a_negative_delay() {
        let music = tune(80_000, 9);
        let video = music[10_000..60_000].to_vec();
        let found = find_alignment(&video, &music, OPTIONS).unwrap();
        assert_eq!(found.delay, -10_000);
        assert_eq!(found.offset_ms(8000), -1250);
    }

    #[test]
    fn two_sounds_with_nothing_in_common_score_low() {
        let video = noise(60_000, 4, 0.5);
        let music = tune(60_000, 5);
        let found = find_alignment(&video, &music, OPTIONS).unwrap();
        assert!(found.score < 8.0, "score {}", found.score);
    }

    #[test]
    fn silence_cannot_be_compared() {
        assert_eq!(
            find_alignment(&[0.0; 20_000], &[0.0; 20_000], OPTIONS),
            None
        );
        assert_eq!(find_alignment(&[], &[0.1], OPTIONS), None);
    }
}
