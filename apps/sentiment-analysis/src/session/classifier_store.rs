use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use loomasplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;
use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;

use crate::reading::Batch;

/// A loaded model, the file and device it was loaded for, and what it really runs on.
struct Loaded {
    file: PathBuf,
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The model, loaded the first time it is asked for and kept until the plugin stops (loading takes
/// far longer than reading). It is loaded again when a call asks for another file or device.
fn slot() -> MutexGuard<'static, Option<Loaded>> {
    static MODEL: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();
    MODEL
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the model really runs on, once it is loaded.
pub fn device_report() -> Option<DeviceReport> {
    slot().as_ref().map(|loaded| loaded.report.clone())
}

fn environment() -> Result<&'static Environment, Failure> {
    static ENVIRONMENT: OnceLock<Result<Environment, String>> = OnceLock::new();
    ENVIRONMENT
        .get_or_init(|| {
            ort::init()
                .with_telemetry(false)
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| Failure::status(500, format!("ONNX Runtime did not start: {message}")))
}

fn load(file: &Path, device: DeviceChoice) -> Result<Loaded, Failure> {
    let (session, report) = open_session(environment()?, device, file).map_err(|error| {
        Failure::status(
            422,
            format!(
                "the model cannot be loaded from {}: {error}",
                file.display()
            ),
        )
    })?;
    Ok(Loaded {
        file: file.to_path_buf(),
        asked: device,
        session,
        report,
    })
}

/// The three logits of each text of `batch`, one after another (positive, neutral, negative).
pub fn classify(file: &Path, device: DeviceChoice, batch: &Batch) -> Result<Vec<f32>, Failure> {
    let ids = Tensor::from_array(([batch.count, batch.length], batch.ids.clone()))
        .map_err(|error| broken("the texts could not be given", &error))?;
    let mask = Tensor::from_array(([batch.count, batch.length], batch.mask.clone()))
        .map_err(|error| broken("the texts could not be given", &error))?;
    let mut loaded = slot();
    if loaded
        .as_ref()
        .is_none_or(|current| current.file != file || current.asked != device)
    {
        *loaded = Some(load(file, device)?);
    }
    let current = loaded
        .as_mut()
        .ok_or_else(|| Failure::status(500, "the model is not loaded"))?;
    let outputs = current
        .session
        .run(ort::inputs!["input_ids" => ids, "attention_mask" => mask])
        .map_err(|error| broken("it could not read", &error))?;
    let (_, numbers) = outputs["logits"]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its answer is not numbers", &error))?;
    if numbers.len() != batch.count * 3 {
        return Err(Failure::status(
            422,
            format!(
                "the model gave {} numbers for {} texts, and {} were expected",
                numbers.len(),
                batch.count,
                batch.count * 3
            ),
        ));
    }
    Ok(numbers.to_vec())
}

fn broken(what: &str, error: &dyn std::fmt::Display) -> Failure {
    Failure::status(500, format!("the model: {what}: {error}"))
}
