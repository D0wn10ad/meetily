use async_trait::async_trait;
use log::{error, info, warn};
use std::path::Path;

use super::provider::{TranscriptResult, TranscriptionError, TranscriptionProvider};
use crate::sherpa_onnx::engine::SherpaOnnxEngine;

pub struct SherpaOnnxProvider {
    engine: SherpaOnnxEngine,
}

impl SherpaOnnxProvider {
    pub fn new(model_dir: &Path) -> Result<Self, crate::sherpa_onnx::SherpaOnnxError> {
        let engine = SherpaOnnxEngine::new(model_dir)?;
        Ok(SherpaOnnxProvider { engine })
    }
}

#[async_trait]
impl TranscriptionProvider for SherpaOnnxProvider {
    fn provider_name(&self) -> &'static str {
        "sherpa-onnx (SenseVoice)"
    }

    async fn is_model_loaded(&self) -> bool {
        true
    }

    async fn get_current_model(&self) -> Option<String> {
        Some("sensevoice".to_string())
    }

    async fn transcribe(
        &self,
        audio: Vec<f32>,
        _language: Option<String>,
    ) -> Result<TranscriptResult, TranscriptionError> {
        if audio.is_empty() {
            return Err(TranscriptionError::AudioTooShort {
                samples: 0,
                minimum: 1,
            });
        }
        info!("Sherpa-ONNX provider transcribing {} samples", audio.len());
        let text = self
            .engine
            .transcribe_audio(&audio, 16000)
            .map_err(|e| {
                error!("Sherpa-ONNX transcription failed: {}", e);
                TranscriptionError::EngineFailed(e.to_string())
            })?;
        info!(
            "Sherpa-ONNX provider transcription completed: '{}'",
            text
        );
        Ok(TranscriptResult {
            text: text.trim().to_string(),
            confidence: None,
            is_partial: false,
        })
    }
}
