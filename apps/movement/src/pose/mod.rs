//! Finding the people in a picture and the keypoints of their bodies, with the pipeline a model
//! uses.

mod detect_people_use_case;
mod estimate_rtmpose_use_case;
mod find_people_use_case;
mod movenet_algorithm;
mod rtmpose_algorithm;
mod yolox_algorithm;

pub use find_people_use_case::{find_people, FindSettings};
