//! Making one picture larger with a network, piece by piece, on the device the host granted.

mod upscale_frame_use_case;
mod upscaler_store;

pub use upscale_frame_use_case::upscale_frame;
pub use upscaler_store::device_report;
