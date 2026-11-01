use std::ffi::OsString;
use std::path::PathBuf;

/// The FFmpeg program to start: what `METIL_FFMPEG` names (a folder with `ffmpeg` in it, or the
/// program itself), and plain `ffmpeg` (the system path) when it is not set.
pub fn ffmpeg_program(configured: Option<OsString>) -> PathBuf {
    let name = format!("ffmpeg{}", std::env::consts::EXE_SUFFIX);
    match configured.map(PathBuf::from) {
        Some(location) if location.is_dir() => location.join(name),
        Some(location) => location,
        None => PathBuf::from(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_set_means_the_system_path_and_a_folder_means_the_program_in_it() {
        assert!(ffmpeg_program(None).to_string_lossy().starts_with("ffmpeg"));
        let folder = std::env::temp_dir();
        assert_eq!(
            ffmpeg_program(Some(folder.clone().into_os_string())).parent(),
            Some(folder.as_path())
        );
        assert_eq!(
            ffmpeg_program(Some("tools/ffmpeg-custom".into())),
            PathBuf::from("tools/ffmpeg-custom")
        );
    }
}
