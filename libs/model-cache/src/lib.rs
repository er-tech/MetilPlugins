//! Keeps the model files a plugin needs: a plugin does not package its models, it names them (where
//! to download each from and its sha256) and this finds the file on the computer or downloads it
//! once, checking its size and sum, into `~/.loomas/models/<plugin>` (or the folder the variable
//! `LOOMAS_MODELS` names). A model that comes in a zip is downloaded as the zip, checked, and the file
//! it needs is taken out of it once.

mod model;

pub use model::{locate_model, locate_zipped_model, ModelError, ModelFile};
