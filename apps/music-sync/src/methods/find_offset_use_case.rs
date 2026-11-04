use serde_json::{json, Value};
use loomasplugins_plugin_sdk::{number, whole_number, Cancel, Failure};
use loomasplugins_sound_reader::{load_sound_input, Sound};

use crate::alignment::{locate_music, AlignmentOptions, Unplaced};

/// The rate both sounds are listened at: enough to tell a song by its notes and rhythm, small
/// enough to compare half an hour of sound in memory.
const LISTEN_RATE: u32 = 8_000;
/// The longest sound it compares, in milliseconds.
const LONGEST_MS: u64 = 30 * 60 * 1000;

/// The `findOffset` action: how `music` lies on the timeline of `video`'s sound.
///
/// Params: `video` and `music` (both sounds, or a video for its sound), `minScoreSigma` (2 to 100,
/// default 30: how far the best match over the whole length must stand out from chance; below it
/// the call still succeeds when three or more pieces of the music agree on a line, and fails with
/// 422 otherwise) and `minOverlapMs` (default 5000: the least the two must share). It gives
/// `offsetMs` (where the music's start falls on the video's timeline; negative when the video starts
/// partway into the music), `speed` (how much longer the video takes than the music for the same
/// stretch: 1 for the same speed, 1.05 when the video plays the music 5% slower), `score`,
/// `overlapMs` and `piecesOnLine` (how many pieces of the music agreed on the speed, null when too
/// few were found to tell).
pub fn find_music_offset(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let min_score = number(params, "minScoreSigma", 30.0, 2.0, 100.0)?;
    let min_overlap_ms = whole_number(params, "minOverlapMs", 5_000, 500, LONGEST_MS)?;
    for key in ["video", "music"] {
        if params.get(key).is_none_or(Value::is_null) {
            return Err(Failure::status(400, format!("{key} is missing")));
        }
    }
    let video = listen(params, "video")?;
    let music = listen(params, "music")?;
    for (name, sound) in [("video", &video), ("music", &music)] {
        if sound.duration_ms() > LONGEST_MS {
            return Err(Failure::status(
                413,
                format!("the {name} is longer than 30 minutes, which is the most this compares"),
            ));
        }
    }
    let options = AlignmentOptions {
        sample_rate: LISTEN_RATE,
        min_overlap: (min_overlap_ms * u64::from(LISTEN_RATE) / 1000) as usize,
    };
    cancel.progress("comparing");
    let located = locate_music(&video.samples, &music.samples, options, min_score).map_err(
        |unplaced| match unplaced {
            Unplaced::Silent => Failure::status(
                422,
                "the sounds are too short or too quiet to compare (are they silent?)",
            ),
            Unplaced::NoMatch(best) => Failure::status(
                422,
                format!(
                    "no match: the best one stands out {:.1} times the spread of chance, less than the {min_score} asked for, and no speed made pieces of the music agree. Is this the same music?",
                    best.score
                ),
            )
            .with_details(json!({ "score": best.score, "offsetMs": best.offset_ms(LISTEN_RATE) })),
        },
    )?;
    if cancel.is_cancelled() {
        return Err(Failure::named("CANCELLED", "cancelled"));
    }
    let rate = f64::from(LISTEN_RATE);
    let shared_start = located.offset.max(0.0);
    let shared_end = (located.offset + located.speed * music.samples.len() as f64)
        .min(video.samples.len() as f64);
    Ok(json!({
        "offsetMs": (located.offset * 1000.0 / rate).round() as i64,
        "speed": (located.speed * 1_000_000.0).round() / 1_000_000.0,
        "score": (located.score * 10.0).round() / 10.0,
        "overlapMs": ((shared_end - shared_start).max(0.0) * 1000.0 / rate) as u64,
        "piecesOnLine": located.pieces,
    }))
}

/// One of the two sounds, at the rate it is compared at.
fn listen(params: &Value, key: &str) -> Result<Sound, Failure> {
    load_sound_input(&json!({ "audio": params[key], "sampleRate": LISTEN_RATE }))
        .map_err(|error| Failure::status(error.code, format!("{key}: {}", error.message)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_sound_is_a_400_that_names_it() {
        let failure =
            find_music_offset(&json!({ "video": "a.wav" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("music is missing"));
    }

    #[test]
    fn params_out_of_range_are_a_400() {
        let failure =
            find_music_offset(&json!({ "minScoreSigma": 1 }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
    }
}
