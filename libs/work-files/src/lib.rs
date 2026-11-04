//! Files a plugin hands to the result. A plugin does not put the bytes of a picture or a clip in its
//! answer: it writes the file in its work folder and answers with `{ "workFile": <path>, "mime": <type> }`,
//! and Loomas moves the file into the result's store. These write the common kinds and give
//! that object.

mod names;
mod png;
mod wav;

pub use names::unique_work_file_name;
pub use png::write_png_file;
pub use wav::write_wav_file;

/// Why a file could not be written: the path and what the system said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkFileError(pub String);

impl std::fmt::Display for WorkFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WorkFileError {}
