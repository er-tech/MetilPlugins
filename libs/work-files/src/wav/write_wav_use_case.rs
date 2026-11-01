use std::path::Path;

use serde_json::{json, Value};

use crate::names::unique_work_file_name;
use crate::WorkFileError;

use super::encode_wav_algorithm::encode_wav;

/// Writes `samples` (mono, from -1 to 1) as a 16-bit WAV named from `prefix` in `work_dir` and
/// describes it the way a result hands a file over: `{ "workFile": <path>, "mime": "audio/wav" }`.
pub fn write_wav_file(
    prefix: &str,
    samples: &[f32],
    sample_rate: u32,
    work_dir: &Path,
) -> Result<Value, WorkFileError> {
    let file = work_dir.join(unique_work_file_name(prefix, "wav"));
    std::fs::write(&file, encode_wav(samples, sample_rate))
        .map_err(|error| WorkFileError(format!("{}: {error}", file.display())))?;
    Ok(json!({ "workFile": file.to_string_lossy(), "mime": "audio/wav" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clip_is_written_in_the_work_folder_and_handed_over() {
        let folder = std::env::temp_dir().join(format!("work-files-wav-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let handed = write_wav_file("clip", &[0.0, 0.5], 16_000, &folder).unwrap();
        assert_eq!(handed["mime"], "audio/wav");
        let path = handed["workFile"].as_str().unwrap();
        assert!(Path::new(path).starts_with(&folder));
        assert_eq!(std::fs::metadata(path).unwrap().len(), 44 + 4);
    }

    #[test]
    fn a_folder_that_is_not_there_is_an_error_that_names_the_file() {
        let error =
            write_wav_file("clip", &[0.0], 16_000, Path::new("no/such/folder")).unwrap_err();
        assert!(error.0.contains("clip-"), "{error}");
    }
}
