//! The constants the network was trained with: the bands, the window and the sizes.

mod read_tables_algorithm;
mod tables_contract;

pub use read_tables_algorithm::read_tables;
pub use tables_contract::{
    Tables, BANDS, BINS, FFT_SIZE, FILTER_BINS, FILTER_LOOKAHEAD, FILTER_ORDER, HOP,
};
