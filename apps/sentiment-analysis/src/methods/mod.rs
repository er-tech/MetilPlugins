//! The actions of the plugin: reading their params and giving their results.

mod analyze_options_contract;
mod analyze_sentiment_use_case;
mod read_analyze_options_mapper;
mod run_method_use_case;

pub use run_method_use_case::run_method;
