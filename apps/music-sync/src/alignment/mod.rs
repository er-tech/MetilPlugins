//! Telling where one sound lies inside another: the delay that lines a music up with the sound of
//! a video, how sure that is, and the speed when the video plays the music faster or slower.

mod correlate_algorithm;
mod find_alignment_use_case;
mod fit_timeline_use_case;
mod locate_music_use_case;
mod search_speed_use_case;
mod stretch_algorithm;

pub use find_alignment_use_case::AlignmentOptions;
pub use locate_music_use_case::{locate_music, Unplaced};
