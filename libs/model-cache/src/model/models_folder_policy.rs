use std::path::PathBuf;

/// Where the models are kept: the folder in `METIL_MODELS` when it is set, else
/// `.metil/models` in the home folder. `None` when neither can be found.
pub fn models_folder(variable: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    if let Some(folder) = variable("METIL_MODELS").filter(|folder| !folder.is_empty()) {
        return Some(PathBuf::from(folder));
    }
    let home = variable("USERPROFILE")
        .or_else(|| variable("HOME"))
        .filter(|home| !home.is_empty())?;
    Some(PathBuf::from(home).join(".metil").join("models"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_string())
        }
    }

    #[test]
    fn the_variable_wins_then_the_home_folder_and_without_either_there_is_no_place() {
        assert_eq!(
            models_folder(env(&[("METIL_MODELS", "/m"), ("HOME", "/h")])),
            Some(PathBuf::from("/m"))
        );
        assert_eq!(
            models_folder(env(&[("HOME", "/h")])),
            Some(PathBuf::from("/h").join(".metil").join("models"))
        );
        assert_eq!(models_folder(env(&[("METIL_MODELS", "")])), None);
    }
}
