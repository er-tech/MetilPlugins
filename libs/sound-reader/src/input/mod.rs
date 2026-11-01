//! The `audio` input of an action: a stored file or a path, read into a sound.

mod load_sound_input_use_case;
mod sound_input_error;

pub use load_sound_input_use_case::load_sound_input;
pub use sound_input_error::SoundInputError;
