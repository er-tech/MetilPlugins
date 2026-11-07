//! Cutting and scaling pictures for the plugins that read them (`rgb24`: three bytes a pixel, row
//! after row, as Loomas decodes them). Pure functions with no dependency: a plugin that finds
//! people or text needs to cut a box out of a picture and bring it to the size a network takes.

mod scaling;

pub use scaling::{crop_to, resize_bilinear};
