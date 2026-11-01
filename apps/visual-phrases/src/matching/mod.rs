//! Comparing a phrase with a picture: scaling their numbers and turning how alike they are into a
//! chance.

mod match_probability_algorithm;
mod normalize_algorithm;

pub use match_probability_algorithm::match_probability;
pub use normalize_algorithm::normalize;
