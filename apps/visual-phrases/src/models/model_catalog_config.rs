use metilplugins_model_cache::ModelFile;

/// The files of SigLIP (base, patch 16, 224 pixels), Google's image-text model (Apache-2.0), as
/// the ONNX export of `Xenova/siglip-base-patch16-224` with the weights quantised to 8 bits. The
/// image model and the text model are apart: a picture and a phrase each become 768 numbers, and
/// how alike they are says how well the phrase describes the picture. Nothing is packaged: they
/// are downloaded once and checked against these sums.
pub const VISION: ModelFile = ModelFile {
    file: "vision_model_int8.onnx",
    url: "https://huggingface.co/Xenova/siglip-base-patch16-224/resolve/main/onnx/vision_model_int8.onnx",
    sha256: "6d00762fcb4aef9bdee1b886fcccc7df466f9eae321e180f97d88e24fa13ac72",
    bytes: 94_098_316,
};

pub const TEXT: ModelFile = ModelFile {
    file: "text_model_int8.onnx",
    url: "https://huggingface.co/Xenova/siglip-base-patch16-224/resolve/main/onnx/text_model_int8.onnx",
    sha256: "9cb5102160e2a2b90c0a999ed6c2b4090865c9d5aa08f09cd10993ad9b38bd5f",
    bytes: 110_982_746,
};

/// How a phrase is cut into the pieces the text model knows.
pub const TOKENIZER: ModelFile = ModelFile {
    file: "tokenizer.json",
    url: "https://huggingface.co/Xenova/siglip-base-patch16-224/resolve/main/tokenizer.json",
    sha256: "4a17c975210be5ab4c36b47d8dae4eefb866dbfb1e676e394aad85dc30a3ae08",
    bytes: 2_398_744,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_has_a_checksum_and_a_url_that_ends_with_its_name() {
        for model in [VISION, TEXT, TOKENIZER] {
            assert_eq!(model.sha256.len(), 64, "{}", model.file);
            assert!(model.url.ends_with(model.file), "{}", model.file);
        }
    }
}
