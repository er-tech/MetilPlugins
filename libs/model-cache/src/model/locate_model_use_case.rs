use std::path::PathBuf;

use std::time::Duration;

use super::download_model_use_case::download_model;
use super::model_error::ModelError;
use super::model_file_contract::ModelFile;
use super::models_folder_policy::models_folder;

/// The file of `model` for the plugin called `plugin` (its own folder among the models), downloading
/// it first when it is not there yet (once: the next call finds it). A file of the wrong size is
/// taken for a broken download and fetched again. `progress` is told when a download starts.
pub fn locate_model(
    plugin: &str,
    model: &ModelFile,
    progress: impl Fn(&str),
) -> Result<PathBuf, ModelError> {
    let folder = models_folder(|name| std::env::var(name).ok()).ok_or_else(|| ModelError {
        code: 422,
        message: "no place to keep the model: set LOOMAS_MODELS to a folder".to_string(),
        retryable: false,
    })?;
    let file = folder.join(plugin).join(model.file);
    if std::fs::metadata(&file).is_ok_and(|meta| meta.len() == model.bytes) {
        return Ok(file);
    }
    progress(&format!("downloading {}", model.file));
    download_model(model, &file, &progress, Duration::from_secs(1)).map_err(|message| {
        ModelError {
            code: 503,
            message: format!("{} could not be downloaded: {message}", model.file),
            retryable: true,
        }
    })?;
    Ok(file)
}
