use std::path::PathBuf;

use loomasplugins_model_cache::{locate_model, locate_zipped_model, ModelFile};
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{ModelSpec, Pipeline};

/// Where the files of a model are on this computer.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub detector: Option<PathBuf>,
    pub pose: PathBuf,
}

/// The files of `spec`, each downloaded first when it is not on the computer yet (see the model
/// cache). A zip is unpacked once.
pub fn locate_models(spec: &ModelSpec, cancel: &Cancel) -> Result<ModelPaths, Failure> {
    let progress = |message: &str| cancel.progress(message);
    let failed = |error: loomasplugins_model_cache::ModelError| {
        Failure::status(error.code, error.message).retryable(error.retryable)
    };
    let zipped = |file: &ModelFile| locate_zipped_model("movement", file, "end2end.onnx", progress);
    let detector = match &spec.detector {
        Some(file) => Some(zipped(file).map_err(failed)?),
        None => None,
    };
    let pose = match spec.pipeline {
        Pipeline::RtmPose => zipped(&spec.pose),
        Pipeline::MoveNet => locate_model("movement", &spec.pose, progress),
    }
    .map_err(failed)?;
    Ok(ModelPaths { detector, pose })
}
