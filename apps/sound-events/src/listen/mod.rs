//! Listening: giving the sound to the tagger and getting a score for each class in each frame.

mod score_frames_use_case;
mod yamnet_store;

pub use score_frames_use_case::{score_frames, HOP, WINDOW};
pub use yamnet_store::{device_report, CLASSES};
