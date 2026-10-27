//! Sherpa-ONNX transcription engine.
//!
//! Wraps the sherpa-onnx [`OfflineRecognizer`] for SenseVoice model inference.
//! The recognizer is configured with the CPU execution provider; GPU
//! acceleration is not enabled in this build.

use std::path::Path;

use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig,
};

use crate::sherpa_onnx::model;
use crate::sherpa_onnx::SherpaOnnxError;

/// Sherpa-ONNX transcription engine wrapping [`OfflineRecognizer`].
pub struct SherpaOnnxEngine {
    recognizer: OfflineRecognizer,
    model_name: Option<String>,
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
        let tokens_path = model_dir.join("tokens.txt");
        config.model_config.tokens = Some(tokens_path.to_string_lossy().into_owned());
        config.model_config.num_threads = 4;
        config.model_config.provider = Some("cpu".into());

        // Create recognizer
        let recognizer = OfflineRecognizer::create(&config).ok_or_else(|| {
            SherpaOnnxError::SherpaOnnx {
                message: recognizer_create_failure_message(&model_path, &tokens_path),
            }
        })?;

        Ok(Self {
            recognizer,
            model_name: resolve_model_name(model_dir),
        })
    }

    /// The resolved model directory name, when the engine was constructed.
    pub fn current_model(&self) -> Option<String> {
        self.model_name.clone()
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
        let sample_rate = validate_transcribe_inputs(samples, sample_rate)?;

        // A fresh stream per call is required by the sherpa-onnx API: offline
        // streams expose no reset/flush (offline_asr.rs offers only create_stream,
        // accept_waveform, decode, get_result), so reuse would bleed audio
        // between calls.
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(sample_rate, samples);
        self.recognizer.decode(&stream);

        let result = stream.get_result().ok_or_else(|| {
            SherpaOnnxError::SherpaOnnx {
                message: "No recognition result returned".into(),
            }
        })?;

        Ok(result.text)
    }
}

/// Validate transcription inputs and convert the sample rate for the
/// sherpa-onnx API, which takes an `i32`.
fn validate_transcribe_inputs(
    samples: &[f32],
    sample_rate: u32,
) -> Result<i32, SherpaOnnxError> {
    if samples.is_empty() {
        return Err(SherpaOnnxError::EmptyAudio);
    }
    if sample_rate == 0 {
        return Err(SherpaOnnxError::InvalidSampleRate { sample_rate });
    }
    i32::try_from(sample_rate).map_err(|_| SherpaOnnxError::InvalidSampleRate { sample_rate })
}

/// Build the enriched failure message for `OfflineRecognizer::create`.
fn recognizer_create_failure_message(model_path: &Path, tokens_path: &Path) -> String {
    format!(
        "failed to create OfflineRecognizer (model={}, tokens={}, provider=cpu, \
         num_threads=4): recognizer creation returned None — model files may be \
         invalid or incompatible",
        model_path.display(),
        tokens_path.display()
    )
}

/// Resolve the model directory name for reporting.
fn resolve_model_name(dir: &Path) -> Option<String> {
    dir.file_name().and_then(|n| n.to_str()).map(str::to_string)
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
            Err(SherpaOnnxError::ModelNotFound { .. }) => {} // expected
            _ => panic!("Expected ModelNotFound error, got {:?}", result),
        }
    }

    #[test]
    fn test_new_with_invalid_dir_missing_tokens() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("model.int8.onnx"), "dummy").ok();
        let result = SherpaOnnxEngine::new(dir.path());
        match result {
            Err(e @ SherpaOnnxError::ModelNotFound { .. }) => {
                let msg = e.to_string();
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
            Err(e @ SherpaOnnxError::ModelNotFound { .. }) => {
                let msg = e.to_string();
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
            Err(SherpaOnnxError::SherpaOnnx { .. }) => {} // expected
            other => panic!("Expected SherpaOnnxError error, got {:?}", other),
        }
    }

    #[test]
    fn test_transcribe_empty_audio_returns_error() {
        let err = SherpaOnnxError::EmptyAudio;
        let msg = err.to_string();
        assert!(
            msg.contains("Empty audio"),
            "Display should mention Empty audio: {}",
            msg
        );
    }

    // T16: the transcribe-input validator is the honest testable path — a real
    // OfflineRecognizer cannot be constructed in CI without valid ONNX files.
    #[test]
    fn test_validate_transcribe_inputs_rejects_empty() {
        let err = validate_transcribe_inputs(&[], 16000).unwrap_err();
        assert!(matches!(err, SherpaOnnxError::EmptyAudio));
    }

    #[test]
    fn test_validate_transcribe_inputs_rejects_zero_sample_rate() {
        let err = validate_transcribe_inputs(&[0.0], 0).unwrap_err();
        assert!(
            matches!(err, SherpaOnnxError::InvalidSampleRate { sample_rate: 0 }),
            "zero sample rate must be rejected, got {:?}",
            err
        );
    }

    #[test]
    fn test_validate_transcribe_inputs_rejects_out_of_range_sample_rate() {
        // u32::MAX does not fit in i32 — must be rejected, never truncated by `as`.
        let err = validate_transcribe_inputs(&[0.0], u32::MAX).unwrap_err();
        assert!(
            matches!(err, SherpaOnnxError::InvalidSampleRate { sample_rate: u32::MAX }),
            "out-of-range sample rate must be rejected, got {:?}",
            err
        );
    }

    #[test]
    fn test_validate_transcribe_inputs_accepts_normal_rate() {
        assert_eq!(validate_transcribe_inputs(&[0.0, 1.0], 16000).unwrap(), 16000);
    }

    #[test]
    fn test_recognizer_create_failure_message_names_model_and_tokens() {
        let msg = recognizer_create_failure_message(
            Path::new("/models/sense-voice/model.int8.onnx"),
            Path::new("/models/sense-voice/tokens.txt"),
        );
        assert!(
            msg.contains("model.int8.onnx"),
            "message should name the model path: {msg}"
        );
        assert!(
            msg.contains("tokens.txt"),
            "message should name the tokens path: {msg}"
        );
        assert!(msg.contains("cpu"), "message should name the provider: {msg}");
    }

    #[test]
    fn test_resolve_model_name_returns_dir_name() {
        let dir = tempfile::tempdir().unwrap();
        let expected = dir
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .to_string();
        assert_eq!(resolve_model_name(dir.path()), Some(expected));
    }

    #[test]
    fn test_resolve_model_name_none_for_root() {
        assert_eq!(resolve_model_name(Path::new("/")), None);
    }
}
