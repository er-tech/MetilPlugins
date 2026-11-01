//! The sound-clean plugin: takes steady noise out of a sound and hands the cleaned sound to the
//! next plugin. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod denoise;
mod methods;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
