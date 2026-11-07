//! The networks the plugin uses and where their files live on the computer.

mod locate_models_use_case;
mod model_catalog_config;

pub use locate_models_use_case::{locate_models, ModelPaths};
pub use model_catalog_config::{
    detector_named, language_named, Detector, Language, DETECTORS, LANGUAGES,
};
