use loomasplugins_model_cache::ModelFile;

/// How a model finds the keypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pipeline {
    /// A detector finds the people, then a pose network reads the keypoints in a crop of each
    /// (RTMDet-family YOLOX and RTMPose of OpenMMLab, Apache-2.0).
    RtmPose,
    /// One network finds up to six people and their keypoints in the whole picture (MoveNet
    /// multipose lightning of Google, Apache-2.0).
    MoveNet,
}

/// A way to read the movements of people.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    pub name: &'static str,
    pub pipeline: Pipeline,
    /// The detector, as a zip with `end2end.onnx` in it (a pose-only pipeline has none).
    pub detector: Option<ModelFile>,
    /// The side of the square the detector looks at, in pixels (416 for the small one, 640 for the
    /// medium one); 0 when there is no detector.
    pub detector_size: usize,
    /// The pose network: a zip with `end2end.onnx` in it for RTMPose, the ONNX file for MoveNet.
    pub pose: ModelFile,
}

/// The files come from the publishers, as ONNX that they made themselves from their official
/// weights: OpenMMLab for the RTMPose networks, and Xenova (a conversion of Google's MoveNet) for
/// MoveNet. Nothing is packaged: the ones asked for are downloaded once and checked against these
/// sums.
pub const MODELS: [ModelSpec; 3] = [
    ModelSpec {
        name: "rtmpose-small",
        pipeline: Pipeline::RtmPose,
        detector_size: 416,
        detector: Some(ModelFile {
            file: "yolox-tiny-humanart.zip",
            url: "https://download.openmmlab.com/mmpose/v1/projects/rtmposev1/onnx_sdk/yolox_tiny_8xb8-300e_humanart-6f3252f9.zip",
            sha256: "36e09ca555916253fa1b5d51bec01e48e98e1f0af966c51ce3e844d8c8fc4cfc",
            bytes: 18_887_737,
        }),
        pose: ModelFile {
            file: "rtmpose-s-body7.zip",
            url: "https://download.openmmlab.com/mmpose/v1/projects/rtmposev1/onnx_sdk/rtmpose-s_simcc-body7_pt-body7_420e-256x192-acd4a1ef_20230504.zip",
            sha256: "7673922e531014906ca4f0f239b7e233b740146a10b632deaa2a28d45470d802",
            bytes: 20_496_303,
        },
    },
    ModelSpec {
        name: "rtmpose-medium",
        pipeline: Pipeline::RtmPose,
        detector_size: 640,
        detector: Some(ModelFile {
            file: "yolox-m-humanart.zip",
            url: "https://download.openmmlab.com/mmpose/v1/projects/rtmposev1/onnx_sdk/yolox_m_8xb8-300e_humanart-c2c7a14a.zip",
            sha256: "a000224fd8ba283202bc62d4a5fcdfe353adb9f468777dbac1ea2ada2093adde",
            bytes: 94_223_081,
        }),
        pose: ModelFile {
            file: "rtmpose-m-body7.zip",
            url: "https://download.openmmlab.com/mmpose/v1/projects/rtmposev1/onnx_sdk/rtmpose-m_simcc-body7_pt-body7_420e-256x192-e48f03d0_20230504.zip",
            sha256: "f7fbb6c5c11a1bb70f3d445e4ddec5d144ea89ad5649c081ef976f9b24a0b741",
            bytes: 50_799_818,
        },
    },
    ModelSpec {
        name: "movenet",
        pipeline: Pipeline::MoveNet,
        detector_size: 0,
        detector: None,
        pose: ModelFile {
            file: "movenet-multipose-lightning.onnx",
            url: "https://huggingface.co/Xenova/movenet-multipose-lightning/resolve/8506174df310cf154b97352afa98d1ea4b1e4b47/onnx/model.onnx",
            sha256: "60f724a56f0119d22d2afcf530019db307c77d4ea598eb5bd53bb65afe93930d",
            bytes: 19_049_169,
        },
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
    fn every_file_has_a_checksum_and_a_secure_address_and_every_name_is_found() {
        for model in &MODELS {
            for file in model.detector.iter().chain([&model.pose]) {
                assert_eq!(file.sha256.len(), 64, "{}", model.name);
                assert!(file.url.starts_with("https://"), "{}", model.name);
            }
            assert_eq!(
                model.detector.is_some(),
                model.pipeline == Pipeline::RtmPose
            );
            assert_eq!(model_named(model.name), Some(model));
        }
        assert!(model_named("huge").is_none());
    }
}
