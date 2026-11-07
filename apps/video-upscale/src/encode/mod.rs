//! Writing the new video: the pictures go to an FFmpeg process that scales them to their final
//! size and encodes them.

mod codec_probe_client;
mod encoder_choice_contract;
mod encoder_client;
mod encoder_settings_contract;
mod ffmpeg_arguments_algorithm;
mod ffmpeg_program_algorithm;
mod frame_rate_algorithm;
mod hardware_codecs_config;
mod pick_codec_use_case;
mod video_codec_contract;

pub use codec_probe_client::codec_works;
pub use encoder_choice_contract::EncoderChoice;
pub use encoder_client::Encoder;
pub use encoder_settings_contract::EncoderSettings;
pub use ffmpeg_program_algorithm::ffmpeg_program;
pub use frame_rate_algorithm::frame_rate;
pub use hardware_codecs_config::hardware_codecs;
pub use pick_codec_use_case::pick_codec;
