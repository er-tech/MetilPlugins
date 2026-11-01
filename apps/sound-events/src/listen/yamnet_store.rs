use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use metilplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use metilplugins_plugin_sdk::Failure;

/// How many classes the model scores in each frame.
pub const CLASSES: usize = 521;

/// The loaded model, the device it was asked to run on and what it really runs on.
struct Loaded {
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The loaded model, kept until the plugin stops: loading takes far longer than listening. It is
/// loaded again when a call asks for another device.
fn session() -> MutexGuard<'static, Option<Loaded>> {
    static SESSION: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();
    SESSION
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the model really runs on, once it is loaded.
pub fn device_report() -> Option<DeviceReport> {
    session().as_ref().map(|loaded| loaded.report.clone())
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

/// Gives the model `samples` (mono, 16 kHz, from -1 to 1) and returns the scores of every frame it
/// found, one frame after another, `CLASSES` values each. `file` is the model, loaded on the first
/// call (or when `device` is not the one it was loaded for).
pub fn run_yamnet(file: &Path, device: DeviceChoice, samples: &[f32]) -> Result<Vec<f32>, Failure> {
    let broken = |what: &str, error: &dyn std::fmt::Display| {
        Failure::status(500, format!("the sound model: {what}: {error}"))
    };
    let mut loaded = session();
    if loaded
        .as_ref()
        .is_none_or(|current| current.asked != device)
    {
        let (session, report) = open_session(environment()?, device, file).map_err(|error| {
            Failure::status(
                422,
                format!(
                    "the sound model cannot be loaded from {}: {error}",
                    file.display()
                ),
            )
        })?;
        *loaded = Some(Loaded {
            asked: device,
            session,
            report,
        });
    }
    let session = &mut loaded
        .as_mut()
        .ok_or_else(|| Failure::status(500, "the sound model is not loaded"))?
        .session;
    let input_name = session
        .inputs()
        .first()
        .map(|input| input.name().to_string())
        .ok_or_else(|| Failure::status(422, "the sound model has no input"))?;
    let input = Tensor::from_array(([samples.len()], samples.to_vec()))
        .map_err(|error| broken("the sound could not be given", &error))?;
    let outputs = session
        .run(ort::inputs![input_name.as_str() => input])
        .map_err(|error| broken("it could not listen", &error))?;
    let (_, scores) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its answer is not numbers", &error))?;
    if scores.len() % CLASSES != 0 {
        return Err(Failure::status(
            422,
            format!(
                "the sound model gave {} values, not a whole number of frames of {CLASSES}",
                scores.len()
            ),
        ));
    }
    Ok(scores.to_vec())
}
