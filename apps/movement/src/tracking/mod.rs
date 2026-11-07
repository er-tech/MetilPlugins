//! Following the same person from picture to picture.

mod track_people_algorithm;

#[cfg(test)]
pub use track_people_algorithm::Sample;
pub use track_people_algorithm::{track_people, Track};
