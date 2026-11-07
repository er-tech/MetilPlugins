//! The sentiment analysis plugin: how positive or negative what is said is, sentence by sentence,
//! with a multilingual model that runs on the computer. See README.md; the protocol is spoken by
//! the SDK in libs/plugin-sdk.

mod methods;
mod models;
mod reading;
mod scoring;
mod session;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
