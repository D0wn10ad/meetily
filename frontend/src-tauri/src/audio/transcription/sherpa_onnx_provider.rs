use async_trait::async_trait;
use log::{debug, error, info};
use std::path::Path;
use std::sync::Arc;

use super::provider::{TranscriptResult, TranscriptionError, TranscriptionProvider};
use crate::sherpa_onnx::engine::SherpaOnnxEngine;
use crate::sherpa_onnx::SherpaOnnxError;

pub struct SherpaOnnxProvider {
    engine: Arc<SherpaOnnxEngine>,
}

/// Map a structured [`SherpaOnnxError`] into the trait's flattened error
/// type while preserving the variant name and source chain through the
/// `Debug` representation (`TranscriptionError` has no variant able to
/// carry a `SherpaOnnxError`, and `provider.rs` is out of scope).
fn map_engine_error(e: &SherpaOnnxError) -> TranscriptionError {
    error!("Sherpa-ONNX transcription failed: {e:?}");
    TranscriptionError::EngineFailed(format!("{e:?}"))
}

impl SherpaOnnxProvider {
    pub fn new(model_dir: &Path) -> Result<Self, crate::sherpa_onnx::SherpaOnnxError> {
        let onnx_files = std::fs::read_dir(model_dir)
            .ok()
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().is_file()
                            && e.path().extension().map_or(false, |ext| ext == "onnx")
                    })
                    .count()
            })
            .unwrap_or(0);
        debug!(
            "Sherpa-ONNX provider new: model_dir={} exists={} tokens.txt={} onnx_files={}",
            model_dir.display(),
            model_dir.exists(),
            model_dir.join("tokens.txt").exists(),
            onnx_files
        );
        let engine = Arc::new(SherpaOnnxEngine::new(model_dir)?);
        debug!("Sherpa-ONNX provider new: engine constructed OK");
        Ok(SherpaOnnxProvider { engine })
    }
}

#[async_trait]
impl TranscriptionProvider for SherpaOnnxProvider {
    fn provider_name(&self) -> &'static str {
        "sherpa-onnx (SenseVoice)"
    }

    async fn is_model_loaded(&self) -> bool {
        self.engine.current_model().is_some()
    }

    async fn get_current_model(&self) -> Option<String> {
        self.engine.current_model()
    }

    async fn transcribe(
        &self,
        audio: Vec<f32>,
        _language: Option<String>,
    ) -> Result<TranscriptResult, TranscriptionError> {
        // NOTE: the by-value `Vec<f32>` parameter is a `TranscriptionProvider`
        // trait constraint (provider.rs, out of scope) — do not change it here.
        // The buffer is MOVED into the blocking task; no copy is made.
        if audio.is_empty() {
            return Err(TranscriptionError::AudioTooShort {
                samples: 0,
                minimum: 1,
            });
        }
        let sample_count = audio.len();
        info!("Sherpa-ONNX provider transcribing {} samples", sample_count);
        let started = std::time::Instant::now();
        let engine = Arc::clone(&self.engine);
        let text = tokio::task::spawn_blocking(move || engine.transcribe_audio(&audio, 16000))
            .await
            .map_err(|e| {
                TranscriptionError::EngineFailed(format!(
                    "Sherpa-ONNX inference task failed: {e}"
                ))
            })?
            .map_err(|e| map_engine_error(&e))?;
        // Never log the transcript itself (PII) — only a non-reversible size.
        info!(
            "Sherpa-ONNX provider transcription completed ({} chars)",
            text.chars().count()
        );
        debug!(
            "Sherpa-ONNX provider timing: {} samples -> {} chars in {} ms",
            sample_count,
            text.chars().count(),
            started.elapsed().as_millis()
        );
        Ok(TranscriptResult {
            text: text.trim().to_string(),
            confidence: None,
            is_partial: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _assert_send_sync<T: Send + Sync>() {}
    fn _assert() {
        _assert_send_sync::<SherpaOnnxEngine>();
    }

    #[test]
    fn test_map_engine_error_preserves_structured_variant() {
        let e = SherpaOnnxError::ModelNotFound {
            path: std::path::PathBuf::from("/tmp/x"),
        };
        let mapped = map_engine_error(&e);
        match mapped {
            TranscriptionError::EngineFailed(msg) => {
                assert!(
                    msg.contains("ModelNotFound"),
                    "message must contain the structured variant name, got: {}",
                    msg
                );
            }
            other => panic!("expected EngineFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_map_engine_error_preserves_source_chain() {
        let e = SherpaOnnxError::Io {
            path: Some(std::path::PathBuf::from("/tmp/x/tokens.txt")),
            kind: std::io::ErrorKind::NotFound,
            message: "missing tokens.txt".to_string(),
        };
        let mapped = map_engine_error(&e);
        match mapped {
            TranscriptionError::EngineFailed(msg) => {
                assert!(
                    msg.contains("Io"),
                    "message must contain the structured variant name, got: {}",
                    msg
                );
                assert!(
                    msg.contains("missing tokens.txt"),
                    "Debug output must contain the error detail, got: {}",
                    msg
                );
            }
            other => panic!("expected EngineFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_source_uses_spawn_blocking_and_no_direct_engine_call() {
        let src = include_str!("sherpa_onnx_provider.rs");
        assert!(
            src.contains("spawn_blocking"),
            "inference must run on the blocking thread pool"
        );
        // Split the needle so this assert does not match its own source text.
        let direct_call = concat!("self.engine.transcribe", "_audio(");
        assert!(
            !src.contains(direct_call),
            "no direct engine call outside the blocking closure"
        );
    }

    #[test]
    fn test_source_does_not_echo_transcript_at_info_level() {
        let src = include_str!("sherpa_onnx_provider.rs");
        // Split the needles so the asserts do not match their own source text.
        let transcript_echo = concat!("transcription completed: '{}", "'");
        assert!(
            !src.contains(transcript_echo),
            "the verbatim transcript must never be logged"
        );
        let size_summary = concat!("chars().count", "()");
        assert!(
            src.contains(size_summary),
            "the completion log must carry a non-reversible size instead"
        );
    }
}
