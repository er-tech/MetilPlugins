//! The networks, loaded once each and kept: loading takes far longer than looking.

mod network_store;

pub use network_store::with_network;
