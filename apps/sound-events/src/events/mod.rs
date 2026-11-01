//! Joining what the tagger scored frame by frame into events: when a sound starts and how long it
//! goes on.

mod event_contract;
mod event_times_algorithm;
mod merge_frames_algorithm;

pub use event_times_algorithm::event_times;
pub use merge_frames_algorithm::merge_frames;
