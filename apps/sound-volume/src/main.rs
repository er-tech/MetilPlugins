//! The sound-volume plugin: how loud a sound is over time, split into low, mid and high bands, and
//! where it goes quiet. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod methods;
mod volume;

use metilplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(|action, params, _cancel| methods::run_method(action, params));
}
