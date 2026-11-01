use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use serde_json::{json, Value};

use crate::names::unique_work_file_name;
use crate::WorkFileError;

/// Writes an `rgb24` picture as a PNG named from `prefix` in `work_dir` and describes it the way a
/// result hands a file over: `{ "workFile": <path>, "mime": "image/png" }`.
pub fn write_png_file(
    prefix: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
    work_dir: &Path,
) -> Result<Value, WorkFileError> {
    let file = work_dir.join(unique_work_file_name(prefix, "png"));
    let broken =
        |error: &dyn std::fmt::Display| WorkFileError(format!("{}: {error}", file.display()));
    if rgb.len() != width * height * 3 {
        return Err(broken(&format!(
            "{} bytes are not a picture of {width} x {height}",
            rgb.len()
        )));
    }
    let output = File::create(&file).map_err(|error| broken(&error))?;
    let mut encoder = png::Encoder::new(BufWriter::new(output), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| broken(&error))?;
    writer
        .write_image_data(rgb)
        .map_err(|error| broken(&error))?;
    writer.finish().map_err(|error| broken(&error))?;
    Ok(json!({ "workFile": file.to_string_lossy(), "mime": "image/png" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_is_written_as_a_png_that_reads_back_the_same() {
        let folder = std::env::temp_dir().join(format!("work-files-png-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let rgb: Vec<u8> = (0..4 * 2 * 3).map(|n| (n * 9) as u8).collect();

        let handed = write_png_file("shot", &rgb, 4, 2, &folder).unwrap();

        assert_eq!(handed["mime"], "image/png");
        let path = handed["workFile"].as_str().unwrap();
        assert!(Path::new(path).starts_with(&folder));
        let decoder = png::Decoder::new(std::io::BufReader::new(File::open(path).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut back = vec![0u8; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut back).unwrap();
        assert_eq!((info.width, info.height), (4, 2));
        assert_eq!(&back[..24], &rgb[..]);
    }

    #[test]
    fn a_missing_folder_and_a_wrong_size_are_errors_that_name_the_file() {
        let error = write_png_file("shot", &[0; 3], 1, 1, Path::new("no/such/folder")).unwrap_err();
        assert!(error.0.contains("shot-"), "{error}");
        let error = write_png_file("shot", &[0; 5], 1, 1, &std::env::temp_dir()).unwrap_err();
        assert!(error.0.contains("not a picture"), "{error}");
    }
}
