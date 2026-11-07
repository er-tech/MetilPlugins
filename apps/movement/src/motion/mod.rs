//! Turning the keypoints of people over time into movements, written as what is seen and never as
//! what it means.

mod hints_algorithm;
mod join_hints_algorithm;
mod torso_algorithm;
mod vocabulary_config;

pub use hints_algorithm::{find_hints, MotionSettings};
pub use join_hints_algorithm::join_hints;
