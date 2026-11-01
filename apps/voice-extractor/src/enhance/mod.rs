//! Making the voice of a sound: the network round the signal processing, a piece at a time, and
//! splitting the sound into the voice and the rest.

mod enhance_piece_use_case;
mod enhance_voice_use_case;
mod split_voice_algorithm;

pub use enhance_voice_use_case::enhance_voice;
pub use split_voice_algorithm::{level_db, split_voice};
