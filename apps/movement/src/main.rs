//! The movement plugin: the movements of the people in a video as a list, found from the keypoints
//! of their bodies. It sees bodies and never faces or names, and it says what moved, never what it
//! means. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod imaging;
mod methods;
mod models;
mod motion;
mod people;
mod pose;
mod session;
mod tracking;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
