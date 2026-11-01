//! The image model and the text model, loaded once and run on a batch.

mod siglip_store;

pub use siglip_store::{embed_pictures, embed_texts, vision_device_report, DIMENSIONS};
