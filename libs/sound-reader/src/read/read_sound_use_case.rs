use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::mix_to_mono_algorithm::mix_to_mono;
use super::read_sound_error::ReadSoundError;
use super::sound_contract::Sound;

/// Reads the first audio track of a file into one channel of samples, at the rate the file has.
pub fn read_sound(path: &Path) -> Result<Sound, ReadSoundError> {
    let file = File::open(path).map_err(|source| ReadSoundError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|error| ReadSoundError::Unsupported(error.to_string()))?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| ReadSoundError::Unsupported("the file has no audio".to_string()))?;
    let track_id = track.id;
    let mut sample_rate = track.codec_params.sample_rate;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|error| ReadSoundError::Unsupported(error.to_string()))?;

    let mut samples = Vec::new();
    let mut buffer: Option<SampleBuffer<f32>> = None;
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            // The end of the file is reported as an error by some formats.
            Err(Error::IoError(error)) if error.kind() == ErrorKind::UnexpectedEof => break,
            Err(Error::ResetRequired) => break,
            Err(error) => return Err(ReadSoundError::Decode(error.to_string())),
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                sample_rate.get_or_insert(spec.rate);
                let wanted = decoded.capacity() as u64;
                let frames = buffer.get_or_insert_with(|| SampleBuffer::<f32>::new(wanted, spec));
                if (frames.capacity() as u64) < wanted * spec.channels.count() as u64 {
                    *frames = SampleBuffer::<f32>::new(wanted, spec);
                }
                frames.copy_interleaved_ref(decoded);
                mix_to_mono(frames.samples(), spec.channels.count(), &mut samples);
            }
            // A damaged packet is skipped; the rest of the sound is still worth listening to.
            Err(Error::DecodeError(_)) => continue,
            Err(error) => return Err(ReadSoundError::Decode(error.to_string())),
        }
    }

    let sample_rate = sample_rate
        .filter(|rate| *rate > 0)
        .ok_or_else(|| ReadSoundError::Decode("the sample rate is not known".to_string()))?;
    if samples.is_empty() {
        return Err(ReadSoundError::Decode(
            "the file holds no sound".to_string(),
        ));
    }
    Ok(Sound {
        samples,
        sample_rate,
        start_ms: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 16-bit mono or stereo wav file in memory: a sine at `hertz` of `seconds`.
    fn wav(sample_rate: u32, channels: u16, hertz: f32, seconds: f32, amplitude: f32) -> Vec<u8> {
        let frames = (sample_rate as f32 * seconds) as usize;
        let mut data = Vec::new();
        for n in 0..frames {
            let value = (amplitude
                * (2.0 * std::f32::consts::PI * hertz * n as f32 / sample_rate as f32).sin()
                * f32::from(i16::MAX)) as i16;
            for _ in 0..channels {
                data.extend_from_slice(&value.to_le_bytes());
            }
        }
        let mut file = Vec::new();
        file.extend_from_slice(b"RIFF");
        file.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        file.extend_from_slice(b"WAVEfmt ");
        file.extend_from_slice(&16u32.to_le_bytes());
        file.extend_from_slice(&1u16.to_le_bytes());
        file.extend_from_slice(&channels.to_le_bytes());
        file.extend_from_slice(&sample_rate.to_le_bytes());
        file.extend_from_slice(&(sample_rate * u32::from(channels) * 2).to_le_bytes());
        file.extend_from_slice(&(channels * 2).to_le_bytes());
        file.extend_from_slice(&16u16.to_le_bytes());
        file.extend_from_slice(b"data");
        file.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file.extend_from_slice(&data);
        file
    }

    fn write(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("sound-reader-{}-{name}", std::process::id()));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn reads_a_wav_file_as_mono_samples_at_its_own_rate() {
        let path = write("mono.wav", &wav(8_000, 1, 440.0, 1.0, 0.5));
        let sound = read_sound(&path).unwrap();
        assert_eq!(sound.sample_rate, 8_000);
        assert_eq!(sound.samples.len(), 8_000);
        assert_eq!(sound.duration_ms(), 1_000);
        let peak = sound
            .samples
            .iter()
            .fold(0.0_f32, |peak, s| peak.max(s.abs()));
        assert!((peak - 0.5).abs() < 0.01, "{peak}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn two_channels_are_mixed_down_to_one() {
        let path = write("stereo.wav", &wav(8_000, 2, 440.0, 0.5, 0.25));
        let sound = read_sound(&path).unwrap();
        assert_eq!(sound.samples.len(), 4_000);
        let peak = sound
            .samples
            .iter()
            .fold(0.0_f32, |peak, s| peak.max(s.abs()));
        assert!((peak - 0.25).abs() < 0.01, "{peak}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_missing_file_a_file_that_is_not_a_sound_and_an_empty_one_are_told_apart() {
        let missing = read_sound(Path::new("no-such-sound.wav")).unwrap_err();
        assert!(matches!(missing, ReadSoundError::Open { .. }), "{missing}");

        let text = write("notes.txt", b"this is not a sound at all, just some words");
        let error = read_sound(&text).unwrap_err();
        assert!(matches!(error, ReadSoundError::Unsupported(_)), "{error}");
        let _ = std::fs::remove_file(text);

        let empty = write("empty.wav", &wav(8_000, 1, 440.0, 0.0, 0.5));
        assert!(read_sound(&empty).is_err());
        let _ = std::fs::remove_file(empty);
    }
}
