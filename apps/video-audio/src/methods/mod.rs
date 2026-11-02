//! The actions of the plugin: reading their params and giving their results.

mod extract_audio_use_case;
mod normalize_audio_use_case;
mod read_audio_params_mapper;
mod run_method_use_case;

pub use run_method_use_case::run_method;
