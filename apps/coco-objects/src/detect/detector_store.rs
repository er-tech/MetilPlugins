use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use loomasplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;

use crate::models::ModelSpec;

/// A loaded detector and what it runs on.
struct Loaded {
    session: Session,
    report: DeviceReport,
}

/// The detectors loaded in this process, by model name and device asked for, and the environment
/// they share. Loading a model takes far longer than looking at a picture, so a model is loaded the
/// first time it is asked for and kept until the plugin stops.
fn sessions() -> MutexGuard<'static, HashMap<(&'static str, DeviceChoice), Loaded>> {
    static SESSIONS: OnceLock<Mutex<HashMap<(&'static str, DeviceChoice), Loaded>>> =
        OnceLock::new();
    SESSIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the detector for `model` asked to run on `device` really runs on, once it is loaded.
pub fn device_report(model: &ModelSpec, device: DeviceChoice) -> Option<DeviceReport> {
    sessions()
        .get(&(model.name, device))
        .map(|loaded| loaded.report.clone())
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

/// Gives the detector `values` (a square picture of `model.input_size`, three planes) and returns
/// everything it answers, flat: rows of candidate boxes. `file` is the model, loaded on the first
/// call for this model.
pub fn run_detector(
    model: &ModelSpec,
    device: DeviceChoice,
    file: &Path,
    values: Vec<f32>,
) -> Result<Vec<f32>, Failure> {
    let broken = |what: &str, error: &dyn std::fmt::Display| {
        Failure::status(500, format!("the {} model: {what}: {error}", model.name))
    };
    let mut loaded = sessions();
    let entry = match loaded.entry((model.name, device)) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            let (session, report) =
                open_session(environment()?, device, file).map_err(|error| {
                    Failure::status(
                        422,
                        format!(
                            "the {} model cannot be loaded from {}: {error}",
                            model.name,
                            file.display()
                        ),
                    )
                })?;
            entry.insert(Loaded { session, report })
        }
    };
    let session = &mut entry.session;
    let input_name = session
        .inputs()
        .first()
        .map(|input| input.name().to_string())
        .ok_or_else(|| Failure::status(422, format!("the {} model has no input", model.name)))?;
    let size = model.input_size;
    let input = Tensor::from_array(([1usize, 3, size, size], values))
        .map_err(|error| broken("the picture could not be given", &error))?;
    let outputs = session
        .run(ort::inputs![input_name.as_str() => input])
        .map_err(|error| broken("it could not look at the picture", &error))?;
    let (_, answer) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its answer is not numbers", &error))?;
    Ok(answer.to_vec())
}
