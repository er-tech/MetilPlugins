//! The music-sync plugin: finds where a music starts inside a video's sound. See README.md; the
//! protocol is spoken by the SDK in libs/plugin-sdk.

mod alignment;
mod methods;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
