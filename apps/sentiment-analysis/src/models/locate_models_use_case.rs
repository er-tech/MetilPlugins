use std::path::PathBuf;

use loomasplugins_model_cache::locate_model;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{tokenizer_file, ModelSpec};

/// Where the two files of a model are on this computer.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub model: PathBuf,
    pub tokenizer: PathBuf,
}

/// The files of `spec`, each downloaded first when it is not on the computer yet (see the model
/// cache).
pub fn locate_models(spec: &ModelSpec, cancel: &Cancel) -> Result<ModelPaths, Failure> {
    let get = |file| {
        locate_model("sentiment-analysis", file, |message| {
            cancel.progress(message)
        })
        .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
    };
    Ok(ModelPaths {
        model: get(&spec.file)?,
        tokenizer: get(tokenizer_file())?,
    })
}
