//! Scaling and cutting with bilinear sampling.

mod crop_to_algorithm;
mod resize_bilinear_algorithm;

pub use crop_to_algorithm::crop_to;
pub use resize_bilinear_algorithm::resize_bilinear;
