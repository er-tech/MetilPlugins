use metilplugins_model_cache::ModelFile;

/// YAMNet, Google's tagger of the 521 sounds of AudioSet (MobileNet v1 on a log-mel spectrogram,
/// Apache-2.0), as an ONNX file converted from the original by someone else. It takes the raw
/// waveform at 16 kHz (the mel front end is inside) and gives, for each frame of about a second,
/// a score from 0 to 1 for each class. Nothing is packaged: it is downloaded once and checked
/// against this sum.
pub const YAMNET: ModelFile = ModelFile {
    file: "yamnet.onnx",
    url: "https://huggingface.co/zeropointnine/yamnet-onnx/resolve/main/yamnet.onnx",
    sha256: "1510041dce24a2e9e84ec546807ac408ae496da6d1ed41bc3ccba649623f8e19",
    bytes: 16_093_603,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_model_has_a_checksum_and_a_url_that_ends_with_its_file() {
        assert_eq!(YAMNET.sha256.len(), 64);
        assert!(YAMNET.url.ends_with(YAMNET.file));
    }
}
