//! Keeps the model files a plugin needs: a plugin does not package its models, it names them (where
//! to download each from and its sha256) and this finds the file on the computer or downloads it
//! once, checking its size and sum, into `~/.loomas/models/<plugin>` (or the folder the variable
//! `LOOMAS_MODELS` names).

mod model;

pub use model::{locate_model, ModelError, ModelFile};
