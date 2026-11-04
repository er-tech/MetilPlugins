//! The sound-graph plugin: draws the waves of a sound as an SVG picture. See README.md; the
//! protocol is spoken by the SDK in libs/plugin-sdk.

mod methods;
mod waveform;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(|action, params, _cancel| methods::run_method(action, params));
}
