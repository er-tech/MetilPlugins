//! The signal processing round the network: the transform of a sound into frames of frequencies, the
//! features the network is given, the deep filter that applies its answer, and the transform back.

mod analyse_sound_algorithm;
mod apply_deep_filter_algorithm;
mod network_features_algorithm;
mod spectrogram_contract;
mod synthesise_sound_algorithm;

pub use analyse_sound_algorithm::analyse_sound;
pub use apply_deep_filter_algorithm::{apply_deep_filter, NetworkAnswer};
pub use network_features_algorithm::{network_features, NetworkFeatures};
pub use synthesise_sound_algorithm::synthesise_sound;
