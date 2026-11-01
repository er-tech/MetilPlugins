use rustfft::{num_complex::Complex, FftPlanner};

/// How well `music` lines up with `video` at every delay, by phase correlation: both sounds are
/// whitened (every frequency counts the same, only its timing matters), which survives the
/// different tone, noise and loudness of a low-quality copy better than plain correlation.
///
/// Entry `i` is the delay `i - (music.len() - 1)` samples: how late the music starts on the video's
/// timeline (negative: the video starts partway into the music).
pub(super) fn correlate(video: &[f32], music: &[f32]) -> Vec<f32> {
    if video.is_empty() || music.is_empty() {
        return Vec::new();
    }
    let size = (video.len() + music.len()).next_power_of_two();
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(size);
    let inverse = planner.plan_fft_inverse(size);
    let spectrum = |samples: &[f32]| {
        let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();
        buffer.resize(size, Complex::new(0.0, 0.0));
        forward.process(&mut buffer);
        buffer
    };
    let (a, b) = (spectrum(video), spectrum(music));
    let mut cross: Vec<Complex<f32>> = a.iter().zip(&b).map(|(x, y)| x * y.conj()).collect();
    let mean = cross.iter().map(|c| c.norm()).sum::<f32>() / size as f32;
    for value in &mut cross {
        *value /= value.norm() + 0.05 * mean + f32::MIN_POSITIVE;
    }
    inverse.process(&mut cross);
    let delays = video.len() + music.len() - 1;
    (0..delays)
        .map(|i| {
            let delay = i as isize - (music.len() as isize - 1);
            let at = delay.rem_euclid(size as isize) as usize;
            cross[at].re / size as f32
        })
        .collect()
}

/// How many samples of the two sounds overlap at `delay`.
pub(super) fn overlap(video_len: usize, music_len: usize, delay: isize) -> usize {
    let start = delay.max(0);
    let end = (delay + music_len as isize).min(video_len as isize);
    (end - start).max(0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn tune(length: usize, seed: u32) -> Vec<f32> {
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

    #[test]
    fn the_peak_is_at_the_delay_the_music_starts_late() {
        let music = tune(16_000, 1);
        let mut video = vec![0.0; 5_000];
        video.extend_from_slice(&music);
        video.extend(vec![0.0; 3_000]);
        let scores = correlate(&video, &music);
        let best = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        assert_eq!(best as isize - (music.len() as isize - 1), 5_000);
    }

    #[test]
    fn overlap_counts_the_samples_both_sounds_have() {
        assert_eq!(overlap(100, 40, 10), 40);
        assert_eq!(overlap(100, 40, -10), 30);
        assert_eq!(overlap(100, 40, 80), 20);
        assert_eq!(overlap(100, 40, 200), 0);
    }
}
