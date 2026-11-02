use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use metilplugins_plugin_sdk::{Cancel, Failure};

/// How often a running FFmpeg is looked at to see if the call was cancelled.
const POLL: Duration = Duration::from_millis(50);

/// How many characters of FFmpeg's output go into a failure message (the end, where the error is).
const KEPT: usize = 600;

/// Runs `program` with `arguments` to the end and gives what it printed on its error output (the
/// place FFmpeg writes its log and what the loudness filter finds). A program that is not there is
/// a 422 that says how to point at it; a program that fails is a 422 with the end of what it said;
/// a cancelled call stops the program and gives `CANCELLED`.
pub fn run_ffmpeg(
    program: &Path,
    arguments: &[String],
    cancel: &Cancel,
) -> Result<String, Failure> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Failure::status(
                422,
                format!(
                    "FFmpeg could not be started from {}: {error}. Install it or set METIL_FFMPEG",
                    program.display()
                ),
            )
        })?;
    let mut log = child
        .stderr
        .take()
        .ok_or_else(|| Failure::status(500, "FFmpeg gave no output to read"))?;
    // The log is read as it comes, so a long file cannot fill the pipe and stop FFmpeg.
    let reader = std::thread::spawn(move || {
        let mut text = Vec::new();
        let _ = log.read_to_end(&mut text);
        String::from_utf8_lossy(&text).into_owned()
    });
    let status = loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                cancel.progress("FFmpeg is working");
                std::thread::sleep(POLL);
            }
            Err(error) => {
                return Err(Failure::status(
                    500,
                    format!("FFmpeg could not be waited for: {error}"),
                ))
            }
        }
    };
    let text = reader.join().unwrap_or_default();
    if status.success() {
        Ok(text)
    } else {
        Err(Failure::status(
            422,
            format!("FFmpeg failed: {}", tail(&text, KEPT)),
        ))
    }
}

/// The last `keep` characters of `text`, with the blank edges trimmed.
fn tail(text: &str, keep: usize) -> String {
    let trimmed = text.trim();
    let skip = trimmed.chars().count().saturating_sub(keep);
    trimmed.chars().skip(skip).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_program_that_is_not_there_is_a_422_that_says_how_to_point_at_it() {
        let failure =
            run_ffmpeg(Path::new("no-such-ffmpeg-program"), &[], &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(serde_json::json!(422)));
        assert!(failure.message.contains("METIL_FFMPEG"), "{failure:?}");
    }

    #[test]
    fn only_the_end_of_a_long_log_goes_into_a_message() {
        let long = format!("{}end of the log\n", "x".repeat(2_000));
        let kept = tail(&long, 20);
        assert_eq!(kept.chars().count(), 20);
        assert!(kept.ends_with("end of the log"));
    }
}
