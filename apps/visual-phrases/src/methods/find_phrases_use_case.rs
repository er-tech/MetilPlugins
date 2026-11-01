use serde_json::{json, Value};
use metilplugins_appearances::{track_appearances, Presence, Sample};
use metilplugins_frame_reader::{read_video_region, PixelFormat};
use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::{Cancel, Failure};
use metilplugins_work_files::write_png_file;

use crate::matching::match_probability;
use crate::models::locate_models;
use crate::phrases::embed_phrases;
use crate::pictures::{embed_all_pictures, Picture};
use crate::session::{vision_device_report, DIMENSIONS};

use super::read_find_options_mapper::read_find_options;

/// The `find` action: when a phrase describes what a video shows.
///
/// Params: `media` (a video; the host decodes colour pictures of it), `phrases`, `minConfidence`,
/// `gapMs` and `minSamples` (see `read_find_options`). It gives
/// `{ appearances, samples, durationMs }`. An appearance is
/// `{ phrase, startMs, endMs, confidence, samples, picture }`: the first picture where the phrase
/// holds (a stored PNG) and its time in the source. A phrase that stays true is one appearance;
/// one that stops for longer than `gapMs` and comes back is two.
pub fn find_phrases(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_find_options(params)?;
    // A call that asked for `auto` runs where the host grants; the default stays the processor.
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
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
    if video.format != PixelFormat::Rgb24 {
        return Err(Failure::status(
            400,
            "this plugin reads colour pictures (rgb24)",
        ));
    }
    let paths = locate_models(cancel)?;
    let phrase_numbers = embed_phrases(&paths, &options.phrases)?;

    // The pictures the video gave, with their place in the list of frames.
    let pictures: Vec<(usize, Picture)> = (0..video.frames.len())
        .filter_map(|index| {
            video.pixels(index).map(|rgb| {
                (
                    index,
                    Picture {
                        rgb,
                        width: video.width,
                        height: video.height,
                    },
                )
            })
        })
        .collect();
    let only_pictures: Vec<Picture> = pictures
        .iter()
        .map(|(_, picture)| Picture {
            rgb: picture.rgb,
            width: picture.width,
            height: picture.height,
        })
        .collect();
    let picture_numbers =
        embed_all_pictures(&paths.vision, options.device, &only_pictures, |done| {
            if cancel.is_cancelled() {
                return Err(Failure::named("CANCELLED", "cancelled"));
            }
            cancel.progress(&format!("{done} of {} pictures", only_pictures.len()));
            Ok(())
        })?;

    let samples: Vec<Sample> = pictures
        .iter()
        .enumerate()
        .map(|(at, (index, _))| {
            let numbers = &picture_numbers[at * DIMENSIONS..(at + 1) * DIMENSIONS];
            let frame = video.frames[*index];
            Sample {
                frame: *index,
                time_ms: frame.pts_ms,
                duration_ms: frame.duration_ms,
                present: phrase_numbers
                    .chunks(DIMENSIONS)
                    .enumerate()
                    .map(|(key, phrase)| Presence {
                        key,
                        score: match_probability(numbers, phrase),
                    })
                    .filter(|presence| presence.score >= options.min_confidence)
                    .collect(),
            }
        })
        .collect();

    let appearances = track_appearances(&samples, options.gap_ms, options.min_samples);
    let mut described = Vec::with_capacity(appearances.len());
    for appearance in &appearances {
        let work_dir = cancel.work_dir().ok_or_else(|| {
            Failure::status(
                422,
                "Metil gave this plugin no folder to write pictures in",
            )
        })?;
        let rgb = video
            .pixels(appearance.first_frame)
            .ok_or_else(|| Failure::status(500, "a picture of the video went missing"))?;
        described.push(json!({
            "phrase": options.phrases[appearance.key],
            "label": options.phrases[appearance.key],
            "startMs": appearance.start_ms,
            "endMs": appearance.end_ms,
            "confidence": (f64::from(appearance.confidence) * 1000.0).round() / 1000.0,
            "samples": appearance.samples,
            "picture": write_png_file("visual-phrases", rgb, video.width, video.height, work_dir)
                .map_err(|error| Failure::status(500, error.to_string()))?,
        }));
    }
    Ok(json!({
        "appearances": described,
        "samples": samples.len(),
        "device": vision_device_report().map(|report| report.to_value()),
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
    }))
}

#[cfg(test)]
mod tests {
    use metilplugins_onnx_device::DeviceChoice;

    use super::*;

    #[test]
    fn a_path_in_place_of_a_decoded_video_and_a_missing_media_are_400s() {
        let params = json!({ "media": "a.mp4", "phrases": ["a cat"] });
        let failure = find_phrases(&params, &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure =
            find_phrases(&json!({ "phrases": ["a cat"] }), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = find_phrases(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("phrases"), "{}", failure.message);
    }

    /// The model, when it is already on this computer (the first real run downloads it).
    fn cached_models() -> Option<crate::models::ModelPaths> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()?;
        let folder = std::path::PathBuf::from(home).join(".metil/models/visual-phrases");
        let paths = crate::models::ModelPaths {
            vision: folder.join("vision_model_int8.onnx"),
            text: folder.join("text_model_int8.onnx"),
            tokenizer: folder.join("tokenizer.json"),
        };
        [&paths.vision, &paths.text, &paths.tokenizer]
            .iter()
            .all(|file| file.is_file())
            .then_some(paths)
    }

    #[test]
    fn a_plain_colour_matches_its_own_phrase_and_not_another_with_the_real_model() {
        let Some(paths) = cached_models() else {
            eprintln!("skipped: the model is not downloaded yet");
            return;
        };
        let phrases = vec!["a black screen".to_string(), "a photo of a dog".to_string()];
        let words = embed_phrases(&paths, &phrases).unwrap();
        let (black, grey) = (vec![0u8; 64 * 48 * 3], vec![200u8; 64 * 48 * 3]);
        let pictures = [&black, &grey].map(|rgb| Picture {
            rgb,
            width: 64,
            height: 48,
        });
        let numbers =
            embed_all_pictures(&paths.vision, DeviceChoice::Cpu, &pictures, |_| Ok(())).unwrap();

        let score = |picture: usize, phrase: usize| {
            match_probability(
                &numbers[picture * DIMENSIONS..(picture + 1) * DIMENSIONS],
                &words[phrase * DIMENSIONS..(phrase + 1) * DIMENSIONS],
            )
        };
        assert!(
            score(0, 0) > 0.1,
            "black screen vs a black picture: {}",
            score(0, 0)
        );
        assert!(
            score(0, 0) > score(1, 0),
            "a black picture matches it better than a grey one"
        );
        assert!(
            score(0, 1) < 0.01,
            "a dog in a black picture: {}",
            score(0, 1)
        );
    }
}
