//! The video-audio plugin: the sound of a video as an audio file, in the format, rate and loudness
//! asked for. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk, and the audio is
//! written by the user's own FFmpeg.

mod ffmpeg;
mod methods;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
