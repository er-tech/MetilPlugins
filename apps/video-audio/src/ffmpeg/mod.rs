//! Running FFmpeg to write audio: the arguments it is given, the formats, the program, and reading
//! the loudness it measures.

mod audio_arguments_algorithm;
mod audio_format_contract;
mod ffmpeg_program_algorithm;
mod loudness_algorithm;
mod run_ffmpeg_client;

pub use audio_arguments_algorithm::{
    extract_arguments, measure_arguments, AudioInput, AudioOutput,
};
pub use audio_format_contract::AudioFormat;
pub use ffmpeg_program_algorithm::ffmpeg_program;
pub use loudness_algorithm::{loudnorm_filter, parse_loudness};
pub use run_ffmpeg_client::run_ffmpeg;
