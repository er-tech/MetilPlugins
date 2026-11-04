//! Reads the pictures Loomas decoded for a plugin. A plugin that lists a `wants` input with
//! `"media": "video"` in its manifest gets, in place of the file, the description of a file of raw
//! pixels and of a list of the pictures (their time, duration and whether they are key frames);
//! `read_video_region` maps the file, checks it against the description and gives the pictures one
//! by one without copying them.

mod region;

pub use region::{read_video_region, Frame, FrameError, PixelFormat, VideoRegion};
