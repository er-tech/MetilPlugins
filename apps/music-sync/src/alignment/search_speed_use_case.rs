use super::correlate_algorithm::correlate;
use super::stretch_algorithm::{decimate, stretch};

/// The slowest and fastest play looked for, and the step between two tried: over a piece of
/// `PIECE_SECONDS` a step smears the match by about 2.5 ms, which a look at a quarter of the rate
/// still sees (a step of 0.002 already loses a degraded copy).
const SLOWEST: f64 = 0.85;
const FASTEST: f64 = 1.25;
const STEP: f64 = 0.001;
const PIECE_SECONDS: usize = 5;
const PIECES: [f64; 5] = [0.1, 0.3, 0.5, 0.7, 0.9];
/// How far a piece's match must stand out from chance, in standard deviations.
const PIECE_SCORE: f64 = 8.0;
/// How far two pieces' offsets may differ and still agree, in milliseconds.
const TOLERANCE_MS: f64 = 120.0;

/// The speed at which the video plays the music, when it is not close to 1: five short pieces of
/// the music are stretched by each speed in turn and looked for in the video at a quarter of the
/// rate; the speed at which the most pieces agree on where the music starts wins. `None` when no
/// speed makes at least two pieces agree. It takes seconds per minute of video (about 400 speeds
/// times five pieces). The answer is a first estimate, good to about 0.1%.
pub fn search_speed(video: &[f32], music: &[f32], sample_rate: u32) -> Option<f64> {
    let rate = f64::from(sample_rate) / 4.0;
    let (video, music) = (decimate(video), decimate(music));
    let piece_len = PIECE_SECONDS * rate as usize;
    if music.len() < 6 * piece_len || video.len() < piece_len {
        return None;
    }
    let starts: Vec<usize> = PIECES
        .iter()
        .map(|fraction| (music.len() as f64 * fraction) as usize - piece_len / 2)
        .collect();
    let tolerance = TOLERANCE_MS * rate / 1000.0;
    let mut best: Option<(f64, f64)> = None;
    let mut speed = SLOWEST;
    while speed <= FASTEST {
        // Where each stretched piece is found, as the start of the music that would explain it.
        let found: Vec<(f64, f64)> = starts
            .iter()
            .filter_map(|&start| {
                let piece = stretch(&music[start..start + piece_len], speed);
                let (delay, z) = best_place(&video, &piece)?;
                (z >= PIECE_SCORE).then_some((delay as f64 - speed * start as f64, z))
            })
            .collect();
        let value = found
            .iter()
            .map(|&(offset, _)| {
                found
                    .iter()
                    .filter(|other| (other.0 - offset).abs() <= tolerance)
                    .map(|other| other.1)
                    .sum::<f64>()
            })
            .fold(0.0, f64::max);
        let agreeing = found
            .iter()
            .filter(|(offset, _)| {
                found
                    .iter()
                    .filter(|other| (other.0 - offset).abs() <= tolerance)
                    .count()
                    >= 2
            })
            .count();
        if agreeing >= 2 && best.is_none_or(|(_, top)| value > top) {
            best = Some((speed, value));
        }
        speed += STEP;
    }
    best.map(|(speed, _)| speed)
}

/// Where `piece` lies completely inside `video` best (as a delay in samples), and how far that
/// stands out from chance in standard deviations.
fn best_place(video: &[f32], piece: &[f32]) -> Option<(isize, f64)> {
    if video.len() < piece.len() {
        return None;
    }
    let scores = correlate(video, piece);
    let first = -(piece.len() as isize - 1);
    let inside: Vec<(isize, f64)> = scores
        .iter()
        .enumerate()
        .map(|(i, &score)| (first + i as isize, f64::from(score)))
        .filter(|&(delay, _)| delay >= 0 && delay as usize + piece.len() <= video.len())
        .collect();
    let &(delay, peak) = inside.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
    let rest: Vec<f64> = inside
        .iter()
        .filter(|(other, _)| (other - delay).abs() > 200)
        .map(|&(_, score)| score)
        .collect();
    if rest.len() < 2 {
        return None;
    }
    let mean = rest.iter().sum::<f64>() / rest.len() as f64;
    let spread = (rest.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / rest.len() as f64).sqrt();
    (spread > 0.0).then_some((delay, (peak - mean) / spread))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn a_video_eight_percent_faster_is_found_within_half_a_percent() {
        let music = tune(480_000);
        let mut video = vec![0.0; 16_000];
        video.extend(stretch(&music, 0.92));
        let speed = search_speed(&video, &music, 8000).unwrap();
        assert!((speed - 0.92).abs() < 0.005, "speed {speed}");
    }

    #[test]
    fn unrelated_sounds_have_no_speed() {
        let music = tune(480_000);
        let mut state = 5u32;
        let video: Vec<f32> = (0..480_000)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5
            })
            .collect();
        assert_eq!(search_speed(&video, &music, 8000), None);
    }
}
