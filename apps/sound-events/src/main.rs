//! The sound events plugin: finds the sounds a video has, from the 521 classes of AudioSet, and
//! gives a clip of each. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod classes;
mod events;
mod listen;
mod methods;
mod models;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
