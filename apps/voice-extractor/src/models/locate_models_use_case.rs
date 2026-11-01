use std::path::PathBuf;

use metilplugins_model_cache::locate_model;
use metilplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{NETWORK, TABLES};

/// Where the two files of the model are on this computer.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub network: PathBuf,
    pub tables: PathBuf,
}

/// The files of the model, each downloaded first when it is not on the computer yet (see the
/// model cache).
pub fn locate_models(cancel: &Cancel) -> Result<ModelPaths, Failure> {
    let get = |file| {
        locate_model("voice-extractor", file, |message| cancel.progress(message))
            .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
    };
    Ok(ModelPaths {
        network: get(&NETWORK)?,
        tables: get(&TABLES)?,
    })
}
