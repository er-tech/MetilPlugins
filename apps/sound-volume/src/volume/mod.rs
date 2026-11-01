//! How loud a sound is, over time: level and peak per window, the level of the low, mid and high
//! bands, the quiet stretches and a summary. Levels are in dB relative to full scale.

mod band_levels_algorithm;
mod find_silences_algorithm;
mod loudness_algorithm;
mod measure_volume_use_case;
mod summarise_volume_algorithm;
mod volume_window_contract;

pub use band_levels_algorithm::BandEdges;
pub use find_silences_algorithm::find_silences;
pub use measure_volume_use_case::{measure_volume, VolumeOptions};
pub use summarise_volume_algorithm::summarise_volume;
