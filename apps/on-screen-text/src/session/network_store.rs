use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use loomasplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;
use ort::environment::Environment;
use ort::session::Session;

/// A loaded network and what it really runs on.
struct Loaded {
    session: Session,
    report: DeviceReport,
}

/// Every network loaded so far, by file and device (a call that asks for another device loads it
/// again).
fn store() -> MutexGuard<'static, HashMap<String, Loaded>> {
    static NETWORKS: OnceLock<Mutex<HashMap<String, Loaded>>> = OnceLock::new();
    NETWORKS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

/// Runs `work` on the network in `file` on `device`, loading it first if need be. Gives what
/// `work` gave and what the network really runs on.
pub fn with_network<T>(
    file: &Path,
    device: DeviceChoice,
    work: impl FnOnce(&mut Session) -> Result<T, Failure>,
) -> Result<(T, DeviceReport), Failure> {
    let key = format!("{}|{device:?}", file.display());
    let mut networks = store();
    if !networks.contains_key(&key) {
        let (session, report) = open_session(environment()?, device, file).map_err(|error| {
            Failure::status(
                422,
                format!(
                    "the network cannot be loaded from {}: {error}",
                    file.display()
                ),
            )
        })?;
        networks.insert(key.clone(), Loaded { session, report });
    }
    let loaded = networks
        .get_mut(&key)
        .ok_or_else(|| Failure::status(500, "the network is not loaded"))?;
    let report = loaded.report.clone();
    work(&mut loaded.session).map(|done| (done, report))
}
