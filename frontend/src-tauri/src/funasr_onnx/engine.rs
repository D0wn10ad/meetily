use std::path::Path;

use tauri::Runtime;
use tauri::AppHandle;
use tauri::Manager;

use crate::funasr_onnx::{FunasrError, FunasrFrontend, FunasrModel};

pub fn initialize_models_directory<R: Runtime>(app: &AppHandle<R>) {
    let app_data_dir = app.path().app_data_dir()
        .expect("Failed to get app data dir");

    let models_dir = app_data_dir.join("models");

    if !models_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&models_dir) {
            log::error!("Failed to create FunASR models directory: {}", e);
            return;
        }
    }

    log::info!("FunASR models directory set to: {}", models_dir.display());
}

pub struct FunasrEngine {
    model: FunasrModel,
    frontend: FunasrFrontend,
}

impl FunasrEngine {
    pub fn new(model_dir: &Path) -> Result<Self, FunasrError> {
        let cmvn_path = model_dir.join("am.mvn");
        let frontend = FunasrFrontend::new(&cmvn_path)?;
        let model = FunasrModel::new(model_dir)?;
        log::info!("FunASR engine initialized with model from {}", model_dir.display());
        Ok(FunasrEngine { model, frontend })
    }

    pub fn transcribe_audio(&self, audio: &[f32]) -> Result<String, FunasrError> {
        log::info!("FunASR transcribing {} audio samples...", audio.len());
        let (features, feats_len) = self.frontend.process(audio)?;
        let (logits, token_num) = self.model.transcribe(&features, feats_len)?;
        let text =
            crate::funasr_onnx::decoder::decode(&logits, token_num, self.model.token_list())?;
        log::info!("FunASR transcription: '{}'", text);
        Ok(text)
    }

    pub fn provider_name(&self) -> &'static str {
        "FunASR (Paraformer)"
    }

    pub fn current_model(&self) -> Option<String> {
        Some("paraformer-large".to_string())
    }
}
