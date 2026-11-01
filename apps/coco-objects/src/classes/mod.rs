//! The classes the detector knows: the 80 of the COCO set.

mod coco_classes_config;
mod select_classes_validator;

pub use coco_classes_config::COCO_CLASSES;
pub use select_classes_validator::select_classes;
