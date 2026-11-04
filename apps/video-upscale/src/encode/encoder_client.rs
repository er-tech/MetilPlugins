use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use loomasplugins_plugin_sdk::Failure;

use super::encoder_settings_contract::EncoderSettings;
use super::ffmpeg_arguments_algorithm::ffmpeg_arguments;

/// A running FFmpeg that turns the pictures written to it into a video file.
pub struct Encoder {
    child: Child,
    input: ChildStdin,
}

impl Encoder {
    /// Starts `program` to write `output`. A program that is not there is a 422 that says how to
    /// point at it.
    pub fn start(
        program: &Path,
        settings: &EncoderSettings,
        output: &Path,
    ) -> Result<Encoder, Failure> {
        let mut child = Command::new(program)
            .args(ffmpeg_arguments(settings, output))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                Failure::status(
                    422,
                    format!(
                        "FFmpeg could not be started from {}: {error}. Install it or set LOOMAS_FFMPEG",
                        program.display()
                    ),
                )
            })?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| Failure::status(500, "FFmpeg gave no input to write pictures to"))?;
        Ok(Encoder { child, input })
    }

    /// Gives the encoder one `rgb24` picture of the size the settings say.
    pub fn write_frame(&mut self, rgb: &[u8]) -> Result<(), Failure> {
        self.input.write_all(rgb).map_err(|error| {
            Failure::status(
                422,
                format!("FFmpeg stopped taking pictures: {error} (see finish for why)"),
            )
        })
    }

    /// Ends the stream and waits for the file. A 422 with what FFmpeg said when it failed.
    pub fn finish(self) -> Result<(), Failure> {
        let Encoder { child, input } = self;
        drop(input);
        let done = child
            .wait_with_output()
            .map_err(|error| Failure::status(500, format!("FFmpeg did not finish: {error}")))?;
        if done.status.success() {
            return Ok(());
        }
        Err(Failure::status(
            422,
            format!(
                "FFmpeg failed: {}",
                String::from_utf8_lossy(&done.stderr).trim()
            ),
        ))
    }

    /// Stops the encoder and leaves the file as it is (the caller removes it).
    pub fn abort(self) {
        let Encoder { mut child, input } = self;
        drop(input);
        let _ = child.kill();
        let _ = child.wait();
    }
}
