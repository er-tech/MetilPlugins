//! Joining what the pictures say into what the video shows: the same text in the same place in many
//! pictures is one appearance.

mod merge_appearances_algorithm;
mod similarity_algorithm;

pub use merge_appearances_algorithm::{merge_appearances, ReadLine, Reading};
