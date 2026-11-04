//! A reference Loomas plugin. It pretends to find things in a video, so recipes and hosts can be
//! tried and tested without a model. See docs/plugin-protocol.md for the protocol it speaks;
//! the SDK in libs/plugin-sdk speaks it, and this program only writes its actions.

mod methods;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(|action, params, cancel| {
        methods::run_method(action, params, &|| cancel.is_cancelled())
    });
}
