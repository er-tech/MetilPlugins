//! Writing the new video: the pictures go to an FFmpeg process that scales them to their final
//! size and encodes them.

mod encoder_client;
mod encoder_settings_contract;
mod ffmpeg_arguments_algorithm;
mod ffmpeg_program_algorithm;
mod frame_rate_algorithm;

pub use encoder_client::Encoder;
pub use encoder_settings_contract::EncoderSettings;
pub use ffmpeg_program_algorithm::ffmpeg_program;
pub use frame_rate_algorithm::frame_rate;
