//! The thumbnail plugin: chooses the pictures of a video that show it best, and lays out pictures
//! from along it as a contact sheet. See README.md; the
//! protocol is spoken by the SDK in libs/plugin-sdk.

mod methods;
mod pick;
mod sheet;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}
