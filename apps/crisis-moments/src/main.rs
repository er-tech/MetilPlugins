//! The crisis-moments plugin: finds moments in a video that a person should look at, from the
//! sentences said, the sounds heard and how loud it is. It uses fixed rules, no model and no
//! network. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod methods;
mod moments;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(|action, params, _cancel| methods::run_method(action, params));
}
