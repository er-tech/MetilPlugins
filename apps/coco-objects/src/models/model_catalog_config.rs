use loomasplugins_model_cache::ModelFile;

/// A detector the plugin can use: the file (where to get it, how to check it) and the size of the
/// square picture it looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    pub name: &'static str,
    pub file: ModelFile,
    /// The side of the square the picture is fitted in before the model sees it.
    pub input_size: usize,
}

/// The YOLOX detectors published by their authors (Megvii) under Apache-2.0, trained on COCO.
/// Nothing is packaged: the one asked for is downloaded once and checked against these sums.
pub const MODELS: [ModelSpec; 3] = [
    ModelSpec {
        name: "nano",
        file: ModelFile {
            file: "yolox_nano.onnx",
            url: "https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_nano.onnx",
            sha256: "c789161ed43c8269fcd4e67c67eeeb4e80c622da2eb296a20bc6007bd18a0b7d",
            bytes: 3_659_407,
        },
        input_size: 416,
    },
    ModelSpec {
        name: "tiny",
        file: ModelFile {
            file: "yolox_tiny.onnx",
            url: "https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_tiny.onnx",
            sha256: "427cc366d34e27ff7a03e2899b5e3671425c262ea2291f88bb942bc1cc70b0f7",
            bytes: 20_219_662,
        },
        input_size: 416,
    },
    ModelSpec {
        name: "s",
        file: ModelFile {
            file: "yolox_s.onnx",
            url: "https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_s.onnx",
            sha256: "c5c2d13e59ae883e6af3b45daea64af4833a4951c92d116ec270d9ddbe998063",
            bytes: 35_858_002,
        },
        input_size: 640,
    },
];

/// The model called `name`, or `None`.
pub fn model_named(name: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|model| model.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_has_a_checksum_and_a_name_the_manifest_can_list() {
        for model in &MODELS {
            assert_eq!(model.file.sha256.len(), 64, "{}", model.name);
            assert!(model.file.url.ends_with(model.file.file));
            assert_eq!(model_named(model.name), Some(model));
        }
        assert!(model_named("huge").is_none());
    }
}
