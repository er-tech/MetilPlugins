use loomasplugins_model_cache::ModelFile;

/// DeepFilterNet3 (version 0.5.6 of the official model, Apache-2.0 or MIT) as the ONNX graph of
/// `soniqo/DeepFilterNet3-ONNX` (MIT). It is only the neural network: it takes features of the
/// sound (computed by this plugin) and gives a mask and filter coefficients. Nothing is packaged:
/// the files are downloaded once and checked against these sums.
pub const NETWORK: ModelFile = ModelFile {
    file: "deepfilter.onnx",
    url: "https://huggingface.co/soniqo/DeepFilterNet3-ONNX/resolve/main/deepfilter.onnx",
    sha256: "e1157049059434ae0d5857e32c812abea227b975e946b2eb64d001abbce156d3",
    bytes: 8_608_859,
};

/// The constants the network was trained with: the matrices that turn 481 frequencies into 32
/// bands and back, and the window of the transform.
pub const TABLES: ModelFile = ModelFile {
    file: "deepfilter-auxiliary.bin",
    url: "https://huggingface.co/soniqo/DeepFilterNet3-ONNX/resolve/main/deepfilter-auxiliary.bin",
    sha256: "47e84480f823ab95bee69d9f8a2344074e3d8e7dbb4370d44785b91698a4dca1",
    bytes: 126_976,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_has_a_checksum_and_a_url_that_ends_with_its_name() {
        for model in [NETWORK, TABLES] {
            assert_eq!(model.sha256.len(), 64, "{}", model.file);
            assert!(model.url.ends_with(model.file), "{}", model.file);
        }
    }
}
