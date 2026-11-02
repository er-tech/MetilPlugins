//! The words and phrases that flag a sentence, for each language the plugin knows.
//!
//! Each list holds words of alarm (help, police, fire, a weapon, "stop") and phrases of a person in
//! distress. They are matched as whole words, ignoring case. The lists were written by a
//! language model and are not checked by a native speaker; the README says so.

/// The codes of the languages that have a list, in the order they are shown.
pub const LANGUAGES: [&str; 5] = ["en", "pt", "es", "fr", "de"];

const ENGLISH: [&str; 13] = [
    "help",
    "emergency",
    "police",
    "ambulance",
    "fire",
    "hurt",
    "bleeding",
    "dying",
    "let go",
    "stop it",
    "gun",
    "knife",
    "don't hurt",
];

const PORTUGUESE: [&str; 15] = [
    "socorro",
    "ajuda",
    "emergência",
    "polícia",
    "ambulância",
    "fogo",
    "incêndio",
    "machucou",
    "sangrando",
    "morrendo",
    "me solta",
    "para com isso",
    "arma",
    "faca",
    "não me machuque",
];

const SPANISH: [&str; 16] = [
    "socorro",
    "auxilio",
    "ayuda",
    "emergencia",
    "policía",
    "ambulancia",
    "fuego",
    "incendio",
    "herido",
    "sangrando",
    "muriendo",
    "suéltame",
    "para ya",
    "pistola",
    "cuchillo",
    "no me hagas daño",
];

const FRENCH: [&str; 16] = [
    "au secours",
    "à l'aide",
    "aidez-moi",
    "urgence",
    "police",
    "ambulance",
    "feu",
    "incendie",
    "blessé",
    "saigne",
    "il meurt",
    "lâchez-moi",
    "arrête",
    "pistolet",
    "couteau",
    "ne me faites pas de mal",
];

const GERMAN: [&str; 13] = [
    "hilfe",
    "notfall",
    "polizei",
    "krankenwagen",
    "feuer",
    "verletzt",
    "blutet",
    "sterbe",
    "lass mich los",
    "hör auf",
    "pistole",
    "messer",
    "tu mir nichts",
];

/// The list for a language code (`en`, `pt`, `es`, `fr` or `de`), or `None` for any other code.
pub fn keywords_for(language: &str) -> Option<&'static [&'static str]> {
    match language {
        "en" => Some(&ENGLISH),
        "pt" => Some(&PORTUGUESE),
        "es" => Some(&SPANISH),
        "fr" => Some(&FRENCH),
        "de" => Some(&GERMAN),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_a_list_and_any_other_code_has_none() {
        for code in LANGUAGES {
            assert!(
                keywords_for(code).is_some_and(|list| !list.is_empty()),
                "{code}"
            );
        }
        assert_eq!(keywords_for("xx"), None);
        assert_eq!(keywords_for("EN"), None);
    }

    #[test]
    fn a_list_has_no_empty_phrase_and_no_word_twice() {
        for code in LANGUAGES {
            let list = keywords_for(code).unwrap();
            let mut seen = std::collections::BTreeSet::new();
            for word in list {
                assert!(!word.trim().is_empty(), "{code} has an empty phrase");
                assert!(seen.insert(word.to_lowercase()), "{code} has {word} twice");
            }
        }
    }
}
