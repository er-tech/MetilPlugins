use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use metilplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use metilplugins_plugin_sdk::Failure;

use crate::spectrum::{NetworkAnswer, NetworkFeatures};
use crate::tables::{BANDS, FILTER_BINS, FILTER_ORDER};

/// The loaded network, the device it was asked to run on and what it really runs on.
struct Loaded {
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The loaded network, kept until the plugin stops: loading takes far longer than a piece of sound.
/// It is loaded again when a call asks for another device.
fn session() -> MutexGuard<'static, Option<Loaded>> {
    static SESSION: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();
    SESSION
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the network really runs on, once it is loaded.
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

/// Gives the network the features of a sound of `frames` frames and returns its answer. `file` is
/// the network, loaded on the first call (or when `device` is not the one it was loaded for).
pub fn run_network(
    file: &Path,
    device: DeviceChoice,
    features: &NetworkFeatures,
    frames: usize,
) -> Result<NetworkAnswer, Failure> {
    let broken = |what: &str, error: &dyn std::fmt::Display| {
        Failure::status(500, format!("the voice model: {what}: {error}"))
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
                    "the voice model cannot be loaded from {}: {error}",
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
        .ok_or_else(|| Failure::status(500, "the voice model is not loaded"))?
        .session;

    let bands = Tensor::from_array(([1usize, 1, frames, BANDS], features.bands.clone()))
        .map_err(|error| broken("the features could not be given", &error))?;
    let spectrum =
        Tensor::from_array(([1usize, 2, frames, FILTER_BINS], features.spectrum.clone()))
            .map_err(|error| broken("the features could not be given", &error))?;
    let outputs = session
        .run(ort::inputs!["feat_erb" => bands, "feat_spec" => spectrum])
        .map_err(|error| broken("it could not listen", &error))?;
    let (_, mask) = outputs["erb_mask"]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its mask is not numbers", &error))?;
    let (_, coefficients) = outputs["df_coefs"]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its filter is not numbers", &error))?;
    if mask.len() != frames * BANDS || coefficients.len() != FILTER_ORDER * frames * FILTER_BINS * 2
    {
        return Err(Failure::status(
            422,
            format!(
                "the voice model gave {} and {} numbers for {frames} frames, and {} and {} were expected",
                mask.len(),
                coefficients.len(),
                frames * BANDS,
                FILTER_ORDER * frames * FILTER_BINS * 2
            ),
        ));
    }
    Ok(NetworkAnswer {
        mask: mask.to_vec(),
        coefficients: coefficients.to_vec(),
    })
}
