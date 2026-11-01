//! The platforms a plugin lists in its `plugin.json` are the ones its release builds in `smith.toml`:
//! the manifest is what a user is told, the release is what exists, and they must not drift apart.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

const ALL: [&str; 4] = [
    "windows-x86_64",
    "macos-aarch64",
    "macos-x86_64",
    "linux-x86_64",
];

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The release platforms of each project in `smith.toml`, as manifest tags: `windows-x64` is
/// `windows-x86_64`, `macos-arm64` is `macos-aarch64`.
fn release_platforms() -> Vec<(String, BTreeSet<String>)> {
    let text = fs::read_to_string(workspace().join("smith.toml")).unwrap();
    let mut project: Option<String> = None;
    let mut found = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(header) = line.strip_prefix('[') {
            project = header
                .strip_suffix(']')
                .and_then(|name| name.strip_prefix("projects."))
                .and_then(|name| name.strip_suffix(".release"))
                .map(str::to_string);
        } else if let (Some(name), Some(list)) = (&project, line.strip_prefix("platforms")) {
            let tags = list
                .split('"')
                .skip(1)
                .step_by(2)
                .map(|tag| {
                    tag.replace("-x64", "-x86_64")
                        .replace("-arm64", "-aarch64")
                })
                .collect();
            found.push((name.clone(), tags));
        }
    }
    found
}

#[test]
fn a_plugin_lists_the_platforms_its_release_builds() {
    let releases = release_platforms();
    assert!(!releases.is_empty(), "no release platforms found in smith.toml");
    for (name, built) in releases {
        let path = workspace().join("apps").join(&name).join("plugin.json");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let manifest: Value = serde_json::from_str(&text).unwrap();
        // A plugin that lists none runs everywhere (a script, one `any` package): the platforms its
        // release is built on say nothing about where it runs.
        let Some(tags) = manifest["platforms"].as_array() else {
            continue;
        };
        let listed: BTreeSet<String> = tags
            .iter()
            .map(|tag| tag.as_str().unwrap().to_string())
            .collect();
        assert!(
            listed.iter().all(|tag| ALL.contains(&tag.as_str())),
            "{name}: plugin.json lists a platform that is not one of {ALL:?}"
        );
        assert_eq!(
            listed, built,
            "{name}: plugin.json platforms and the release platforms in smith.toml differ"
        );
    }
}
