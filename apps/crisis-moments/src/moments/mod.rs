//! The rules that turn sentences, sounds and loudness into moments to review.

mod label_signals_algorithm;
mod merge_signals_algorithm;
mod signal_contract;
mod text_signals_algorithm;
mod timed_item_contract;
mod volume_signals_algorithm;

pub use label_signals_algorithm::label_signals;
pub use merge_signals_algorithm::merge_signals;
pub use signal_contract::Moment;
pub use text_signals_algorithm::text_signals;
pub use timed_item_contract::TimedItem;
pub use volume_signals_algorithm::volume_signals;
