use serde_json::{json, Value};
use loomasplugins_frame_reader::{read_video_region, PixelFormat, VideoRegion};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use loomasplugins_work_files::unique_work_file_name;

use crate::encode::{ffmpeg_program, frame_rate, Encoder, EncoderSettings};
use crate::models::{locate_model, ModelSpec, NETWORK_SCALE};
use crate::tiles::TILE_MARGIN;
use crate::upscale::{device_report, upscale_frame};

use super::read_upscale_options_mapper::read_upscale_options;

/// The largest side of the new video, as large as the host decodes.
const MAX_SIDE: usize = 16_384;

/// A side of a video made even: H.264 in yuv420p needs it.
fn even(side: usize) -> usize {
    (side & !1).max(2)
}

/// The `upscale` action: a video made larger.
///
/// Params: `media` (a video; the host decodes every colour picture of it), `model`, `device`,
/// `tile` and `scale` (see `read_upscale_options`). It gives
/// `{ video, frames, width, height, model, device }`: `video` is the new video, a stored MP4 with no
/// sound (the plugin sees pictures only), `width` and `height` are its size.
pub fn upscale_video(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_upscale_options(params)?;
    // A call that left the device to `auto` runs where the host grants.
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let video = read_video_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(
                400,
                "media must be a video Loomas decoded for this plugin (it asks for pictures)",
            )
        })?;
    if video.format != PixelFormat::Rgb24 {
        return Err(Failure::status(
            400,
            "this plugin reads colour pictures (rgb24)",
        ));
    }
    if video.frames.is_empty() {
        return Err(Failure::status(422, "the video has no picture"));
    }
    let (target_width, target_height) = (
        even(video.width * options.scale),
        even(video.height * options.scale),
    );
    if target_width.max(target_height) > MAX_SIDE {
        return Err(Failure::status(
            400,
            format!(
                "the new video would be {target_width} x {target_height} pixels; the most is {MAX_SIDE} on a side. Use scale 2"
            ),
        ));
    }
    let work_dir = cancel.work_dir().ok_or_else(|| {
        Failure::status(
            422,
            "Loomas gave this plugin no folder to write the video in",
        )
    })?;
    let file = locate_model(options.model, cancel)?;
    let grown = file.is_some();
    let side_scale = if grown { NETWORK_SCALE } else { 1 };
    let tile = options.model.fixed_tile.unwrap_or(options.tile);
    if grown && tile <= 2 * TILE_MARGIN {
        return Err(Failure::status(
            400,
            "tile is too small for the pieces to share their edges",
        ));
    }

    let output = work_dir.join(unique_work_file_name("video-upscale", "mp4"));
    let settings = EncoderSettings {
        in_width: video.width * side_scale,
        in_height: video.height * side_scale,
        out_width: target_width,
        out_height: target_height,
        frame_rate: frame_rate(video.frames.len(), video.start_ms, video.end_ms),
    };
    let program = ffmpeg_program(std::env::var_os("LOOMAS_FFMPEG"));
    let mut encoder = Encoder::start(&program, &settings, &output)?;
    let fed = feed_pictures(
        &video,
        options.model,
        options.device,
        file.as_deref(),
        tile,
        &mut encoder,
        cancel,
    );
    match fed {
        Ok(()) => encoder.finish()?,
        Err(failure) => {
            encoder.abort();
            let _ = std::fs::remove_file(&output);
            return Err(failure);
        }
    }
    Ok(json!({
        "video": { "workFile": output.to_string_lossy(), "mime": "video/mp4" },
        "frames": video.frames.len(),
        "width": target_width,
        "height": target_height,
        "model": options.model.name,
        "device": device_report(options.model, options.device).map(|report| report.to_value()),
    }))
}

/// Makes each picture larger (when the model is a network) and writes it to the encoder, in order.
fn feed_pictures(
    video: &VideoRegion,
    model: &ModelSpec,
    device: DeviceChoice,
    file: Option<&std::path::Path>,
    tile: usize,
    encoder: &mut Encoder,
    cancel: &Cancel,
) -> Result<(), Failure> {
    for index in 0..video.frames.len() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!("{index} of {} pictures", video.frames.len()));
        let pixels = video
            .pixels(index)
            .ok_or_else(|| Failure::status(500, "a picture of the video went missing"))?;
        match file {
            Some(file) => {
                let larger =
                    upscale_frame(model, device, file, pixels, video.width, video.height, tile)?;
                encoder.write_frame(&larger)?;
            }
            None => encoder.write_frame(pixels)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_a_decoded_video_and_a_missing_media_are_400s() {
        let failure = upscale_video(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = upscale_video(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = upscale_video(
            &json!({ "media": "a.mp4", "model": "unicorn" }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(failure.message.contains("unicorn"), "{}", failure.message);
    }

    #[test]
    fn a_side_is_made_even_and_never_below_two() {
        assert_eq!((even(1920), even(1081), even(1)), (1920, 1080, 2));
    }
}
