use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use metilplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use metilplugins_plugin_sdk::Failure;

/// How many numbers a picture or a phrase becomes.
pub const DIMENSIONS: usize = 768;

/// A loaded model, the device it was asked to run on and what it really runs on.
struct Loaded {
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The image model and the text model, loaded the first time each is asked for and kept until the
/// plugin stops: loading takes far longer than looking. The image model is loaded again when a call
/// asks for another device.
fn slot(vision: bool) -> MutexGuard<'static, Option<Loaded>> {
    static VISION: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();
    static TEXT: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();
    let cell = if vision { &VISION } else { &TEXT };
    cell.get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the image model really runs on, once it is loaded.
pub fn vision_device_report() -> Option<DeviceReport> {
    slot(true).as_ref().map(|loaded| loaded.report.clone())
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
        asked: device,
        session,
        report,
    })
}

/// The 768 numbers of each of `count` pictures, one after another. `values` holds the pictures
/// as `prepare_picture` makes them, one after another.
pub fn embed_pictures(
    file: &Path,
    device: DeviceChoice,
    values: Vec<f32>,
    count: usize,
) -> Result<Vec<f32>, Failure> {
    let side = crate::pictures::SIDE;
    let input = Tensor::from_array(([count, 3, side, side], values))
        .map_err(|error| broken("the pictures could not be given", &error))?;
    with_session(true, file, device, |session| {
        let outputs = session
            .run(ort::inputs!["pixel_values" => input])
            .map_err(|error| broken("it could not look", &error))?;
        numbers_of(&outputs, count)
    })
}

/// The 768 numbers of each of `count` phrases, one after another. `ids` holds the pieces of each
/// phrase, `length` of them, padded.
pub fn embed_texts(
    file: &Path,
    ids: Vec<i64>,
    count: usize,
    length: usize,
) -> Result<Vec<f32>, Failure> {
    let input = Tensor::from_array(([count, length], ids))
        .map_err(|error| broken("the phrases could not be given", &error))?;
    with_session(false, file, DeviceChoice::Cpu, |session| {
        let outputs = session
            .run(ort::inputs!["input_ids" => input])
            .map_err(|error| broken("it could not read", &error))?;
        numbers_of(&outputs, count)
    })
}

/// Runs `work` on the image model (`vision`) or the text model, loading it first if need be. The
/// text model is small and always runs on the CPU.
fn with_session(
    vision: bool,
    file: &Path,
    device: DeviceChoice,
    work: impl FnOnce(&mut Session) -> Result<Vec<f32>, Failure>,
) -> Result<Vec<f32>, Failure> {
    let mut loaded = slot(vision);
    if loaded
        .as_ref()
        .is_none_or(|current| current.asked != device)
    {
        *loaded = Some(load(file, device)?);
    }
    let current = loaded
        .as_mut()
        .ok_or_else(|| Failure::status(500, "the model is not loaded"))?;
    work(&mut current.session)
}

/// What the model answered for `count` inputs: its `pooler_output`, `DIMENSIONS` numbers each.
fn numbers_of(outputs: &ort::session::SessionOutputs, count: usize) -> Result<Vec<f32>, Failure> {
    let (_, numbers) = outputs["pooler_output"]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its answer is not numbers", &error))?;
    if numbers.len() != count * DIMENSIONS {
        return Err(Failure::status(
            422,
            format!(
                "the model gave {} numbers for {count} inputs, and {} were expected",
                numbers.len(),
                count * DIMENSIONS
            ),
        ));
    }
    Ok(numbers.to_vec())
}

fn broken(what: &str, error: &dyn std::fmt::Display) -> Failure {
    Failure::status(500, format!("the model: {what}: {error}"))
}
