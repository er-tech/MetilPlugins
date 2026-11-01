use super::find_alignment_use_case::{find_alignment, Alignment, AlignmentOptions};
use super::fit_timeline_use_case::fit_timeline;
use super::search_speed_use_case::search_speed;
use super::stretch_algorithm::stretch;

/// A speed this close to 1 is the same speed: over a song it is less than 0.1 s.
const SAME_SPEED: f64 = 0.0005;
/// How far either side of the first estimate the speed is searched, and how exact it is made.
const REACH: f64 = 0.006;
const STOP: f64 = 0.00002;

/// How the music lies on the video's timeline: `video time = offset + speed * music time`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Located {
    /// Where the music's start falls on the video's timeline, in samples.
    pub offset: f64,
    /// 1.0 when both play at the same speed; 1.05 when the video plays the music 5% slower.
    pub speed: f64,
    /// How far the match over the whole length stands out from chance (0 when it does not match as
    /// a whole, as with a video at another speed).
    pub score: f64,
    /// How many pieces of the music agreed on the speed, when enough were found.
    pub pieces: Option<usize>,
}

/// Why the music could not be placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unplaced {
    /// Too short or too quiet to compare.
    Silent,
    /// The best match over the whole length, which is not enough.
    NoMatch(Alignment),
}

/// Places `music` on the timeline of `video`: the delay and, when the video runs at another speed,
/// the speed too. It first looks for one delay over the whole length and for a line through
/// pieces of the music; when neither holds it tries speeds from 0.85 to 1.25 until three pieces
/// agree; then it stretches the music by the speed it found and measures again, so the answer is
/// as exact as the whole length allows.
pub fn locate_music(
    video: &[f32],
    music: &[f32],
    options: AlignmentOptions,
    min_score: f64,
) -> Result<Located, Unplaced> {
    let (global, first) = attempt(video, music, options, min_score);
    // A line through pieces of the music confirms the speed. Without one the whole-length match
    // may be a smeared match of a video at another speed, so the speeds are tried as well, and
    // the stronger match wins.
    let mut found = first;
    if found.is_none_or(|placed| placed.pieces.is_none()) {
        let tried = search_speed(video, music, options.sample_rate).and_then(|speed| {
            let (_, placed) = attempt(video, &stretch(music, speed), options, min_score);
            placed.map(|placed| {
                refine(
                    video,
                    music,
                    options,
                    min_score,
                    Located {
                        speed: speed * placed.speed,
                        ..placed
                    },
                )
            })
        });
        if let Some(tried) = tried {
            if found.is_none_or(|placed| tried.score > placed.score) {
                found = Some(tried);
            }
        }
    } else if let Some(placed) = found.filter(|placed| (placed.speed - 1.0).abs() > SAME_SPEED) {
        found = Some(refine(video, music, options, min_score, placed));
    }
    found.ok_or(match global {
        Some(best) => Unplaced::NoMatch(best),
        None => Unplaced::Silent,
    })
}

/// One look: the best delay over the whole length and a line through pieces. The line wins when
/// its speed is not 1; the whole-length delay wins otherwise. Placed only when one of them holds.
fn attempt(
    video: &[f32],
    music: &[f32],
    options: AlignmentOptions,
    min_score: f64,
) -> (Option<Alignment>, Option<Located>) {
    let global = find_alignment(video, music, options);
    let line = fit_timeline(video, music, options.sample_rate);
    let matched = global.filter(|found| found.score >= min_score);
    let score = global.map_or(0.0, |found| found.score);
    let placed = match (matched, line) {
        (Some(_), Some(line)) if (line.speed - 1.0).abs() > SAME_SPEED => Some(Located {
            offset: line.offset,
            speed: line.speed,
            score,
            pieces: Some(line.on_line),
        }),
        (Some(found), line) => Some(Located {
            offset: found.delay as f64,
            speed: 1.0,
            score,
            pieces: line.map(|line| line.on_line),
        }),
        (None, Some(line)) => Some(Located {
            offset: line.offset,
            speed: line.speed,
            score,
            pieces: Some(line.on_line),
        }),
        (None, None) => None,
    };
    (global, placed)
}

/// Narrows the speed found: the music is stretched by speeds close to it, golden-section search
/// keeps the one at which the match over the whole length is strongest, and the delay is read at
/// that speed. Keeps `first` when no speed there matches as a whole.
fn refine(
    video: &[f32],
    music: &[f32],
    options: AlignmentOptions,
    min_score: f64,
    first: Located,
) -> Located {
    let score_at = |speed: f64| {
        find_alignment(video, &stretch(music, speed), options).map_or(f64::MIN, |found| found.score)
    };
    let speed = golden_maximum(first.speed - REACH, first.speed + REACH, STOP, score_at);
    match find_alignment(video, &stretch(music, speed), options) {
        Some(found) if found.score >= min_score => Located {
            offset: found.delay as f64,
            speed,
            score: found.score,
            pieces: first.pieces,
        },
        _ => first,
    }
}

/// The place in `low..=high` where `value` is highest, by golden-section search, to within `stop`.
/// It assumes one hill in the range.
fn golden_maximum(mut low: f64, mut high: f64, stop: f64, value: impl Fn(f64) -> f64) -> f64 {
    let ratio = (5f64.sqrt() - 1.0) / 2.0;
    let mut left = high - ratio * (high - low);
    let mut right = low + ratio * (high - low);
    let (mut at_left, mut at_right) = (value(left), value(right));
    while high - low > stop {
        if at_left < at_right {
            low = left;
            left = right;
            at_left = at_right;
            right = low + ratio * (high - low);
            at_right = value(right);
        } else {
            high = right;
            right = left;
            at_right = at_left;
            left = high - ratio * (high - low);
            at_left = value(left);
        }
    }
    (low + high) / 2.0
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

    const OPTIONS: AlignmentOptions = AlignmentOptions {
        sample_rate: 8000,
        min_overlap: 40_000,
    };

    #[test]
    fn a_video_at_the_same_speed_gets_speed_one() {
        let music = tune(480_000);
        let mut video = vec![0.0; 24_000];
        video.extend_from_slice(&music);
        let placed = locate_music(&video, &music, OPTIONS, 30.0).unwrap();
        assert_eq!(placed.offset, 24_000.0);
        assert_eq!(placed.speed, 1.0);
    }

    #[test]
    fn a_video_four_percent_slower_is_placed_with_its_speed() {
        let music = tune(480_000);
        let mut video = vec![0.0; 24_000];
        video.extend(stretch(&music, 1.04));
        let placed = locate_music(&video, &music, OPTIONS, 30.0).unwrap();
        assert!(
            (placed.speed - 1.04).abs() < 0.001,
            "speed {}",
            placed.speed
        );
        assert!(
            (placed.offset - 24_000.0).abs() < 200.0,
            "offset {}",
            placed.offset
        );
    }

    #[test]
    fn a_video_eight_percent_faster_is_placed_with_its_speed() {
        let music = tune(480_000);
        let mut video = vec![0.0; 16_000];
        video.extend(stretch(&music, 0.92));
        let placed = locate_music(&video, &music, OPTIONS, 30.0).unwrap();
        assert!(
            (placed.speed - 0.92).abs() < 0.001,
            "speed {}",
            placed.speed
        );
        assert!(
            (placed.offset - 16_000.0).abs() < 200.0,
            "offset {}",
            placed.offset
        );
    }

    #[test]
    fn unrelated_sounds_are_not_placed() {
        let music = tune(480_000);
        let mut state = 5u32;
        let video: Vec<f32> = (0..480_000)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5
            })
            .collect();
        assert!(matches!(
            locate_music(&video, &music, OPTIONS, 30.0),
            Err(Unplaced::NoMatch(_))
        ));
    }
}
