use std::fmt;
use std::path::PathBuf;

/// Why a sound could not be read.
#[derive(Debug)]
pub enum ReadSoundError {
    /// The file is not there or cannot be opened.
    Open {
        path: PathBuf,
        source: std::io::Error,
    },
    /// No known format fits the file, or it has no audio.
    Unsupported(String),
    /// The file is damaged or cannot be decoded.
    Decode(String),
}

impl fmt::Display for ReadSoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadSoundError::Open { path, source } => write!(f, "{}: {source}", path.display()),
            ReadSoundError::Unsupported(reason) => {
                write!(f, "cannot read this as a sound: {reason}")
            }
            ReadSoundError::Decode(reason) => write!(f, "cannot decode the sound: {reason}"),
        }
    }
}

impl std::error::Error for ReadSoundError {}
