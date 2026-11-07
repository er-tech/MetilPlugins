use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};

use super::locate_model_use_case::locate_model;
use super::model_error::ModelError;
use super::model_file_contract::ModelFile;

/// The file called `member` inside the zip `archive` of the plugin called `plugin`: the zip is
/// downloaded and checked like any model file, and the member is taken out of it once, into the
/// same folder (the next call finds it). `member` is a file name; the folders in front of it in the
/// zip do not matter, so the version folder a publisher puts in the zip can change. `progress` is
/// told when a download starts.
pub fn locate_zipped_model(
    plugin: &str,
    archive: &ModelFile,
    member: &str,
    progress: impl Fn(&str),
) -> Result<PathBuf, ModelError> {
    let zip_path = locate_model(plugin, archive, &progress)?;
    let target = zip_path.with_file_name(format!(
        "{}.{member}",
        archive.file.trim_end_matches(".zip")
    ));
    if target.is_file() {
        return Ok(target);
    }
    progress(&format!("unpacking {member} from {}", archive.file));
    take_out(&zip_path, member, &target).map_err(|message| ModelError {
        code: 422,
        message: format!(
            "{member} could not be taken out of {}: {message}",
            archive.file
        ),
        retryable: false,
    })?;
    Ok(target)
}

/// Copies the member of the zip at `archive` to `target`, through a partial file so that a stop in
/// the middle leaves nothing that looks like a model.
fn take_out(archive: &Path, member: &str, target: &Path) -> Result<(), String> {
    let file = File::open(archive).map_err(|error| error.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    let name = (0..zip.len())
        .filter_map(|at| zip.by_index(at).ok().map(|entry| entry.name().to_string()))
        .find(|name| name == member || name.ends_with(&format!("/{member}")))
        .ok_or_else(|| "it is not in the zip".to_string())?;
    let mut entry = zip.by_name(&name).map_err(|error| error.to_string())?;
    let partial = target.with_extension("partial");
    let mut out = File::create(&partial).map_err(|error| error.to_string())?;
    let copied = copy(&mut entry, &mut out).map_err(|error| error.to_string());
    drop(out);
    if let Err(message) = copied {
        let _ = std::fs::remove_file(&partial);
        return Err(message);
    }
    std::fs::rename(&partial, target).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn a_member_is_taken_out_whatever_folders_come_before_it_and_a_missing_one_is_refused() {
        let folder = std::env::temp_dir().join(format!("loomas-zip-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let archive = folder.join("pack.zip");
        let mut writer = zip::ZipWriter::new(File::create(&archive).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        writer
            .start_file("20230928/pack/model.onnx", options)
            .unwrap();
        writer.write_all(b"the network").unwrap();
        writer
            .start_file("20230928/pack/other.txt", options)
            .unwrap();
        writer.write_all(b"not it").unwrap();
        writer.finish().unwrap();

        let target = folder.join("pack.model.onnx");
        take_out(&archive, "model.onnx", &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"the network");
        let missing = take_out(&archive, "nothing.onnx", &folder.join("nothing")).unwrap_err();
        assert!(missing.contains("not in the zip"), "{missing}");
        assert!(!folder.join("nothing").exists());
        let _ = std::fs::remove_dir_all(&folder);
    }
}
