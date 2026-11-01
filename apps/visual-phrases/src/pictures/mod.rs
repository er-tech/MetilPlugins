//! Pictures for the image model: fitting a frame of the video to what it takes, and getting the
//! numbers that stand for it.

mod embed_pictures_use_case;
mod prepare_picture_algorithm;
mod resize_bicubic_algorithm;

pub use embed_pictures_use_case::{embed_all_pictures, Picture};
pub use prepare_picture_algorithm::SIDE;
