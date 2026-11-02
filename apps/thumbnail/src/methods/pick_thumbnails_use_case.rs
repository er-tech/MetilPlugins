use metilplugins_frame_reader::{read_video_region, PixelFormat};
use metilplugins_plugin_sdk::{whole_number, Cancel, Failure};
use serde_json::{json, Value};

use crate::pick::{choose_picks, score_picture, Candidate};

/// The `pick` action: the pictures that show a video best.
///
/// Params: `media` (a video; the host decodes pictures of it, in grey, one a second), `count`
/// (1 to 50, default 5) and `minGapMs` (0 to an hour, default 2000: how far apart two picks must
/// be, so they do not all come from the same scene). It gives `{ picks, candidates, durationMs }`:
/// `picks` is `[{ timeMs, score, sharpness, brightness }]` best first, the time to take the picture
/// at (with `media.frame`, for one).
pub fn pick_thumbnails(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let count = whole_number(params, "count", 5, 1, 50)? as usize;
    let min_gap_ms = whole_number(params, "minGapMs", 2_000, 0, 3_600_000)?;
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let video = read_video_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(
                400,
                "media must be a video Metil decoded for this plugin (it asks for pictures)",
            )
        })?;
    if video.format != PixelFormat::Gray8 {
        return Err(Failure::status(
            400,
            "this plugin reads grey pictures (gray8)",
        ));
    }

    let mut candidates = Vec::with_capacity(video.frames.len());
    for (index, frame) in video.frames.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        if index % 200 == 0 {
            cancel.progress(&format!("{index} of {} pictures", video.frames.len()));
        }
        let Some(pixels) = video.pixels(index) else {
            continue;
        };
        let (sharpness, brightness, score) = score_picture(pixels, video.width, video.height);
        candidates.push(Candidate {
            time_ms: frame.pts_ms,
            score,
            sharpness,
            brightness,
        });
    }
    let picks = choose_picks(&candidates, count, min_gap_ms);
    Ok(json!({
        "picks": picks.iter().copied().map(Candidate::to_value).collect::<Vec<_>>(),
        "candidates": candidates.len(),
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A video region of grey pictures 8x8: a sharp checkerboard, black, a smooth one with a little texture.
    fn region(name: &str) -> Value {
        let folder = std::env::temp_dir().join(format!("thumb-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let checkerboard: Vec<u8> = (0..64)
            .map(|n| if (n % 8 + n / 8) % 2 == 0 { 40 } else { 220 })
            .collect();
        let black = vec![0u8; 64];
        let ramp: Vec<u8> = (0..64).map(|n| 120 + ((n * 7) % 5) as u8).collect();
        let pixels: Vec<u8> = [checkerboard, black, ramp].concat();
        let (file, list) = (folder.join("v.gray8"), folder.join("v.frames.jsonl"));
        std::fs::write(&file, pixels).unwrap();
        let text: String = (0..3)
            .map(|n| {
                format!(
                    "{{\"index\":{n},\"ptsMs\":{},\"durationMs\":1000,\"isKeyFrame\":true}}\n",
                    n * 10_000
                )
            })
            .collect();
        std::fs::write(&list, text).unwrap();
        json!({ "kind": "video", "file": file.to_string_lossy(), "format": "gray8", "width": 8,
                "height": 8, "frameBytes": 64, "frameCount": 3, "offsetBytes": 0,
                "framesFile": list.to_string_lossy(), "startMs": 0, "endMs": 30_000 })
    }

    #[test]
    fn the_sharp_picture_wins_and_a_black_one_is_never_picked_and_times_are_the_sources() {
        let result = pick_thumbnails(
            &json!({ "media": region("pick"), "count": 3, "minGapMs": 0 }),
            &Cancel::default(),
        )
        .unwrap();
        let picks = result["picks"].as_array().unwrap();
        let times: Vec<u64> = picks
            .iter()
            .map(|p| p["timeMs"].as_u64().unwrap())
            .collect();
        assert_eq!(
            times,
            [0, 20_000],
            "the checkerboard first, then the textured one"
        );
        assert!(picks[0]["score"].as_f64().unwrap() > picks[1]["score"].as_f64().unwrap());
        assert_eq!(result["candidates"], 3);
        assert_eq!(result["durationMs"], 30_000);
    }

    #[test]
    fn a_gap_between_picks_and_the_wrong_input_are_handled() {
        let result = pick_thumbnails(
            &json!({ "media": region("gap"), "count": 3, "minGapMs": 15_000 }),
            &Cancel::default(),
        )
        .unwrap();
        // 0 ms is taken; 10 000 ms is too near it; 20 000 ms is far enough.
        let times: Vec<u64> = result["picks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pick| pick["timeMs"].as_u64().unwrap())
            .collect();
        assert_eq!(times, [0, 20_000]);

        let failure =
            pick_thumbnails(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = pick_thumbnails(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }
}
