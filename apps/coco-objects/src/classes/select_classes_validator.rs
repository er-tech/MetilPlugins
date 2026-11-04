use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

use super::coco_classes_config::COCO_CLASSES;

/// Which classes a call looks for: one flag for each of the 80, from the `classes` param (a list of
/// names). Without it, or with an empty list, every class is looked for. A name that is not a COCO
/// class is a 400 that says which.
pub fn select_classes(params: &Value) -> Result<[bool; 80], Failure> {
    let Some(value) = params.get("classes").filter(|value| !value.is_null()) else {
        return Ok([true; 80]);
    };
    let names = value.as_array().ok_or_else(|| {
        Failure::status(400, format!("classes must be a list of names, not {value}"))
    })?;
    if names.is_empty() {
        return Ok([true; 80]);
    }
    let mut wanted = [false; 80];
    for name in names {
        let index = name
            .as_str()
            .and_then(|name| COCO_CLASSES.iter().position(|known| *known == name))
            .ok_or_else(|| {
                Failure::status(
                    400,
                    format!("{name} is not a class this plugin knows (see the enum in plugin.json, for example \"cat\")"),
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
    fn named_classes_are_the_only_ones_wanted_and_none_named_means_all() {
        let cats = select_classes(&json!({ "classes": ["cat", "dog"] })).unwrap();
        assert_eq!(cats.iter().filter(|wanted| **wanted).count(), 2);
        assert!(cats[15] && cats[16], "cat and dog are 15 and 16 in COCO");
        assert!(select_classes(&json!({}))
            .unwrap()
            .iter()
            .all(|wanted| *wanted));
        assert!(select_classes(&json!({ "classes": [] }))
            .unwrap()
            .iter()
            .all(|wanted| *wanted));
    }

    #[test]
    fn a_name_outside_the_set_is_a_400_that_names_it() {
        let failure = select_classes(&json!({ "classes": ["cat", "unicorn"] })).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("unicorn"));
        assert!(select_classes(&json!({ "classes": "cat" })).is_err());
    }
}
