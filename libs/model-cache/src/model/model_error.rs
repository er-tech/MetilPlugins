/// Why a model could not be had: a status code with the meaning of the HTTP status of that number
/// (422 there is no place to keep it, 503 it could not be downloaded) and a message for a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    pub code: u16,
    pub message: String,
    /// Whether trying again could work.
    pub retryable: bool,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ModelError {}
