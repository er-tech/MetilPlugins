//! Opens an ONNX Runtime session on the CPU or on a GPU, for the plugins that run models. A plugin
//! gives it the `device` its user asked for (`auto`, `cpu`, `gpu` or `gpu:N`) and gets back the
//! session and a report of what really ran, because a GPU that cannot be used (no provider on this
//! platform, a model it cannot run) is a fall back to the CPU and not a failure.
//!
//! On Windows the GPU is DirectML, which runs on any DirectX 12 GPU (AMD, NVIDIA, Intel). Adapters
//! are listed and ranked here because DirectML's default adapter may be a slow integrated one.
//! Research and measurements: `docs/research/gpu.md` in the Metil repository.

mod adapters;
mod device;
mod session;

pub use adapters::{list_adapters, Adapter};
pub use device::{DeviceChoice, DeviceReport};
pub use session::open_session;
