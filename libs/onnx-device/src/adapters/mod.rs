//! The graphics adapters of the computer, and which one to use.

mod adapter_contract;
mod choose_adapter_policy;
mod list_adapters_client;

pub use adapter_contract::Adapter;
pub(crate) use choose_adapter_policy::choose_adapter;
pub use list_adapters_client::list_adapters;
