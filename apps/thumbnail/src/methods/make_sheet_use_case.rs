use loomasplugins_frame_reader::{read_video_region, PixelFormat};
use loomasplugins_plugin_sdk::{whole_number, Cancel, Failure};
use loomasplugins_work_files::write_png_file;
use serde_json::{json, Value};

use crate::sheet::{choose_pictures, compose_sheet};

/// The space between the pictures of a sheet, in pixels.
const GAP: usize = 4;

/// The `sheet` action: one picture that shows a video as a grid of small pictures from along it.
///
/// Params: `media` (a video; the host decodes colour pictures of it, one every five seconds),
/// `columns` (1 to 12, default 4) and `count` (1 to 100, default 12: how many pictures, spread
/// from the start to the end). It gives `{ sheet, columns, rows, pictureWidth, pictureHeight,
/// pictures, durationMs }`: `sheet` is the stored PNG, `pictures` is `[{ timeMs }]` in the order
/// they are on the sheet (row by row), so a flow can say what is where.
pub fn make_sheet(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let columns = whole_number(params, "columns", 4, 1, 12)? as usize;
    let count = whole_number(params, "count", 12, 1, 100)? as usize;
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
            "this action reads colour pictures (rgb24)",
        ));
    }
    if video.frames.is_empty() {
        return Err(Failure::status(422, "the video has no picture to show"));
    }
    let work_dir = cancel
        .work_dir()
        .ok_or_else(|| Failure::status(500, "Loomas gave no work folder to write the sheet in"))?;

    let mut pictures: Vec<&[u8]> = Vec::new();
    let mut times = Vec::new();
    for index in choose_pictures(video.frames.len(), count) {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        let pixels = video
            .pixels(index)
            .ok_or_else(|| Failure::status(422, "a picture of the video could not be read"))?;
        pictures.push(pixels);
        times.push(json!({ "timeMs": video.frames[index].pts_ms }));
    }
    let sheet = compose_sheet(&pictures, video.width, video.height, columns, GAP);
    let file = write_png_file(
        "thumbnail-sheet",
        &sheet.rgb,
        sheet.width,
        sheet.height,
        work_dir,
    )
    .map_err(|error| Failure::status(500, error.to_string()))?;
    Ok(json!({
        "sheet": file,
        "columns": sheet.columns,
        "rows": sheet.rows,
        "pictureWidth": video.width,
        "pictureHeight": video.height,
        "pictures": times,
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A video region of three rgb24 pictures 2x2 (values 30, 130, 230), 10 seconds apart.
    fn region(name: &str) -> Value {
        let folder = std::env::temp_dir().join(format!("sheet-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let pixels: Vec<u8> = [30u8, 130, 230]
            .iter()
            .flat_map(|value| vec![*value; 12])
            .collect();
        let (file, list) = (folder.join("v.rgb24"), folder.join("v.frames.jsonl"));
        std::fs::write(&file, pixels).unwrap();
        let text: String = (0..3)
            .map(|n| {
                format!(
                    "{{\"index\":{n},\"ptsMs\":{},\"durationMs\":5000,\"isKeyFrame\":true}}\n",
                    n * 10_000
                )
            })
            .collect();
        std::fs::write(&list, text).unwrap();
        json!({ "kind": "video", "file": file.to_string_lossy(), "format": "rgb24", "width": 2,
                "height": 2, "frameBytes": 12, "frameCount": 3, "offsetBytes": 0,
                "framesFile": list.to_string_lossy(), "startMs": 0, "endMs": 30_000 })
    }

    fn cancel_with_work_dir(name: &str) -> (Cancel, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("sheet-work-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        (Cancel::with_work_dir(dir.clone()), dir)
    }

    #[test]
    fn the_sheet_is_a_png_of_the_pictures_with_their_times_in_order() {
        let (cancel, dir) = cancel_with_work_dir("ok");
        let result = make_sheet(
            &json!({ "media": region("ok"), "columns": 2, "count": 3 }),
            &cancel,
        )
        .unwrap();
        assert_eq!(result["rows"], 2);
        assert_eq!(result["columns"], 2);
        assert_eq!(result["pictureWidth"], 2);
        let times: Vec<u64> = result["pictures"]
            .as_array()
            .unwrap()
            .iter()
            .map(|picture| picture["timeMs"].as_u64().unwrap())
            .collect();
        assert_eq!(times, vec![0, 10_000, 20_000]);
        assert_eq!(result["sheet"]["mime"], "image/png");
        let file = result["sheet"]["workFile"].as_str().unwrap();
        assert!(std::path::Path::new(file).starts_with(&dir));
        assert!(std::fs::metadata(file).unwrap().len() > 0);
    }

    #[test]
    fn fewer_pictures_than_asked_for_make_a_smaller_sheet() {
        let (cancel, _) = cancel_with_work_dir("few");
        let result = make_sheet(
            &json!({ "media": region("few"), "columns": 12, "count": 50 }),
            &cancel,
        )
        .unwrap();
        assert_eq!(result["pictures"].as_array().unwrap().len(), 3);
        assert_eq!(
            (result["columns"].clone(), result["rows"].clone()),
            (json!(3), json!(1))
        );
    }

    #[test]
    fn bad_params_a_missing_video_and_a_missing_work_folder_are_refused() {
        let (cancel, _) = cancel_with_work_dir("bad");
        let media = region("bad");
        for params in [
            json!({ "media": media, "columns": 0 }),
            json!({ "media": media, "columns": 13 }),
            json!({ "media": media, "count": 101 }),
            json!({}),
            json!({ "media": "not a decoded video" }),
        ] {
            let failure = make_sheet(&params, &cancel).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{params}: {failure:?}");
        }
        let failure = make_sheet(&json!({ "media": media }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(500)));
    }
}
