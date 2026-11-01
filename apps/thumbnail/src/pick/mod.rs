//! Scoring the pictures of a video and choosing the ones that show it best.

mod candidate_contract;
mod choose_picks_algorithm;
mod score_picture_algorithm;

pub use candidate_contract::Candidate;
pub use choose_picks_algorithm::choose_picks;
pub use score_picture_algorithm::score_picture;
