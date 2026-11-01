//! The upscalers the plugin can use and where their files live on the computer.

mod locate_model_use_case;
mod model_catalog_config;

pub use locate_model_use_case::locate_model;
pub use model_catalog_config::{model_named, ModelSpec, MODELS, NETWORK_SCALE};
