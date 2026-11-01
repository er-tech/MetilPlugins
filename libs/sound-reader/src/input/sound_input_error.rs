/// Why the `audio` input of an action could not become a sound: an HTTP-style status code for the
/// plugin to fail with (400 the input is wrong, 404 the file is not there, 422 it is not a sound
/// that can be read) and a message for a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundInputError {
    pub code: u16,
    pub message: String,
}

impl SoundInputError {
    pub(crate) fn new(code: u16, message: impl Into<String>) -> Self {
        SoundInputError {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SoundInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SoundInputError {}
