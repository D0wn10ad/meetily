use std::path::Path;

use crate::funasr_onnx::{FunasrError, FunasrFrontend, FunasrModel};

pub struct FunasrEngine {
    model: FunasrModel,
    frontend: FunasrFrontend,
}

impl FunasrEngine {
    pub fn new(model_dir: &Path) -> Result<Self, FunasrError> {
        let cmvn_path = model_dir.join("am.mvn");
        let frontend = FunasrFrontend::new(&cmvn_path)?;
        let model = FunasrModel::new(model_dir)?;
        Ok(FunasrEngine { model, frontend })
    }

    pub fn transcribe_audio(&self, audio: &[f32]) -> Result<String, FunasrError> {
        let (features, feats_len) = self.frontend.process(audio)?;
        let (logits, token_num) = self.model.transcribe(&features, feats_len)?;
        let text =
            crate::funasr_onnx::decoder::decode(&logits, token_num, self.model.token_list())?;
        Ok(text)
    }

    pub fn provider_name(&self) -> &'static str {
        "FunASR (Paraformer)"
    }

    pub fn current_model(&self) -> Option<String> {
        Some("paraformer-large".to_string())
    }
}
