//! Reading what the plugin is given and turning the texts into what the model takes.

mod read_segments_algorithm;
mod token_batches_algorithm;

pub use read_segments_algorithm::read_segments;
pub use token_batches_algorithm::{token_batches, Batch};
