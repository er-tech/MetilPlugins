//! The thumbnail plugin: chooses the pictures of a video that show it best. See README.md; the
//! protocol is spoken by the SDK in libs/plugin-sdk.

mod methods;
mod pick;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
