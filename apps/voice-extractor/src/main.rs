//! The voice extractor plugin: splits the sound of a video into the voice and the rest, with
//! DeepFilterNet3. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod enhance;
mod methods;
mod models;
mod network;
mod spectrum;
mod tables;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
