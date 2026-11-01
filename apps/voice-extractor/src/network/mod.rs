//! The network: the features go in, a mask and the coefficients of a filter come out.

mod network_store;

pub use network_store::{device_report, run_network};
