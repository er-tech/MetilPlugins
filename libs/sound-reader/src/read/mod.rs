//! Reading a sound file.

mod mix_to_mono_algorithm;
mod read_sound_error;
mod read_sound_use_case;
mod sound_contract;

pub use read_sound_error::ReadSoundError;
pub use read_sound_use_case::read_sound;
pub use sound_contract::Sound;
