//! The words and phrases that flag a sentence, for each language the plugin knows, in three kinds:
//! words of distress, words about harming oneself, and aggressive words.
//!
//! They are matched as whole words, ignoring case, and a phrase must come in order. The lists were
//! written by a language model and are not checked by a native speaker; the README says so. A
//! phrase inside another phrase of the same kind is left out (`help` covers `help me`), because a
//! sentence counts its different phrases.

/// The codes of the languages that have lists, in the order they are shown.
pub const LANGUAGES: [&str; 5] = ["en", "pt", "es", "fr", "de"];

/// The built-in words of one language.
#[derive(Debug, Clone, Copy)]
pub struct WordLists {
    /// What a person in distress may say: a cry for help, not being able to breathe, fear, "stop".
    pub distress: &'static [&'static str],
    /// What a person may say about ending their own life or hurting themselves.
    pub self_harm: &'static [&'static str],
    /// Threats and insults.
    pub aggression: &'static [&'static str],
}

const ENGLISH: WordLists = WordLists {
    distress: &[
        "help",
        "can't breathe",
        "cannot breathe",
        "i'm dying",
        "i'm going to die",
        "please stop",
        "stop it",
        "don't hurt me",
        "let go",
        "get off me",
        "leave me alone",
        "i can't do this",
        "i can't take it",
        "i'm scared",
    ],
    self_harm: &[
        "want to die",
        "kill myself",
        "end my life",
        "end it all",
        "can't go on",
        "no reason to live",
        "better off without me",
        "hurt myself",
    ],
    aggression: &[
        "kill you",
        "hurt you",
        "hit you",
        "beat you",
        "shut up",
        "get out",
        "back off",
        "you're dead",
        "i hate you",
        "you idiot",
        "you bastard",
        "fuck you",
        "you'll regret",
        "i'll find you",
    ],
};

const PORTUGUESE: WordLists = WordLists {
    distress: &[
        "socorro",
        "ajuda",
        "não consigo respirar",
        "estou morrendo",
        "vou morrer",
        "para com isso",
        "me solta",
        "me larga",
        "me deixa em paz",
        "não me machuque",
        "não aguento",
        "não consigo mais",
        "estou com medo",
    ],
    self_harm: &[
        "quero morrer",
        "me matar",
        "acabar com a minha vida",
        "acabar com tudo",
        "não quero mais viver",
        "melhor sem mim",
        "me machucar",
    ],
    aggression: &[
        "te matar",
        "te machucar",
        "te bater",
        "cala a boca",
        "sai daqui",
        "vai embora",
        "você está morto",
        "te odeio",
        "idiota",
        "filho da puta",
    ],
};

const SPANISH: WordLists = WordLists {
    distress: &[
        "socorro",
        "auxilio",
        "ayuda",
        "no puedo respirar",
        "me muero",
        "voy a morir",
        "para ya",
        "suéltame",
        "déjame en paz",
        "no me hagas daño",
        "no aguanto más",
        "no puedo más",
        "tengo miedo",
    ],
    self_harm: &[
        "quiero morir",
        "quiero morirme",
        "matarme",
        "acabar con mi vida",
        "acabar con todo",
        "no quiero vivir",
        "estaría mejor sin mí",
        "hacerme daño",
    ],
    aggression: &[
        "matarte",
        "te mato",
        "te voy a pegar",
        "cállate",
        "lárgate",
        "vete de aquí",
        "estás muerto",
        "te odio",
        "idiota",
        "hijo de puta",
    ],
};

const FRENCH: WordLists = WordLists {
    distress: &[
        "au secours",
        "à l'aide",
        "aidez-moi",
        "je n'arrive pas à respirer",
        "je ne peux pas respirer",
        "je meurs",
        "je vais mourir",
        "arrête",
        "lâchez-moi",
        "laisse-moi tranquille",
        "ne me faites pas de mal",
        "je n'en peux plus",
        "j'ai peur",
    ],
    self_harm: &[
        "je veux mourir",
        "me tuer",
        "en finir",
        "mettre fin à mes jours",
        "plus envie de vivre",
        "me faire du mal",
    ],
    aggression: &[
        "te tuer",
        "te frapper",
        "ferme-la",
        "dégage",
        "tu es mort",
        "je te déteste",
        "connard",
        "va te faire foutre",
    ],
};

const GERMAN: WordLists = WordLists {
    distress: &[
        "hilfe",
        "ich kann nicht atmen",
        "ich sterbe",
        "ich werde sterben",
        "hör auf",
        "lass mich los",
        "lass mich in ruhe",
        "tu mir nichts",
        "ich kann nicht mehr",
        "ich habe angst",
    ],
    self_harm: &[
        "ich will sterben",
        "mich umbringen",
        "mein leben beenden",
        "nicht mehr leben",
        "mir wehtun",
    ],
    aggression: &[
        "dich umbringen",
        "dich schlagen",
        "halt die klappe",
        "verschwinde",
        "du bist tot",
        "ich hasse dich",
        "du idiot",
        "arschloch",
    ],
};

/// The lists for a language code (`en`, `pt`, `es`, `fr` or `de`), or `None` for any other code.
pub fn word_lists_for(language: &str) -> Option<WordLists> {
    match language {
        "en" => Some(ENGLISH),
        "pt" => Some(PORTUGUESE),
        "es" => Some(SPANISH),
        "fr" => Some(FRENCH),
        "de" => Some(GERMAN),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_three_lists_and_any_other_code_has_none() {
        for code in LANGUAGES {
            let lists = word_lists_for(code).unwrap();
            for list in [lists.distress, lists.self_harm, lists.aggression] {
                assert!(!list.is_empty(), "{code}");
            }
        }
        assert!(word_lists_for("xx").is_none());
        assert!(word_lists_for("EN").is_none());
    }

    #[test]
    fn a_list_has_no_empty_phrase_no_phrase_twice_and_no_phrase_inside_another() {
        for code in LANGUAGES {
            let lists = word_lists_for(code).unwrap();
            for list in [lists.distress, lists.self_harm, lists.aggression] {
                for (index, word) in list.iter().enumerate() {
                    assert!(!word.trim().is_empty(), "{code} has an empty phrase");
                    for other in &list[index + 1..] {
                        assert_ne!(word.to_lowercase(), other.to_lowercase(), "{code}: {word}");
                        let (long, short) = if word.len() >= other.len() {
                            (word, other)
                        } else {
                            (other, word)
                        };
                        let padded = format!(" {long} ");
                        assert!(
                            !padded.contains(&format!(" {short} ")),
                            "{code}: {short:?} is inside {long:?}, so one sentence counts it twice"
                        );
                    }
                }
            }
        }
    }
}
