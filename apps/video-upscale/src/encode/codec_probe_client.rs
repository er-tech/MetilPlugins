use std::path::Path;
use std::process::{Command, Stdio};

use super::video_codec_contract::VideoCodec;

/// Tells whether `program` (FFmpeg) can really encode with `codec` here: a driver, a card and a
/// build of FFmpeg that all agree. Listing the encoders is not enough, because FFmpeg lists the
/// ones it was built with whether or not the computer has the card, so a short test encode of a
/// few black pictures decides.
pub fn codec_works(program: &Path, codec: &VideoCodec) -> bool {
    Command::new(program)
        .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i"])
        .arg("color=c=black:s=256x256:r=10:d=0.5")
        .args(["-pix_fmt", "yuv420p"])
        .args(codec.to_arguments())
        .args(["-f", "null", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
