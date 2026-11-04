//! Reads a sound file into samples, for the plugins that listen: any common audio format (mp3,
//! wav, flac, ogg, aac and alac, also the sound of an mp4), mixed down to one channel as numbers
//! between -1 and 1, and resampled to another rate when one is wanted. Pure Rust, nothing to install.
//!
//! A plugin that lists `wants` in its manifest gets the sound already decoded by Loomas, as a
//! region of raw samples in a file that is mapped (`read_audio_region`); then it needs no decoder.
//!
//! `load_sound_input` is what an action calls: it takes the `audio` and `sampleRate` of its params
//! and gives a sound, or the status code and message to fail with.

mod input;
mod read;
mod region;
mod resample;
mod write;

pub use input::{load_sound_input, SoundInputError};
pub use read::{read_sound, ReadSoundError, Sound};
pub use region::read_audio_region;
pub use resample::resample;
pub use write::write_audio_region;
