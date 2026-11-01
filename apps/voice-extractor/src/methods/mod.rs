//! The actions of the plugin: reading their params and giving their results.

mod extract_options_contract;
mod extract_voice_use_case;
mod read_extract_options_mapper;
mod run_method_use_case;

pub use run_method_use_case::run_method;
