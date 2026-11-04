use std::path::PathBuf;

use loomasplugins_model_cache::locate_model;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{TEXT, TOKENIZER, VISION};

/// Where the three files of the model are on this computer.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub vision: PathBuf,
    pub text: PathBuf,
    pub tokenizer: PathBuf,
}

/// The files of the model, each downloaded first when it is not on the computer yet (see the
/// model cache).
pub fn locate_models(cancel: &Cancel) -> Result<ModelPaths, Failure> {
    let get = |file| {
        locate_model("visual-phrases", file, |message| cancel.progress(message))
            .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
    };
    Ok(ModelPaths {
        vision: get(&VISION)?,
        text: get(&TEXT)?,
        tokenizer: get(&TOKENIZER)?,
    })
}
