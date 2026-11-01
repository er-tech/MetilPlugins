//! The visual phrases plugin: finds when a phrase describes what a video shows ("two cats sleeping
//! on a couch"), with an image-text model and no language model. See README.md; the protocol is
//! spoken by the SDK in libs/plugin-sdk.

mod matching;
mod methods;
mod models;
mod phrases;
mod pictures;
mod session;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
