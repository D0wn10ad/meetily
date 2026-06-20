use async_trait::async_trait;
use std::path::Path;

use super::provider::{TranscriptResult, TranscriptionError, TranscriptionProvider};
use crate::funasr_onnx::FunasrEngine;

pub struct FunasrProvider {
    engine: FunasrEngine,
}

impl FunasrProvider {
    pub fn new(model_dir: &Path) -> Result<Self, crate::funasr_onnx::FunasrError> {
        let engine = FunasrEngine::new(model_dir)?;
        Ok(FunasrProvider { engine })
    }
}

#[async_trait]
impl TranscriptionProvider for FunasrProvider {
    fn provider_name(&self) -> &'static str {
        self.engine.provider_name()
    }

    async fn is_model_loaded(&self) -> bool {
        true
    }

    async fn get_current_model(&self) -> Option<String> {
        self.engine.current_model()
    }

    async fn transcribe(
        &self,
        audio: Vec<f32>,
        _language: Option<String>,
    ) -> Result<TranscriptResult, TranscriptionError> {
        let text = self
            .engine
            .transcribe_audio(&audio)
            .map_err(|e| TranscriptionError::EngineFailed(e.to_string()))?;
        Ok(TranscriptResult {
            text: text.trim().to_string(),
            confidence: None,
            is_partial: false,
        })
    }
}
