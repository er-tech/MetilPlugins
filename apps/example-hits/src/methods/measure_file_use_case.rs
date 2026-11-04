use serde_json::{json, Value};

use loomasplugins_plugin_sdk::Failure;

/// Gives the size in bytes of the stored file passed as `params.audio`: a stored file arrives as an
/// object with an absolute `file` path the plugin can open.
pub fn measure_file(params: &Value) -> Result<Value, Failure> {
    let file = params["audio"]["file"]
        .as_str()
        .ok_or_else(|| Failure::status(400, "audio has no file"))?;
    let metadata = std::fs::metadata(file)
        .map_err(|error| Failure::status(404, format!("{file}: {error}")))?;
    Ok(json!({ "bytes": metadata.len() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_a_file_and_explains_a_missing_one() {
        let path = std::env::temp_dir().join(format!("example-hits-{}.bin", std::process::id()));
        std::fs::write(&path, [0_u8; 42]).unwrap();
        let params = json!({ "audio": { "file": path.to_string_lossy() } });
        assert_eq!(measure_file(&params).unwrap(), json!({ "bytes": 42 }));
        let _ = std::fs::remove_file(&path);

        // A file that is not there is a 404; a parameter that is missing is a 400.
        let missing = measure_file(&params).unwrap_err();
        assert_eq!(missing.code, Some(json!(404)));
        let no_file = measure_file(&json!({})).unwrap_err();
        assert_eq!(no_file.message, "audio has no file");
        assert_eq!(no_file.code, Some(json!(400)));
    }
}
