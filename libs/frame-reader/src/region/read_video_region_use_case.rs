use std::fs::File;
use std::path::Path;

use memmap2::Mmap;
use serde_json::Value;

use super::frame_error::FrameError;
use super::video_region_contract::{Frame, PixelFormat, VideoRegion};

/// The pictures a description names, or `None` when `value` is not one (a path or a stored file).
///
/// The description is what Metil hands over for an input the manifest lists under `wants` with
/// `"media": "video"`: `kind` `video`, `file` (raw pixels, a picture after another), `format`
/// (`rgb24` or `gray8`), `width`, `height`, `frameBytes`, `frameCount`, `offsetBytes`, `startMs`,
/// `endMs` and `framesFile` (one JSON object a line: `index`, `ptsMs`, `durationMs`,
/// `isKeyFrame`). The file is mapped read-only, so the operating system shares its pages with every
/// other process that reads it, and the description is checked against both files before a picture
/// is read.
pub fn read_video_region(value: &Value) -> Result<Option<VideoRegion>, FrameError> {
    let Some(fields) = value.as_object() else {
        return Ok(None);
    };
    if fields.get("kind").and_then(Value::as_str) != Some("video") {
        return Ok(None);
    }
    let number = |key: &str| {
        fields
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| FrameError::new(400, format!("the video region has no {key}")))
    };
    let text = |key: &str| {
        fields
            .get(key)
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| FrameError::new(400, format!("the video region has no {key}")))
    };
    let format = match text("format")? {
        "rgb24" => PixelFormat::Rgb24,
        "gray8" => PixelFormat::Gray8,
        other => {
            return Err(FrameError::new(
                400,
                format!("the video region is in {other:?}, and only rgb24 and gray8 are read"),
            ))
        }
    };
    let (width, height) = (number("width")? as usize, number("height")? as usize);
    let (count, offset) = (
        number("frameCount")? as usize,
        number("offsetBytes")? as usize,
    );
    let frame_bytes = width * height * format.bytes_per_pixel();
    if width == 0 || height == 0 || number("frameBytes")? as usize != frame_bytes {
        return Err(FrameError::new(
            400,
            "the video region's frameBytes is not width x height x bytes a pixel",
        ));
    }
    let file = text("file")?;
    let wanted = count
        .checked_mul(frame_bytes)
        .and_then(|bytes| bytes.checked_add(offset))
        .ok_or_else(|| FrameError::new(400, "the video region is too large"))?;
    let opened = File::open(Path::new(file))
        .map_err(|error| FrameError::new(404, format!("{file}: {error}")))?;
    let length = opened
        .metadata()
        .map_err(|error| FrameError::new(422, format!("{file}: {error}")))?
        .len() as usize;
    if wanted > length {
        return Err(FrameError::new(
            400,
            format!("the video region says {wanted} bytes, but {file} has {length}"),
        ));
    }
    let frames = read_list(text("framesFile")?)?;
    if frames.len() != count {
        return Err(FrameError::new(
            400,
            format!(
                "the video region says {count} pictures, but its list has {}",
                frames.len()
            ),
        ));
    }
    // Read-only: nothing here writes, and the host never changes the file once it is there.
    let map = if count == 0 {
        None
    } else {
        Some(
            unsafe { Mmap::map(&opened) }
                .map_err(|error| FrameError::new(422, format!("{file}: {error}")))?,
        )
    };
    Ok(Some(VideoRegion {
        width,
        height,
        format,
        start_ms: number("startMs")?,
        end_ms: number("endMs")?,
        frames,
        map,
        offset,
    }))
}

fn read_list(file: &str) -> Result<Vec<Frame>, FrameError> {
    let text = std::fs::read_to_string(file)
        .map_err(|error| FrameError::new(404, format!("{file}: {error}")))?;
    text.lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line)
                .map_err(|error| FrameError::new(422, format!("{file}: {error}")))?;
            let number = |key: &str| {
                value[key]
                    .as_u64()
                    .ok_or_else(|| FrameError::new(422, format!("{file}: a picture has no {key}")))
            };
            Ok(Frame {
                index: number("index")?,
                pts_ms: number("ptsMs")?,
                duration_ms: number("durationMs")?,
                is_key_frame: value["isKeyFrame"].as_bool().unwrap_or(false),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Two 2x2 grey pictures and their list, in a folder of their own.
    fn files(name: &str) -> (String, String) {
        let folder = std::env::temp_dir().join(format!("frames-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let pixels = folder.join("a.gray8");
        std::fs::write(&pixels, [1u8, 2, 3, 4, 10, 20, 30, 40]).unwrap();
        let list = folder.join("a.frames.jsonl");
        std::fs::write(
            &list,
            "{\"index\":0,\"ptsMs\":0,\"durationMs\":500,\"isKeyFrame\":true}\n\
             {\"index\":1,\"ptsMs\":1000,\"durationMs\":500,\"isKeyFrame\":false}\n",
        )
        .unwrap();
        (
            pixels.to_string_lossy().into_owned(),
            list.to_string_lossy().into_owned(),
        )
    }

    fn region(pixels: &str, list: &str) -> Value {
        json!({ "kind": "video", "file": pixels, "format": "gray8", "width": 2, "height": 2,
                "frameBytes": 4, "frameCount": 2, "framesFile": list, "startMs": 0, "endMs": 1500,
                "offsetBytes": 0 })
    }

    #[test]
    fn the_pictures_are_read_with_their_times() {
        let (pixels, list) = files("read");
        let video = read_video_region(&region(&pixels, &list)).unwrap().unwrap();
        assert_eq!((video.width, video.height, video.frames.len()), (2, 2, 2));
        assert_eq!(video.pixels(0), Some(&[1u8, 2, 3, 4][..]));
        assert_eq!(video.pixels(1), Some(&[10u8, 20, 30, 40][..]));
        assert_eq!(video.pixels(2), None);
        assert_eq!(video.frames[1].pts_ms, 1000);
        assert!(video.frames[0].is_key_frame && !video.frames[1].is_key_frame);
    }

    #[test]
    fn a_path_or_a_sound_is_not_a_video_region() {
        assert!(read_video_region(&json!("a.mp4")).unwrap().is_none());
        assert!(read_video_region(&json!({ "kind": "audio" }))
            .unwrap()
            .is_none());
    }

    #[test]
    fn a_description_that_does_not_fit_the_files_is_refused() {
        let (pixels, list) = files("refuse");
        let mut longer = region(&pixels, &list);
        longer["frameCount"] = json!(3);
        assert_eq!(read_video_region(&longer).unwrap_err().code, 400);
        let mut size = region(&pixels, &list);
        size["frameBytes"] = json!(5);
        assert_eq!(read_video_region(&size).unwrap_err().code, 400);
        let mut format = region(&pixels, &list);
        format["format"] = json!("yuv420p");
        assert_eq!(read_video_region(&format).unwrap_err().code, 400);
        let mut gone = region(&pixels, &list);
        gone["file"] = json!("no-such-file.gray8");
        assert_eq!(read_video_region(&gone).unwrap_err().code, 404);
        let mut list_gone = region(&pixels, &list);
        list_gone["framesFile"] = json!("no-such-list.jsonl");
        assert_eq!(read_video_region(&list_gone).unwrap_err().code, 404);
    }
}
