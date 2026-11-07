use loomasplugins_model_cache::ModelFile;

/// A sentiment model the plugin can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    pub name: &'static str,
    pub file: ModelFile,
}

/// How a text is cut into the pieces the model knows (the same for both models).
pub const TOKENIZER: ModelFile = ModelFile {
    file: "tokenizer.json",
    url: "https://huggingface.co/lxyuan/distilbert-base-multilingual-cased-sentiments-student/resolve/cf991100d706c13c0a080c097134c05b7f436c45/tokenizer.json",
    sha256: "a4a4fa4b10a2a40361529954532f10257fda53cb22177554a862bcc661b7e4d8",
    bytes: 2_919_615,
};

/// `lxyuan/distilbert-base-multilingual-cased-sentiments-student` (Apache-2.0): a DistilBERT that
/// learned from a larger model to tell positive, neutral and negative in 12 languages, English and
/// Portuguese among them, as Xenova converted it to ONNX: `standard` is the 8-bit version (a
/// fifth of the size, a little less sure of itself) and `full` is the one with 32-bit numbers. The
/// ONNX file in the repository of the author itself gives the same answer (negative) for every
/// text, so it is not used. Nothing is packaged: the one asked for is downloaded once and checked
/// against these sums.
pub const MODELS: [ModelSpec; 2] = [
    ModelSpec {
        name: "standard",
        file: ModelFile {
            file: "sentiment-int8.onnx",
            url: "https://huggingface.co/Xenova/distilbert-base-multilingual-cased-sentiments-student/resolve/9d9ac661fd7b0b48535a1fc99b20ae6947629e65/onnx/model_int8.onnx",
            sha256: "9570fdd726d952dbaf559ba0b9f38a523554f08a8038d62d9263a90dcbb3cd3e",
            bytes: 135_953_840,
        },
    },
    ModelSpec {
        name: "full",
        file: ModelFile {
            file: "sentiment-fp32.onnx",
            url: "https://huggingface.co/Xenova/distilbert-base-multilingual-cased-sentiments-student/resolve/9d9ac661fd7b0b48535a1fc99b20ae6947629e65/onnx/model.onnx",
            sha256: "089dbb9dabce36b1fe8154c8516aeb77b4da12aa3129d17949774de3cfeeb306",
            bytes: 541_443_593,
        },
    },
];

/// The model called `name`, or `None`.
pub fn model_named(name: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|model| model.name == name)
}

/// The tokenizer file.
pub fn tokenizer_file() -> &'static ModelFile {
    &TOKENIZER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_has_a_checksum_and_a_secure_address_and_every_name_is_found() {
        for model in &MODELS {
            assert_eq!(model.file.sha256.len(), 64, "{}", model.name);
            assert!(model.file.url.starts_with("https://"), "{}", model.name);
            assert_eq!(model_named(model.name), Some(model));
        }
        assert_eq!(TOKENIZER.sha256.len(), 64);
        assert!(model_named("huge").is_none());
    }
}
