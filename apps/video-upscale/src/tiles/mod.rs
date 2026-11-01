//! Cutting a picture into overlapping square pieces for a network that sees one size only, and
//! putting the answers back together without seams.

mod cut_tile_algorithm;
mod paste_tile_algorithm;
mod plan_tiles_algorithm;

pub use cut_tile_algorithm::cut_tile;
pub use paste_tile_algorithm::{paste_tile, Placement};
pub use plan_tiles_algorithm::{tiles_along, TILE_MARGIN};
