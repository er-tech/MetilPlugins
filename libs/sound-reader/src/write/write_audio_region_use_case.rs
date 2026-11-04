use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};

use crate::read::Sound;

static WRITTEN: AtomicU64 = AtomicU64::new(0);

/// Writes `sound` in `folder` as raw little-endian 32-bit float samples, one channel, in a file
/// that no other call of this process uses, and gives its description in the form Loomas hands
/// over decoded sound: the next plugin is given it as it is. `folder` is what Loomas offered
/// the plugin to write in; `sound.start_ms` says where the sound starts in the source, so the next
/// plugin answers in the time of the source.
pub fn write_audio_region(folder: &Path, name: &str, sound: &Sound) -> io::Result<Value> {
    std::fs::create_dir_all(folder)?;
    let number = WRITTEN.fetch_add(1, Ordering::SeqCst);
    let file = folder.join(format!(
        "{}-{}-{number}.f32le",
        name.replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "_"),
        std::process::id()
    ));
    let bytes: Vec<u8> = sound
        .samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    std::fs::write(&file, &bytes)?;
    let rate = u64::from(sound.sample_rate);
    let samples = sound.samples.len() as u64;
    Ok(json!({
        "kind": "audio",
        "file": file.to_string_lossy(),
        "format": "f32le",
        "sampleRate": sound.sample_rate,
        "channels": 1,
        "sampleCount": samples,
        "startSample": sound.start_ms * rate / 1000,
        "startMs": sound.start_ms,
        "endMs": sound.start_ms + samples * 1000 / rate.max(1),
        "durationMs": samples * 1000 / rate.max(1),
        "offsetBytes": 0,
        "bytes": bytes.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::read_audio_region;

    #[test]
    fn what_is_written_is_read_back_by_the_reader_of_the_next_plugin() {
        let folder = std::env::temp_dir().join(format!("sound-write-{}", std::process::id()));
        let sound = Sound {
            samples: vec![0.25, -0.5, 1.0, 0.0],
            sample_rate: 16_000,
            start_ms: 3_000,
        };
        let described = write_audio_region(&folder, "clean", &sound).unwrap();
        assert_eq!(described["sampleCount"], 4);
        assert_eq!(described["startMs"], 3_000);
        assert_eq!(described["bytes"], 16);
        let read = read_audio_region(&described).unwrap().unwrap();
        assert_eq!(read, sound);

        // A second sound does not overwrite the first.
        let other = write_audio_region(&folder, "clean", &sound).unwrap();
        assert_ne!(other["file"], described["file"]);
        let _ = std::fs::remove_dir_all(&folder);
    }
}
