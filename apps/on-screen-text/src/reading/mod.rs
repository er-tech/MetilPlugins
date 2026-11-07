//! Finding the text in a picture and reading it: a detector finds where the lines of text are, a
//! recognizer reads each line.

mod detect_text_algorithm;
mod read_picture_use_case;
mod recognize_text_algorithm;
mod text_box_contract;

pub use read_picture_use_case::read_picture;
pub use recognize_text_algorithm::load_characters;
