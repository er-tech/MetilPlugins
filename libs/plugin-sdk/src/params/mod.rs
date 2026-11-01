//! Reading the params of an action: a value in a range, or a default when it is not given, and a
//! clear 400 when it is wrong.

mod read_param_algorithm;

pub use read_param_algorithm::{flag, number, text, whole_number};
