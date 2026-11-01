//! Appearances: joining what a plugin found picture by picture (or frame by frame) into stretches.
//! For each thing it looks for (a class, a phrase), the stretch from the first sample it is in to
//! the last of the same run; when it leaves for longer than a gap and comes back, that is another.

mod track;

pub use track::{track_appearances, Appearance, Presence, Sample};
