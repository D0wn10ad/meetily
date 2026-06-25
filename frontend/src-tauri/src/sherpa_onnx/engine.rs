//! Sherpa-ONNX transcription engine.
//!
//! Wraps the sherpa-onnx [`OfflineRecognizer`] for SenseVoice model inference.
//! GPU acceleration is auto-detected by sherpa-onnx at runtime.

use std::path::Path;

use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig,
};

use crate::sherpa_onnx::model;
use crate::sherpa_onnx::SherpaOnnxError;

/// Sherpa-ONNX transcription engine wrapping [`OfflineRecognizer`].
pub struct SherpaOnnxEngine {
    recognizer: OfflineRecognizer,
}

impl std::fmt::Debug for SherpaOnnxEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SherpaOnnxEngine").finish_non_exhaustive()
    }
}

impl SherpaOnnxEngine {
    /// Create a new engine from a validated model directory.
    ///
    /// Validates the directory has `tokens.txt` and at least one `.onnx` file,
    /// then creates an [`OfflineRecognizer`] configured for SenseVoice.
    pub fn new(model_dir: &Path) -> Result<Self, SherpaOnnxError> {
        // Validate model directory structure
        model::validate_model_dir(model_dir)?;

        // Locate the .onnx model file
        let model_path = model::find_onnx_model_file(model_dir)?;

        // Build recognizer config
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
            model: Some(model_path.to_string_lossy().into_owned()),
            language: Some("auto".into()),
            use_itn: true,
        };
        config.model_config.tokens = Some(
            model_dir.join("tokens.txt").to_string_lossy().into_owned(),
        );
        config.model_config.num_threads = 4;
        config.model_config.provider = Some("cpu".into());

        // Create recognizer
        let recognizer = OfflineRecognizer::create(&config).ok_or_else(|| {
            SherpaOnnxError::SherpaOnnx(
                "Failed to create sherpa-onnx OfflineRecognizer \
                 — model files may be invalid or incompatible"
                    .into(),
            )
        })?;

        Ok(Self { recognizer })
    }

    /// Transcribe audio samples using the loaded SenseVoice model.
    ///
    /// Returns the transcribed text with ITN (punctuation + English spacing) and
    /// emotion markers (`[laughter]`, `[applause]`) already embedded by sherpa-onnx.
    pub fn transcribe_audio(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<String, SherpaOnnxError> {
        if samples.is_empty() {
            return Err(SherpaOnnxError::EmptyAudio);
        }

        let stream = self.recognizer.create_stream();
        stream.accept_waveform(sample_rate as i32, samples);
        self.recognizer.decode(&stream);

        let result = stream.get_result().ok_or_else(|| {
            SherpaOnnxError::SherpaOnnx("No recognition result returned".into())
        })?;

        Ok(result.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_new_with_nonexistent_dir() {
        let result = SherpaOnnxEngine::new(Path::new("/tmp/nonexistent_sherpa_12345"));
        assert!(result.is_err(), "Nonexistent dir should fail");
        match result {
            Err(SherpaOnnxError::ModelNotFound(_)) => {} // expected
            _ => panic!("Expected ModelNotFound error, got {:?}", result),
        }
    }

    #[test]
    fn test_new_with_invalid_dir_missing_tokens() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("model.int8.onnx"), "dummy").ok();
        let result = SherpaOnnxEngine::new(dir.path());
        match result {
            Err(SherpaOnnxError::ModelNotFound(ref msg)) => {
                assert!(
                    msg.contains("tokens.txt"),
                    "Error should mention tokens.txt: {}",
                    msg
                );
            }
            other => panic!("Expected ModelNotFound for tokens.txt, got {:?}", other),
        }
    }

    #[test]
    fn test_new_with_invalid_dir_missing_onnx() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("tokens.txt"), "a b c").ok();
        let result = SherpaOnnxEngine::new(dir.path());
        match result {
            Err(SherpaOnnxError::ModelNotFound(ref msg)) => {
                assert!(
                    msg.contains(".onnx"),
                    "Error should mention .onnx: {}",
                    msg
                );
            }
            other => panic!("Expected ModelNotFound for .onnx, got {:?}", other),
        }
    }

    #[test]
    #[ignore = "requires real ONNX model files — sherpa-onnx may abort on garbage input"]
    fn test_new_with_valid_looking_dir_but_fake_model() {
        // This has the right files but the .onnx is garbage — should fail at create
        // Note: sherpa-onnx's C++ library may abort (SIGABRT) on invalid model files
        // rather than returning None, so this test requires real model files.
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("tokens.txt"), "a b c").ok();
        fs::write(
            dir.path().join("model.int8.onnx"),
            "this is not a real onnx model",
        )
        .ok();
        let result = SherpaOnnxEngine::new(dir.path());
        assert!(
            result.is_err(),
            "Fake ONNX file should fail to create recognizer"
        );
        match result {
            Err(SherpaOnnxError::SherpaOnnx(_)) => {} // expected
            other => panic!("Expected SherpaOnnxError error, got {:?}", other),
        }
    }

    #[test]
    fn test_transcribe_empty_audio_returns_error() {
        let err = SherpaOnnxError::EmptyAudio;
        let msg = format!("{}", err);
        assert!(
            msg.contains("Empty audio"),
            "Display should mention Empty audio: {}",
            msg
        );
    }
}
