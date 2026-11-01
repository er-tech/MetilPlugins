//! The protocol over standard input and output: one JSON message per line in, one per line out.

mod cancel_token_model;
mod failure_error;
mod serve_lines_handler;

pub use cancel_token_model::Cancel;
pub use failure_error::Failure;
pub use serve_lines_handler::{serve_lines, serve_stdio};
