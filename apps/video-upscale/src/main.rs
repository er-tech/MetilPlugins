//! The video upscale plugin: makes the pictures of a video larger with a Real-ESRGAN network (or
//! plain Lanczos scaling) and gives the new video. See README.md; the protocol is spoken by the SDK
//! in libs/plugin-sdk.

mod encode;
mod methods;
mod models;
mod tiles;
mod upscale;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
