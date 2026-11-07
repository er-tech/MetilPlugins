//! The on-screen text plugin: the text a video shows (titles, subtitles burnt into the picture,
//! signs, captions), read with PaddleOCR networks that run on the computer. See README.md; the
//! protocol is spoken by the SDK in libs/plugin-sdk.

mod appearances;
mod methods;
mod models;
mod reading;
mod session;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
