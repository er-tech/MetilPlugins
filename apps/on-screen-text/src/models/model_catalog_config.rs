use loomasplugins_model_cache::ModelFile;

/// A network that finds where the text is in a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Detector {
    pub name: &'static str,
    pub file: ModelFile,
}

/// A script the plugin can read: the network that reads a line of text and the list of the
/// characters it knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    pub name: &'static str,
    pub title: &'static str,
    pub recognizer: ModelFile,
    pub dictionary: ModelFile,
}

/// The PaddleOCR networks, as ONNX files that `monkt` converted from the official weights
/// (Apache-2.0), at one commit. Nothing is packaged: the ones asked for are downloaded once and
/// checked against these sums, so a file that someone changed is refused.
pub const DETECTORS: [Detector; 2] = [
    Detector {
        name: "fast",
        file: ModelFile { file: "detector-fast.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/detection/v3/det.onnx", sha256: "ee40e80071ba3a320d4efda75f3e22047a7d049e9bf7bcaaf9daea23fc21b935", bytes: 2_429_873 },
    },
    Detector {
        name: "accurate",
        file: ModelFile { file: "detector-accurate.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/detection/v5/det.onnx", sha256: "61824840edf6e74581898930b8091b1b2318f4b2705a2e8a40ad3de7ac480133", bytes: 88_030_804 },
    },
];

/// The scripts, the Latin one first (it reads Portuguese, Spanish, French and most of Europe).
pub const LANGUAGES: [Language; 11] = [
    Language {
        name: "latin",
        title: "Latin letters: Portuguese, Spanish, French, German, Italian, Dutch, Polish, Turkish and about forty more",
        recognizer: ModelFile { file: "latin-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/latin/rec.onnx", sha256: "614ffc2d6d3902d360fad7f1b0dd455ee45e877069d14c4e51a99dc4ef144409", bytes: 7_862_832 },
        dictionary: ModelFile { file: "latin-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/latin/dict.txt", sha256: "3c0a8a79b612653c25f765271714f71281e4e955962c153e272b7b8c1d2b13ff", bytes: 1_634 },
    },
    Language {
        name: "english",
        title: "English only (a little more exact on English than the Latin model)",
        recognizer: ModelFile { file: "english-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/english/rec.onnx", sha256: "4e16deb22c4da6468bdca539b2cd3c8687825538b67109177c47d359ab994cd7", bytes: 7_830_888 },
        dictionary: ModelFile { file: "english-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/english/dict.txt", sha256: "e025a66d31f327ba0c232e03f407ae8d105e1e709e7ccb3f408aa778c24e70d6", bytes: 1_416 },
    },
    Language {
        name: "chinese",
        title: "Chinese, Japanese and English",
        recognizer: ModelFile { file: "chinese-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/chinese/rec.onnx", sha256: "26fa4f47060f58e25962b9af6beaee05c8182b90e026c4ecc6db165d9dfdc38a", bytes: 84_468_836 },
        dictionary: ModelFile { file: "chinese-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/chinese/dict.txt", sha256: "d1979e9f794c464c0d2e0b70a7fe14dd978e9dc644c0e71f14158cdf8342af1b", bytes: 74_012 },
    },
    Language {
        name: "korean",
        title: "Korean and English",
        recognizer: ModelFile { file: "korean-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/korean/rec.onnx", sha256: "322f140154c820fcb83c3d24cfe42c9ec70dd1a1834163306a7338136e4f1eaa", bytes: 13_401_252 },
        dictionary: ModelFile { file: "korean-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/korean/dict.txt", sha256: "a88071c68c01707489baa79ebe0405b7beb5cca229f4fc94cc3ef992328802d7", bytes: 47_451 },
    },
    Language {
        name: "arabic",
        title: "Arabic, Persian and Urdu",
        recognizer: ModelFile { file: "arabic-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/arabic/rec.onnx", sha256: "7982d371612785238fd99080cff36354deaec84fdc6ff7da9c82af4243fa0c9a", bytes: 8_978_664 },
        dictionary: ModelFile { file: "arabic-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/arabic/dict.txt", sha256: "637c27c88512c22089bef927b34ada08f748dc132ac70facd68d8202384c2726", bytes: 405 },
    },
    Language {
        name: "eslav",
        title: "Cyrillic: Russian, Ukrainian, Belarusian, Bulgarian and others",
        recognizer: ModelFile { file: "eslav-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/eslav/rec.onnx", sha256: "dc6bf0e855247decce214ba6dae5bc135fa0ad725a5918a7fcfb59fad6c9cdee", bytes: 7_870_092 },
        dictionary: ModelFile { file: "eslav-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/eslav/dict.txt", sha256: "3e95f1581557162870cacdba5af91a4c6be2890710d395b0c3c7578e7ee5e6eb", bytes: 1_663 },
    },
    Language {
        name: "greek",
        title: "Greek",
        recognizer: ModelFile { file: "greek-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/greek/rec.onnx", sha256: "13373f736dbb229e96945fc41c2573403d91503b0775c7b7294839e0c5f3a7a3", bytes: 7_791_200 },
        dictionary: ModelFile { file: "greek-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/greek/dict.txt", sha256: "31defc62c0c3ad3674a82da6192226a2ba98ef4ff014a7045cb88d59f9c3de31", bytes: 1_103 },
    },
    Language {
        name: "hindi",
        title: "Devanagari: Hindi, Marathi, Nepali",
        recognizer: ModelFile { file: "hindi-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/hindi/rec.onnx", sha256: "43df175fa3c877fbf7bcc4e5bd1e203e24ec450cd3ea96c9e802c86e39a4d4cf", bytes: 8_980_224 },
        dictionary: ModelFile { file: "hindi-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/hindi/dict.txt", sha256: "b5f1be6d8bbff1a19fb96c5d4ca96a423380234bb7d2ce0e07b5838adb4d18ea", bytes: 508 },
    },
    Language {
        name: "thai",
        title: "Thai",
        recognizer: ModelFile { file: "thai-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/thai/rec.onnx", sha256: "2b6e56b1872200349e227574c25aeb0e0f9af9b8356e9ff5f75ac543a535669a", bytes: 7_873_480 },
        dictionary: ModelFile { file: "thai-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/thai/dict.txt", sha256: "57f5406f94bb6688fb7077f7be65f08bbd71cecf48c01ea26c522cb5c4836b7a", bytes: 1_767 },
    },
    Language {
        name: "tamil",
        title: "Tamil",
        recognizer: ModelFile { file: "tamil-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/tamil/rec.onnx", sha256: "fba9a00af746c8f3ef4f091cf966880222cd7245a60f6a953670593630c0ca4f", bytes: 8_970_084 },
        dictionary: ModelFile { file: "tamil-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/tamil/dict.txt", sha256: "e93e694814afd9ff1e918b6f4bb4267fb4c65a9344ee5d7464f9135b90c0a270", bytes: 352 },
    },
    Language {
        name: "telugu",
        title: "Telugu",
        recognizer: ModelFile { file: "telugu-rec.onnx", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/telugu/rec.onnx", sha256: "669946d9ad4de93e8d1b13f7e72c59cc2cc2de91bb24e22aad7503a6cc5757ee", bytes: 8_976_064 },
        dictionary: ModelFile { file: "telugu-dict.txt", url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/telugu/dict.txt", sha256: "ee9946a60d7701977474e89883694a4ca20eaa3868a23334bf7940dcd008f0f9", bytes: 429 },
    },
];

/// The detector called `name`, or `None`.
pub fn detector_named(name: &str) -> Option<&'static Detector> {
    DETECTORS.iter().find(|detector| detector.name == name)
}

/// The script called `name`, or `None`.
pub fn language_named(name: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|language| language.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_has_a_checksum_and_a_secure_address_and_every_name_is_found() {
        for detector in &DETECTORS {
            assert_eq!(detector.file.sha256.len(), 64, "{}", detector.name);
            assert!(detector.file.url.starts_with("https://"));
            assert_eq!(detector_named(detector.name), Some(detector));
        }
        for language in &LANGUAGES {
            for file in [&language.recognizer, &language.dictionary] {
                assert_eq!(file.sha256.len(), 64, "{}", language.name);
                assert!(file.url.starts_with("https://"));
            }
            assert_eq!(language_named(language.name), Some(language));
        }
        assert!(language_named("klingon").is_none() && detector_named("huge").is_none());
    }
}
