use super::correlate_algorithm::correlate;

/// How the music's time maps onto the video's: `video time = offset + speed * music time`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timeline {
    /// Where the music's start falls on the video's timeline, in samples.
    pub offset: f64,
    /// How much longer the video takes than the music for the same stretch: 1.0 for the same
    /// speed, 1.01 when the video plays the music 1% slower.
    pub speed: f64,
    /// How many pieces of the music lie on the line, and how many were found at all.
    pub on_line: usize,
    pub found: usize,
}

/// The most pieces of the music that are looked for.
const PIECES: usize = 10;
/// How far a piece may lie from the line and still count, in milliseconds.
const TOLERANCE_MS: f64 = 100.0;
/// How far a piece's match must stand out from chance, in standard deviations.
const PIECE_SCORE: f64 = 10.0;

/// Finds how the music's time maps onto the video's by looking for short pieces of the music in the
/// whole video: where each piece is found gives a point, and a line through them (the most pieces
/// on it wins) gives the offset and the speed. It works for a video that was sped up or slowed
/// down, where one delay for the whole length does not exist. `None` when fewer than three pieces
/// agree.
pub fn fit_timeline(video: &[f32], music: &[f32], sample_rate: u32) -> Option<Timeline> {
    let piece_len = (10 * sample_rate as usize).min(music.len() / 3);
    if piece_len < 3 * sample_rate as usize || video.len() < piece_len {
        return None;
    }
    let loud = |samples: &[f32]| samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    let average = loud(music);
    let step = (music.len() - piece_len) / (PIECES - 1);
    let points: Vec<(f64, f64)> = (0..PIECES)
        .map(|i| i * step)
        .filter(|&start| loud(&music[start..start + piece_len]) > 0.2 * average)
        .filter_map(|start| {
            find_piece(video, &music[start..start + piece_len], sample_rate)
                .map(|delay| (start as f64, delay as f64))
        })
        .collect();
    let found = points.len();
    let tolerance = TOLERANCE_MS * f64::from(sample_rate) / 1000.0;
    let mut best: Vec<(f64, f64)> = Vec::new();
    for (i, &(m1, d1)) in points.iter().enumerate() {
        for &(m2, d2) in &points[i + 1..] {
            let speed = (d2 - d1) / (m2 - m1);
            if !(0.5..=2.0).contains(&speed) {
                continue;
            }
            let offset = d1 - speed * m1;
            let on_line: Vec<(f64, f64)> = points
                .iter()
                .copied()
                .filter(|&(m, d)| (d - (offset + speed * m)).abs() <= tolerance)
                .collect();
            if on_line.len() > best.len() {
                best = on_line;
            }
        }
    }
    if best.len() < 3 {
        return None;
    }
    let n = best.len() as f64;
    let mean_m = best.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_d = best.iter().map(|p| p.1).sum::<f64>() / n;
    let covariance: f64 = best.iter().map(|p| (p.0 - mean_m) * (p.1 - mean_d)).sum();
    let variance: f64 = best.iter().map(|p| (p.0 - mean_m).powi(2)).sum();
    let speed = covariance / variance;
    Some(Timeline {
        offset: mean_d - speed * mean_m,
        speed,
        on_line: best.len(),
        found,
    })
}

/// Where in `video` the piece of music lies completely (as a delay in samples), if that stands
/// out from chance.
fn find_piece(video: &[f32], piece: &[f32], sample_rate: u32) -> Option<isize> {
    let scores = correlate(video, piece);
    let first = -(piece.len() as isize - 1);
    let inside: Vec<(isize, f64)> = scores
        .iter()
        .enumerate()
        .map(|(i, &score)| (first + i as isize, f64::from(score)))
        .filter(|&(delay, _)| delay >= 0 && delay as usize + piece.len() <= video.len())
        .collect();
    let &(delay, peak) = inside.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
    let guard = (sample_rate / 10) as isize;
    let rest: Vec<f64> = inside
        .iter()
        .filter(|(other, _)| (other - delay).abs() > guard)
        .map(|&(_, score)| score)
        .collect();
    if rest.len() < 2 {
        return None;
    }
    let mean = rest.iter().sum::<f64>() / rest.len() as f64;
    let spread = (rest.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / rest.len() as f64).sqrt();
    (spread > 0.0 && (peak - mean) / spread >= PIECE_SCORE).then_some(delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Notes with a drum-like decaying noise on each: pure tones alone leave the timing ambiguous.
    fn tune(length: usize) -> Vec<f32> {
        let mut state = 3u32;
        let mut note = 200.0f32;
        let mut phase = 0.0f32;
        (0..length)
            .map(|i| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                let hiss = (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5;
                if i % 1500 == 0 {
                    note = 150.0 + (state >> 20) as f32 % 900.0;
                }
                phase += note / 8000.0 * std::f32::consts::TAU;
                let decay = (-((i % 1500) as f32) / 300.0).exp();
                phase.sin() * 0.3 + hiss * decay
            })
            .collect()
    }

    /// The music as a video plays it: starting `offset` samples in, `speed` times as slow.
    fn played(music: &[f32], offset: usize, speed: f64) -> Vec<f32> {
        let mut video = vec![0.0; offset];
        video.extend((0..(music.len() as f64 * speed) as usize).map(|n| {
            let at = (n as f64 / speed) as usize;
            music[at.min(music.len() - 1)]
        }));
        video
    }

    #[test]
    fn a_video_half_a_percent_slower_gives_a_rough_speed_and_offset() {
        let music = tune(480_000);
        let video = played(&music, 24_000, 1.005);
        let timeline = fit_timeline(&video, &music, 8000).unwrap();
        assert!(
            (timeline.speed - 1.005).abs() < 0.005,
            "speed {}",
            timeline.speed
        );
        assert!(
            (timeline.offset - 24_000.0).abs() < 400.0,
            "offset {}",
            timeline.offset
        );
        assert!(timeline.on_line >= 3);
    }

    #[test]
    fn a_copy_at_the_same_speed_has_speed_one() {
        let music = tune(480_000);
        let video = played(&music, 8_000, 1.0);
        let timeline = fit_timeline(&video, &music, 8000).unwrap();
        assert!((timeline.speed - 1.0).abs() < 0.0002);
    }

    #[test]
    fn unrelated_sounds_have_no_timeline() {
        let music = tune(480_000);
        let mut state = 99u32;
        let video: Vec<f32> = (0..480_000)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5
            })
            .collect();
        assert_eq!(fit_timeline(&video, &music, 8000), None);
    }
}
