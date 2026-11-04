use loomasplugins_model_cache::ModelFile;

/// How many times larger a picture a network makes. All the networks of this plugin make it four.
pub const NETWORK_SCALE: usize = 4;

/// An upscaler the plugin can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    pub name: &'static str,
    /// Where to get the network and how to check it. `None` for plain Lanczos scaling.
    pub file: Option<ModelFile>,
    /// The side of the square piece the network accepts, when it accepts one size only. `None`
    /// when it takes any size, and the `tile` option decides.
    pub fixed_tile: Option<usize>,
}

/// The Real-ESRGAN networks, which Xintao Wang and others published under BSD-3-Clause, as ONNX
/// files that other people converted from the official weights. Nothing is packaged: the one asked
/// for is downloaded once and checked against these sums, so a file that someone changed is refused.
pub const MODELS: [ModelSpec; 4] = [
    ModelSpec {
        name: "general-x4v3",
        file: Some(ModelFile {
            file: "realesr-general-x4v3.onnx",
            url: "https://huggingface.co/vlad-grig/realesr-general-x4v3-onnx/resolve/main/onnx/model.onnx",
            sha256: "7b10e80021ec3e0ff35017ae5022791879d93a799b2e201e1010a37510202266",
            bytes: 4_866_420,
        }),
        fixed_tile: None,
    },
    ModelSpec {
        name: "animevideo-v3",
        file: Some(ModelFile {
            file: "realesr-animevideov3-256.onnx",
            url: "https://huggingface.co/Tomoali/realesr-animevideov3-onnx/resolve/main/animevideov3_nchw_256x256_fp32.onnx",
            sha256: "139f166bd2b955c7d4cd31ca2616eb97dacc4365c1c315d880735ce72a0663d9",
            bytes: 2_494_775,
        }),
        fixed_tile: Some(256),
    },
    ModelSpec {
        name: "x4plus",
        file: Some(ModelFile {
            file: "real-esrgan-x4plus-128.onnx",
            url: "https://huggingface.co/bukuroo/RealESRGAN-ONNX/resolve/main/real-esrgan-x4plus-128.onnx",
            sha256: "6a6f4a3d58553d40fdd443d9e5f4b2deb9b52bef1ec2947700fc2167ac876c7d",
            bytes: 67_160_311,
        }),
        fixed_tile: Some(128),
    },
    ModelSpec {
        name: "lanczos",
        file: None,
        fixed_tile: None,
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
    fn every_network_has_a_checksum_and_every_name_can_be_found() {
        for model in &MODELS {
            if let Some(file) = &model.file {
                assert_eq!(file.sha256.len(), 64, "{}", model.name);
                assert!(file.url.starts_with("https://"), "{}", model.name);
            }
            assert_eq!(model_named(model.name), Some(model));
        }
        assert!(model_named("huge").is_none());
    }

    #[test]
    fn only_the_baseline_has_no_file() {
        let without: Vec<&str> = MODELS
            .iter()
            .filter(|model| model.file.is_none())
            .map(|model| model.name)
            .collect();
        assert_eq!(without, ["lanczos"]);
    }
}
