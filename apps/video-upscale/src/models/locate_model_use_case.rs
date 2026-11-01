use std::path::PathBuf;

use metilplugins_model_cache::locate_model as locate_file;
use metilplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::ModelSpec;

/// The file of `model`, downloaded first when it is not on the computer yet (see the model cache).
/// `None` for a model that has no file (the Lanczos baseline).
pub fn locate_model(model: &ModelSpec, cancel: &Cancel) -> Result<Option<PathBuf>, Failure> {
    let Some(file) = &model.file else {
        return Ok(None);
    };
    locate_file("video-upscale", file, |message| cancel.progress(message))
        .map(Some)
        .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
}
