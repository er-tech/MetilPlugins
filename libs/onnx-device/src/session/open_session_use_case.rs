use std::path::Path;

use ort::environment::Environment;
use ort::session::Session;

use crate::adapters::{choose_adapter, list_adapters, Adapter};
use crate::device::{DeviceChoice, DeviceReport};

/// Loads the model `file` into a session on the device `choice` asks for, and says what it runs
/// on. A GPU that cannot be used is a fall back to the CPU, with the reason in the report; only a
/// model that cannot be loaded at all is an error.
///
/// `auto` takes a discrete GPU when there is one and the CPU otherwise; `gpu` takes the best
/// adapter even if it is integrated; `gpu:N` takes that adapter.
pub fn open_session(
    environment: &Environment,
    choice: DeviceChoice,
    file: &Path,
) -> Result<(Session, DeviceReport), String> {
    let on_cpu = |fallback: Option<String>| -> Result<(Session, DeviceReport), String> {
        let session = Session::builder(environment)
            .and_then(|mut builder| builder.commit_from_file(file))
            .map_err(|error| error.to_string())?;
        Ok((session, DeviceReport::cpu(fallback)))
    };
    if choice == DeviceChoice::Cpu {
        return on_cpu(None);
    }
    let adapters = list_adapters();
    let wanted = match choice {
        DeviceChoice::Gpu(Some(number)) => match adapters.iter().find(|a| a.index == number) {
            Some(adapter) if !adapter.software => Ok(adapter),
            Some(adapter) => Err(format!("adapter {number} is {}, not a GPU", adapter.name)),
            None => Err(format!(
                "there is no adapter {number} (this computer has {})",
                adapters.len()
            )),
        },
        DeviceChoice::Gpu(None) => choose_adapter(&adapters, false)
            .ok_or_else(|| "no GPU was found on this computer".to_string()),
        _ => choose_adapter(&adapters, true)
            .ok_or_else(|| "no discrete GPU was found, so the CPU is used".to_string()),
    };
    let adapter = match wanted {
        Ok(adapter) => adapter,
        Err(reason) => return on_cpu(Some(reason)),
    };
    match gpu_session(environment, adapter, file) {
        Ok(session) => Ok((
            session,
            DeviceReport {
                device: format!("directml:{}", adapter.index),
                name: Some(adapter.name.clone()),
                fallback: None,
            },
        )),
        Err(reason) => on_cpu(Some(format!(
            "{} could not be used: {reason}",
            adapter.name
        ))),
    }
}

#[cfg(windows)]
fn gpu_session(
    environment: &Environment,
    adapter: &Adapter,
    file: &Path,
) -> Result<Session, String> {
    use ort::ep;

    let index = i32::try_from(adapter.index).map_err(|error| error.to_string())?;
    let builder = Session::builder(environment).map_err(|error| error.to_string())?;
    let mut builder = builder
        .with_execution_providers([ep::DirectML::default()
            .with_device_id(index)
            .build()
            .error_on_failure()])
        .map_err(|error| error.to_string())?;
    builder
        .commit_from_file(file)
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn gpu_session(_: &Environment, _: &Adapter, _: &Path) -> Result<Session, String> {
    Err("there is no GPU provider for this platform yet (only DirectML on Windows)".to_string())
}
