//! Turning what the model answers into labels, scores and an overall value.

mod sentiment_scores_algorithm;

pub use sentiment_scores_algorithm::{overall, score_logits, Scored};
