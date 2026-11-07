use std::path::PathBuf;

use loomasplugins_model_cache::locate_model;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{Detector, Language};

/// Where the files of the networks are on this computer.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub detector: PathBuf,
    pub recognizer: PathBuf,
    pub dictionary: PathBuf,
}

/// The files for reading `language` with `detector`, each downloaded first when it is not on the
/// computer yet (see the model cache).
pub fn locate_models(
    detector: &Detector,
    language: &Language,
    cancel: &Cancel,
) -> Result<ModelPaths, Failure> {
    let get = |file| {
        locate_model("on-screen-text", file, |message| cancel.progress(message))
            .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
    };
    Ok(ModelPaths {
        detector: get(&detector.file)?,
        recognizer: get(&language.recognizer)?,
        dictionary: get(&language.dictionary)?,
    })
}
