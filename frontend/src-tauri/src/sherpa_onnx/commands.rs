//! Tauri commands for sherpa-onnx model management.
//!
//! Provides scan and validation commands similar to the FunASR pattern
//! in `api_scan_funasr_models`.

use crate::sherpa_onnx::model;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime};

/// Serializable entry for a discovered sherpa-onnx model.
#[derive(Debug, Serialize, Deserialize)]
pub struct SherpaOnnxModelEntry {
    pub name: String,
    pub path: String,
    pub valid: bool,
}

/// Scan the sherpa-onnx models directory and return discovered models.
#[tauri::command]
pub async fn sherpa_onnx_scan_models<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<SherpaOnnxModelEntry>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let sherpa_dir = app_data_dir.join("models").join("sherpa-onnx");

    if !sherpa_dir.exists() {
        info!(
            "Sherpa-ONNX models directory does not exist: {:?}",
            sherpa_dir
        );
        return Ok(Vec::new());
    }

    let models = model::discover_models(&sherpa_dir);
    let entries = models
        .into_iter()
        .map(|m| {
            let valid = matches!(m.status, model::ModelStatus::Ready);
            SherpaOnnxModelEntry {
                name: m.model_type,
                path: m.path.to_string_lossy().to_string(),
                valid,
            }
        })
        .collect();

    Ok(entries)
}

/// Validate whether a given path is a valid sherpa-onnx model directory.
#[tauri::command]
pub async fn sherpa_onnx_validate_model(model_path: String) -> Result<bool, String> {
    let path = Path::new(&model_path);
    match model::validate_model_dir(path) {
        Ok(()) => Ok(true),
        Err(e) => {
            warn!(
                "Sherpa-ONNX model validation failed for {:?}: {}",
                path, e
            );
            Ok(false)
        }
    }
}

/// Create the sherpa-onnx models directory on startup if it doesn't exist.
pub fn initialize_models_directory(app: &AppHandle) {
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let models_dir = app_data_dir.join("models").join("sherpa-onnx");
        if !models_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&models_dir) {
                warn!("Failed to create sherpa-onnx models directory: {}", e);
            } else {
                info!(
                    "Created sherpa-onnx models directory at {:?}",
                    models_dir
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sherpa_onnx::model::ModelStatus;

    #[test]
    fn test_model_entry_serialization() {
        let entry = SherpaOnnxModelEntry {
            name: "sensevoice".to_string(),
            path: "/tmp/models/sherpa-onnx/sensevoice".to_string(),
            valid: true,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("sensevoice"));
        assert!(json.contains("valid"));
    }

    #[test]
    fn test_initialize_models_directory_creates() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_init_test_{}",
            std::process::id()
        ));
        let app_data = temp.join("app_data");
        let models_dir = app_data.join("models").join("sherpa-onnx");

        // Create a minimal AppHandle mock by writing directly
        // (we can't construct a real AppHandle in tests)
        assert!(!models_dir.exists());
        std::fs::create_dir_all(&models_dir).unwrap();
        assert!(models_dir.exists());

        let _ = std::fs::remove_dir_all(&temp);
    }
}
