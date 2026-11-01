/// Why the description of the pictures could not become pictures: an HTTP-style status code for
/// the plugin to fail with (400 the description is wrong, 404 a file is not there, 422 a file
/// cannot be read) and a message for a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameError {
    pub code: u16,
    pub message: String,
}

impl FrameError {
    pub(crate) fn new(code: u16, message: impl Into<String>) -> Self {
        FrameError {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FrameError {}
