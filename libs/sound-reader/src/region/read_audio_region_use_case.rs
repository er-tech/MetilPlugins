use std::fs::File;
use std::path::Path;

use memmap2::Mmap;
use serde_json::Value;

use crate::input::SoundInputError;
use crate::read::Sound;

/// The sound a region describes, or `None` when `value` is not a region (a path or a stored file).
///
/// A region is what Metil hands over for an input the manifest lists under `wants`: raw
/// little-endian 32-bit float samples in a file the core decoded once, described by `file`,
/// `format` (`f32le`), `sampleRate`, `channels`, `sampleCount`, `startMs` and `offsetBytes`. The file is mapped
/// read-only, so the operating system shares its pages with every other process that reads it, and
/// the description is checked against the file before a sample is read. Two channels are mixed down.
pub fn read_audio_region(value: &Value) -> Result<Option<Sound>, SoundInputError> {
    let Some(fields) = value.as_object() else {
        return Ok(None);
    };
    if fields.get("kind").and_then(Value::as_str) != Some("audio") {
        return Ok(None);
    }
    let number = |key: &str| {
        fields
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| SoundInputError::new(400, format!("the audio region has no {key}")))
    };
    if fields.get("format").and_then(Value::as_str) != Some("f32le") {
        return Err(SoundInputError::new(
            400,
            "the audio region is not in f32le, the only format read",
        ));
    }
    let file = fields
        .get("file")
        .and_then(Value::as_str)
        .ok_or_else(|| SoundInputError::new(400, "the audio region has no file"))?;
    let (sample_rate, channels, frames, offset, start_ms) = (
        number("sampleRate")?,
        number("channels")?,
        number("sampleCount")?,
        number("offsetBytes")?,
        number("startMs")?,
    );
    if !(1..=2).contains(&channels) || sample_rate == 0 || sample_rate > u64::from(u32::MAX) {
        return Err(SoundInputError::new(
            400,
            "the audio region must have 1 or 2 channels and a sample rate",
        ));
    }
    let wanted_bytes = frames
        .checked_mul(channels * 4)
        .and_then(|bytes| bytes.checked_add(offset))
        .ok_or_else(|| SoundInputError::new(400, "the audio region is too large"))?;

    let opened = File::open(Path::new(file))
        .map_err(|error| SoundInputError::new(404, format!("{file}: {error}")))?;
    let length = opened
        .metadata()
        .map_err(|error| SoundInputError::new(422, format!("{file}: {error}")))?
        .len();
    if wanted_bytes > length {
        return Err(SoundInputError::new(
            400,
            format!("the audio region says {wanted_bytes} bytes, but {file} has {length}"),
        ));
    }
    let sample_rate = sample_rate as u32;
    if frames == 0 {
        return Ok(Some(Sound {
            samples: Vec::new(),
            sample_rate,
            start_ms,
        }));
    }
    // Read-only: nothing here writes, and the core never changes the file once it has put it there.
    let map = unsafe { Mmap::map(&opened) }
        .map_err(|error| SoundInputError::new(422, format!("{file}: {error}")))?;
    let bytes = &map[offset as usize..wanted_bytes as usize];
    let (chunks, _) = bytes.as_chunks::<4>();
    let mut samples = chunks.iter().map(|sample| f32::from_le_bytes(*sample));
    let samples = if channels == 1 {
        samples.collect()
    } else {
        let mut mixed = Vec::with_capacity(frames as usize);
        while let (Some(left), Some(right)) = (samples.next(), samples.next()) {
            mixed.push((left + right) / 2.0);
        }
        mixed
    };
    Ok(Some(Sound {
        samples,
        sample_rate,
        start_ms,
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn file_of(name: &str, samples: &[f32]) -> String {
        let path = std::env::temp_dir().join(format!("region-{}-{name}.f32le", std::process::id()));
        let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        std::fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn region(file: &str, channels: u64, frames: u64) -> Value {
        json!({ "kind": "audio", "file": file, "format": "f32le", "sampleRate": 16000,
                "channels": channels, "sampleCount": frames, "startMs": 0, "offsetBytes": 0 })
    }

    #[test]
    fn mono_is_read_as_it_is_and_stereo_is_mixed_down() {
        let mono = file_of("mono", &[0.5, -0.5, 0.25]);
        let sound = read_audio_region(&region(&mono, 1, 3)).unwrap().unwrap();
        assert_eq!(
            (sound.sample_rate, sound.samples),
            (16_000, vec![0.5, -0.5, 0.25])
        );
        let stereo = file_of("stereo", &[1.0, 0.0, 0.5, 0.5]);
        let sound = read_audio_region(&region(&stereo, 2, 2)).unwrap().unwrap();
        assert_eq!(sound.samples, vec![0.5, 0.5]);
    }

    #[test]
    fn a_stretch_knows_where_it_starts_in_the_source() {
        let file = file_of("stretch", &[0.1, 0.2]);
        let mut stretch = region(&file, 1, 2);
        stretch["startMs"] = json!(2500);
        let sound = read_audio_region(&stretch).unwrap().unwrap();
        assert_eq!(sound.start_ms, 2500);
        stretch.as_object_mut().unwrap().remove("startMs");
        assert_eq!(read_audio_region(&stretch).unwrap_err().code, 400);
    }

    #[test]
    fn a_path_or_a_stored_file_is_not_a_region() {
        assert!(read_audio_region(&json!("a.wav")).unwrap().is_none());
        assert!(read_audio_region(&json!({ "file": "a.wav" }))
            .unwrap()
            .is_none());
    }

    #[test]
    fn a_description_that_does_not_fit_the_file_is_refused() {
        let file = file_of("short", &[0.0, 0.0]);
        let error = read_audio_region(&region(&file, 1, 3)).unwrap_err();
        assert_eq!(error.code, 400);
        let mut other = region(&file, 1, 2);
        other["format"] = json!("s16le");
        assert_eq!(read_audio_region(&other).unwrap_err().code, 400);
        let gone = region("no-such-file.f32le", 1, 1);
        assert_eq!(read_audio_region(&gone).unwrap_err().code, 404);
        let empty = region(&file, 1, 0);
        assert!(read_audio_region(&empty)
            .unwrap()
            .unwrap()
            .samples
            .is_empty());
    }
}
