//! The COCO objects plugin: finds the things a video shows, from the 80 classes of the COCO set,
//! and gives the first picture of each appearance. See README.md; the protocol is spoken by the SDK
//! in libs/plugin-sdk.

mod classes;
mod detect;
mod methods;
mod models;
mod picture;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
