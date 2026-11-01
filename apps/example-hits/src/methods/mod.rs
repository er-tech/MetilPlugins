//! What the plugin can do when it is called: the methods a recipe may name.

mod find_hits_algorithm;
mod interpret_response_algorithm;
mod measure_file_use_case;
mod run_method_use_case;
mod wait_use_case;

pub use run_method_use_case::run_method;
