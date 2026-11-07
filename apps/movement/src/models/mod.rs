//! The pose models the plugin can use and where their files live on the computer.

mod locate_models_use_case;
mod model_catalog_config;

pub use locate_models_use_case::{locate_models, ModelPaths};
pub use model_catalog_config::{model_named, ModelSpec, Pipeline, MODELS};
