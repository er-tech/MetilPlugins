//! The sentiment model, loaded once and run on a batch.

mod classifier_store;

pub use classifier_store::{classify, device_report};
