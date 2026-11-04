//! The seed of the Rust SDK for Loomas plugins: it speaks the protocol (see
//! docs/plugin-protocol.md) so a plugin only writes its actions.
//!
//! ```no_run
//! use loomasplugins_plugin_sdk::{serve_stdio, Failure};
//!
//! serve_stdio(|action, params, cancel| match action {
//!     "echo" => Ok(params.clone()),
//!     other => Err(Failure::status(404, format!("no action {other:?}"))),
//! });
//! ```
//!
//! What it does: answers `hello`, runs every call on its own thread (so a plugin that declares a
//! `concurrency` above 1 takes that many calls at once), tells a call that the host cancelled it,
//! turns a returned [`Failure`] into an `error` message with its code, retry hint and details, turns
//! a panic into a 500, and stops at `shutdown`. `whole_number`, `number`, `flag` and `text` read a
//! param with a default and a range, and fail with a 400 that says what was wrong. What the plugin decides stays the plugin's: the SDK
//! never reads a result to find out whether it failed.

mod params;
mod serve;

pub use params::{flag, number, text, whole_number};
pub use serve::{serve_lines, serve_stdio, Cancel, Failure};
