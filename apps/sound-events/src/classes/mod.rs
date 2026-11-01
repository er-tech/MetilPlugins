//! The sounds the tagger knows: the 521 classes of AudioSet.

mod audioset_classes_config;
mod select_labels_validator;

pub use audioset_classes_config::AUDIOSET_CLASSES;
pub use select_labels_validator::select_labels;
