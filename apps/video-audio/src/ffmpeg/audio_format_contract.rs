/// A kind of audio file the plugin can write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Flac,
    Opus,
    Mp3,
}

impl AudioFormat {
    /// The format a param names (`wav`, `flac`, `opus` or `mp3`), or `None`.
    pub fn parse(name: &str) -> Option<AudioFormat> {
        match name {
            "wav" => Some(AudioFormat::Wav),
            "flac" => Some(AudioFormat::Flac),
            "opus" => Some(AudioFormat::Opus),
            "mp3" => Some(AudioFormat::Mp3),
            _ => None,
        }
    }

    /// The name of the format as a param spells it, which is also the file extension.
    pub fn name(self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Flac => "flac",
            AudioFormat::Opus => "opus",
            AudioFormat::Mp3 => "mp3",
        }
    }

    /// The media type of the file.
    pub fn mime(self) -> &'static str {
        match self {
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Opus => "audio/ogg",
            AudioFormat::Mp3 => "audio/mpeg",
        }
    }

    /// The FFmpeg options that choose the encoder.
    pub fn encoder_arguments(self) -> &'static [&'static str] {
        match self {
            AudioFormat::Wav => &["-c:a", "pcm_s16le"],
            AudioFormat::Flac => &["-c:a", "flac"],
            AudioFormat::Opus => &["-c:a", "libopus"],
            AudioFormat::Mp3 => &["-c:a", "libmp3lame", "-q:a", "2"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_format_is_known_by_its_name_and_a_made_up_one_is_not() {
        for name in ["wav", "flac", "opus", "mp3"] {
            assert_eq!(AudioFormat::parse(name).unwrap().name(), name);
        }
        assert_eq!(AudioFormat::parse("aac"), None);
        assert_eq!(AudioFormat::parse("WAV"), None);
    }
}
