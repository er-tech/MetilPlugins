//! The model the plugin uses and where its files live on the computer.

mod locate_models_use_case;
mod model_catalog_config;

pub use locate_models_use_case::{locate_models, ModelPaths};
