use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::path::Path;
use std::sync::Arc;

use super::provider::{TranscriptResult, TranscriptionError, TranscriptionProvider};
use crate::funasr_onnx::FunasrEngine;
use crate::funasr_onnx::FunasrError;

pub struct FunasrProvider {
    engine: Arc<FunasrEngine>,
    model_name: String,
}

/// Derive the stored model name from the model directory.
fn derived_model_name(model_dir: &Path) -> String {
    model_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Pure derivation of `is_model_loaded` from the stored model name.
fn model_loaded_from_name(model_name: &str) -> bool {
    !model_name.is_empty()
}

/// Map a structured [`FunasrError`] into the trait's flattened error type
/// while preserving the variant name and source chain through the `Debug`
/// representation (`TranscriptionError` has no variant able to carry a
/// `FunasrError`, and `provider.rs` is out of scope).
fn map_engine_error(e: &FunasrError) -> TranscriptionError {
    error!("FunASR transcription failed: {e:?}");
    TranscriptionError::EngineFailed(format!("{e:?}"))
}

impl FunasrProvider {
    pub fn new(model_dir: &Path) -> Result<Self, crate::funasr_onnx::FunasrError> {
        let engine = Arc::new(FunasrEngine::new(model_dir)?);
        let model_name = derived_model_name(model_dir);
        Ok(FunasrProvider { engine, model_name })
    }
}

#[async_trait]
impl TranscriptionProvider for FunasrProvider {
    fn provider_name(&self) -> &'static str {
        self.engine.provider_name()
    }

    async fn is_model_loaded(&self) -> bool {
        model_loaded_from_name(&self.model_name)
    }

    async fn get_current_model(&self) -> Option<String> {
        Some(self.model_name.clone())
    }

    async fn transcribe(
        &self,
        audio: Vec<f32>,
        _language: Option<String>,
    ) -> Result<TranscriptResult, TranscriptionError> {
        // NOTE: the by-value `Vec<f32>` parameter is a `TranscriptionProvider`
        // trait constraint (provider.rs, out of scope) — do not change it here.
        // The buffer is MOVED into the blocking task; no additional copy is made.
        if let Some(ref lang) = _language {
            warn!("FunASR (Paraformer) defaults to Chinese, ignoring language preference '{}'", lang);
        }
        info!("FunASR provider transcribing {} samples", audio.len());
        let engine = Arc::clone(&self.engine);
        let text = tokio::task::spawn_blocking(move || engine.transcribe_audio(&audio))
            .await
            .map_err(|e| {
                TranscriptionError::EngineFailed(format!("FunASR inference task failed: {e}"))
            })?
            .map_err(|e| map_engine_error(&e))?;
        debug!("FunASR provider transcription completed: '{}'", text);
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
    use crate::funasr_onnx::FunasrError;

    fn _assert_send_sync<T: Send + Sync>() {}
    fn _assert() {
        _assert_send_sync::<FunasrEngine>();
    }

    #[test]
    fn test_map_engine_error_preserves_structured_variant() {
        let e = FunasrError::ModelNotFound(std::path::PathBuf::from("/tmp/nope"));
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
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "missing tokens");
        let e = FunasrError::InvalidTokenFile {
            path: std::path::PathBuf::from("/tmp/tokens"),
            source,
        };
        let mapped = map_engine_error(&e);
        match mapped {
            TranscriptionError::EngineFailed(msg) => {
                assert!(
                    msg.contains("InvalidTokenFile"),
                    "message must contain the structured variant name, got: {}",
                    msg
                );
                assert!(
                    msg.contains("missing tokens"),
                    "Debug output must contain the source chain, got: {}",
                    msg
                );
            }
            other => panic!("expected EngineFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_model_loaded_from_name() {
        assert!(!model_loaded_from_name(""));
        assert!(model_loaded_from_name("paraformer-large"));
    }

    #[test]
    fn test_derived_model_name_from_dir() {
        assert_eq!(
            derived_model_name(Path::new("/data/models/funasr/paraformer-large")),
            "paraformer-large"
        );
        assert_eq!(
            derived_model_name(Path::new("paraformer-large")),
            "paraformer-large"
        );
        assert_eq!(derived_model_name(Path::new("/")), "");
    }

    #[test]
    fn test_source_uses_spawn_blocking_and_no_direct_engine_call() {
        let src = include_str!("funasr_provider.rs");
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
}
