//! Tauri commands for sherpa-onnx model management.
//!
//! Provides scan and validation commands similar to the FunASR pattern
//! in `api_scan_funasr_models`.
//!
//! The recursive model-directory scan runs on the blocking thread pool
//! (`tokio::task::spawn_blocking`) so it never stalls the async runtime,
//! and `sherpa_onnx_validate_model` only accepts paths that resolve
//! inside the application's sherpa-onnx models directory.

use crate::sherpa_onnx::model;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};
use tokio::task::spawn_blocking;

/// Serializable entry for a discovered sherpa-onnx model.
#[derive(Debug, Serialize, Deserialize)]
pub struct SherpaOnnxModelEntry {
    pub name: String,
    pub path: String,
    pub valid: bool,
    pub size_mb: f64,
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

    // The discovery walk and the per-model size computation are blocking
    // filesystem work — run them off the async runtime thread.
    let entries = spawn_blocking(move || scan_models_blocking(&sherpa_dir))
        .await
        .map_err(|e| format!("Sherpa-ONNX model scan task failed: {}", e))?;

    Ok(entries)
}

/// Blocking implementation of the model scan: discovery + size computation.
///
/// Must be invoked through `spawn_blocking` — it performs a recursive
/// directory walk.
fn scan_models_blocking(sherpa_dir: &Path) -> Vec<SherpaOnnxModelEntry> {
    let models = model::discover_models(sherpa_dir);
    models
        .into_iter()
        .map(|m| {
            let valid = matches!(m.status, model::ModelStatus::Ready);
            // `directory_size_mb` does not panic (it returns 0.0 for
            // unreadable paths), but a path that vanished between discovery
            // and sizing is worth surfacing explicitly instead of silently
            // reporting `size_mb: 0.0`.
            let size_mb = if m.path.exists() {
                crate::directory_size_mb(&m.path)
            } else {
                warn!(
                    "Sherpa-ONNX model path vanished during scan: {:?}",
                    m.path
                );
                0.0
            };
            SherpaOnnxModelEntry {
                name: m.model_type,
                path: m.path.to_string_lossy().into_owned(),
                valid,
                size_mb,
            }
        })
        .collect()
}

/// Validate whether a given path is a valid sherpa-onnx model directory.
///
/// The path must resolve inside the application's sherpa-onnx models
/// directory (`app_data_dir/models/sherpa-onnx`); anything else is rejected
/// as a path-traversal attempt before validation runs.
#[tauri::command]
pub async fn sherpa_onnx_validate_model<R: Runtime>(
    app: AppHandle<R>,
    model_path: String,
) -> Result<bool, String> {
    let models_root = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("models")
        .join("sherpa-onnx");

    validate_model_path(&model_path, &models_root)
}

/// Validate `model_path` for containment within `models_root`, then run the
/// sherpa-onnx model-directory validation.
///
/// Rejects explicit `..` components before any filesystem resolution and
/// rejects any path that does not stay inside the canonicalized models root
/// (symlinks and relative segments cannot escape it). On a validation
/// failure the underlying error is preserved in the returned `Err` so the
/// caller can distinguish "could not check / not a valid model" from a
/// successful verdict.
pub(crate) fn validate_model_path(model_path: &str, models_root: &Path) -> Result<bool, String> {
    let candidate = Path::new(model_path);

    // Reject explicit `..` components before any filesystem resolution.
    if candidate
        .components()
        .any(|c| c == Component::ParentDir)
    {
        return Err(format!(
            "Rejected path-traversal attempt ('..' component): {:?}",
            model_path
        ));
    }

    // Canonicalize both sides so symlinks and relative segments cannot
    // escape the models root. A non-existent candidate falls back to its
    // literal form; the validation below fails for it anyway.
    let canonical_root: PathBuf = std::fs::canonicalize(models_root)
        .unwrap_or_else(|_| models_root.to_path_buf());
    let canonical_candidate: PathBuf = std::fs::canonicalize(candidate)
        .unwrap_or_else(|_| candidate.to_path_buf());

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!(
            "Rejected path outside the sherpa-onnx models root {:?}: {:?}",
            canonical_root, model_path
        ));
    }

    match model::validate_model_dir(&canonical_candidate) {
        Ok(()) => Ok(true),
        Err(e) => {
            warn!(
                "Sherpa-ONNX model validation failed for {:?}: {}",
                canonical_candidate, e
            );
            Err(format!(
                "Sherpa-ONNX model validation failed for {:?}: {}",
                canonical_candidate, e
            ))
        }
    }
}

/// Create the sherpa-onnx models directory on startup if it doesn't exist.
pub fn initialize_models_directory(app: &AppHandle) {
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let models_dir = app_data_dir.join("models").join("sherpa-onnx");
        match ensure_models_dir(&models_dir) {
            Ok(()) => {
                info!(
                    "Ensured sherpa-onnx models directory at {:?}",
                    models_dir
                );
            }
            Err(e) => {
                warn!("Failed to create sherpa-onnx models directory: {}", e);
            }
        }
    }
}

/// Pure filesystem logic of [`initialize_models_directory`]: create the
/// sherpa-onnx models directory (and any missing parents) if it does not
/// exist yet. Idempotent — returns `Ok` when the directory already exists.
pub(crate) fn ensure_models_dir(root: &Path) -> std::io::Result<()> {
    if root.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_entry_serialization() {
        let entry = SherpaOnnxModelEntry {
            name: "sensevoice".to_string(),
            path: "/tmp/models/sherpa-onnx/sensevoice".to_string(),
            valid: true,
            size_mb: 123.4,
        };
        let json = serde_json::to_string(&entry).unwrap();
        // The frontend depends on these exact field names.
        assert!(json.contains("\"name\""), "missing field name: {}", json);
        assert!(json.contains("\"path\""), "missing field path: {}", json);
        assert!(json.contains("\"valid\""), "missing field valid: {}", json);
        assert!(
            json.contains("\"size_mb\""),
            "missing field size_mb: {}",
            json
        );
        assert!(json.contains("sensevoice"));
    }

    #[test]
    fn test_ensure_models_dir_creates_fresh_dir() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_ensure_fresh_{}",
            std::process::id()
        ));
        let models_dir = temp.join("app_data").join("models").join("sherpa-onnx");
        let _ = std::fs::remove_dir_all(&temp);

        assert!(!models_dir.exists());
        ensure_models_dir(&models_dir).expect("ensure_models_dir should create the dir");
        assert!(models_dir.exists(), "models dir must exist after the call");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_ensure_models_dir_ok_when_already_exists() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_ensure_exists_{}",
            std::process::id()
        ));
        let models_dir = temp.join("models").join("sherpa-onnx");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&models_dir).unwrap();

        ensure_models_dir(&models_dir)
            .expect("ensure_models_dir must be idempotent for an existing dir");
        assert!(models_dir.exists());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_validate_model_path_rejects_traversal() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_traversal_{}",
            std::process::id()
        ));
        let models_root = temp.join("models").join("sherpa-onnx");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&models_root).unwrap();

        let traversal = format!("{}/../../etc", models_root.display());
        let result = validate_model_path(&traversal, &models_root);
        match result {
            Err(msg) => {
                assert!(
                    msg.to_lowercase().contains("traversal"),
                    "error must name the traversal attempt: {}",
                    msg
                );
            }
            Ok(_) => panic!("traversal path must be rejected, got Ok"),
        }

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_validate_model_path_rejects_outside_root() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_outside_{}",
            std::process::id()
        ));
        let models_root = temp.join("models").join("sherpa-onnx");
        let outside = temp.join("elsewhere").join("model");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&models_root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        // Make the outside dir a *valid* model so only the containment
        // guard can be responsible for the rejection.
        std::fs::write(outside.join("tokens.txt"), "a b c").unwrap();
        std::fs::write(outside.join("model.int8.onnx"), "dummy").unwrap();

        let result = validate_model_path(&outside.display().to_string(), &models_root);
        match result {
            Err(msg) => {
                assert!(
                    msg.contains("outside the sherpa-onnx models root"),
                    "error must name the containment violation: {}",
                    msg
                );
            }
            Ok(_) => panic!("path outside the models root must be rejected, got Ok"),
        }

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_validate_model_path_accepts_valid_model_inside_root() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_inside_{}",
            std::process::id()
        ));
        let models_root = temp.join("models").join("sherpa-onnx");
        let model_dir = models_root.join("sensevoice");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("tokens.txt"), "a b c").unwrap();
        std::fs::write(model_dir.join("model.int8.onnx"), "dummy").unwrap();

        let result = validate_model_path(&model_dir.display().to_string(), &models_root);
        assert_eq!(result, Ok(true), "valid model inside the root must validate");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_validate_model_path_reports_invalid_model_as_err() {
        let temp = std::env::temp_dir().join(format!(
            "sherpa_invalid_{}",
            std::process::id()
        ));
        let models_root = temp.join("models").join("sherpa-onnx");
        let model_dir = models_root.join("broken");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&model_dir).unwrap();
        // Missing tokens.txt -> not a valid model.

        let result = validate_model_path(&model_dir.display().to_string(), &models_root);
        match result {
            Err(msg) => {
                assert!(
                    msg.contains("validation failed"),
                    "error must preserve the underlying validation failure: {}",
                    msg
                );
            }
            Ok(_) => panic!("an invalid model dir must not validate, got Ok"),
        }

        let _ = std::fs::remove_dir_all(&temp);
    }
}
