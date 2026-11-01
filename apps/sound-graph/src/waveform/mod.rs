//! Drawing the waves of a sound: the lowest and highest samples under each column, the regions to
//! mark, a time axis, and the SVG that puts it together.

mod compute_columns_algorithm;
mod draw_waveform_svg_algorithm;
mod escape_xml_algorithm;
mod region_contract;
mod time_ticks_algorithm;

pub use compute_columns_algorithm::compute_columns;
pub use draw_waveform_svg_algorithm::{draw_waveform_svg, SvgOptions};
pub use region_contract::read_regions;
