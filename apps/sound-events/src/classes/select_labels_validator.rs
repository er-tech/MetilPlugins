use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

use super::audioset_classes_config::AUDIOSET_CLASSES;

/// Which classes a call looks for: one flag for each of the 521, from the `labels` param (a list of
/// names, capitals as in the enum of plugin.json). Without it, or with an empty list, every class
/// is looked for. A name that is not a class is a 400 that says which.
pub fn select_labels(params: &Value) -> Result<[bool; 521], Failure> {
    let Some(value) = params.get("labels").filter(|value| !value.is_null()) else {
        return Ok([true; 521]);
    };
    let names = value.as_array().ok_or_else(|| {
        Failure::status(400, format!("labels must be a list of names, not {value}"))
    })?;
    if names.is_empty() {
        return Ok([true; 521]);
    }
    let mut wanted = [false; 521];
    for name in names {
        let index = name
            .as_str()
            .and_then(|name| AUDIOSET_CLASSES.iter().position(|known| *known == name))
            .ok_or_else(|| {
                Failure::status(
                    400,
                    format!("{name} is not a class this plugin knows (see the enum in plugin.json, for example \"Meow\")"),
                )
            })?;
        wanted[index] = true;
    }
    Ok(wanted)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn named_labels_are_the_only_ones_wanted_and_none_named_means_all() {
        let wanted = select_labels(&json!({ "labels": ["Cat", "Meow"] })).unwrap();
        assert_eq!(wanted.iter().filter(|wanted| **wanted).count(), 2);
        assert!(wanted[76], "Cat is 76 in the class map");
        assert!(select_labels(&json!({})).unwrap().iter().all(|w| *w));
        assert!(select_labels(&json!({ "labels": [] }))
            .unwrap()
            .iter()
            .all(|w| *w));
    }

    #[test]
    fn a_name_outside_the_set_is_a_400_that_names_it() {
        let failure = select_labels(&json!({ "labels": ["Cat", "cat"] })).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("\"cat\""));
        assert!(select_labels(&json!({ "labels": "Cat" })).is_err());
    }
}
