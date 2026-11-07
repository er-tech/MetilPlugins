//! Cutting and scaling pictures (`rgb24`, three bytes a pixel, row after row).

mod crop_algorithm;
mod resize_algorithm;

pub use crop_algorithm::crop_to;
pub use resize_algorithm::resize_bilinear;
