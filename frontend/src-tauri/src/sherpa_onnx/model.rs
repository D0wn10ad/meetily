//! Sherpa-ONNX model discovery and validation.

use std::path::{Path, PathBuf};

use crate::sherpa_onnx::SherpaOnnxError;

/// Status of a discovered model.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelStatus {
    /// Model has all required files and is ready for inference.
    Ready,
    /// Model is missing some required files.
    Missing { details: String },
}

/// Information about a discovered Sherpa-ONNX model.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Display name, e.g. "SherpaONNX-sensevoice"
    pub name: String,
    /// Model type identifier, e.g. "sensevoice"
    pub model_type: String,
    /// Full path to the model directory.
    pub path: PathBuf,
    /// Whether the model is ready for use.
    pub status: ModelStatus,
}

/// Required files for a SenseVoice model.
/// The `.onnx` file can have any name, so we check for its existence
/// separately via `find_onnx_model_file`.
const SENSEVOICE_REQUIRED_FILES: &[&str] = &["tokens.txt"];

/// Find the first `.onnx` file in a model directory.
pub fn find_onnx_model_file(model_dir: &Path) -> Result<PathBuf, SherpaOnnxError> {
    if !model_dir.exists() {
        return Err(SherpaOnnxError::ModelNotFound(model_dir.display().to_string()));
    }
    for entry in std::fs::read_dir(model_dir).map_err(SherpaOnnxError::from)? {
        let entry = entry.map_err(SherpaOnnxError::from)?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "onnx") {
            return Ok(path);
        }
    }
    Err(SherpaOnnxError::ModelNotFound(
        format!("No .onnx file found in {}", model_dir.display()),
    ))
}

/// Scan `models_dir` for valid Sherpa-ONNX model directories.
/// Each subdirectory that passes `validate_model_dir` is listed as Ready.
pub fn discover_models(models_dir: &Path) -> Vec<ModelInfo> {
    if !models_dir.exists() {
        return vec![];
    }

    let mut models = Vec::new();
    if let Ok(entries) = std::fs::read_dir(models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let model_type = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let name = format!("SherpaONNX-{}", &model_type);
                let status = match validate_model_dir(&path) {
                    Ok(()) => ModelStatus::Ready,
                    Err(ref e) => ModelStatus::Missing {
                        details: e.to_string(),
                    },
                };
                models.push(ModelInfo {
                    name,
                    model_type,
                    path,
                    status,
                });
            }
        }
    }
    models
}

/// Validate that a model directory has all required files.
/// For SenseVoice: tokens.txt and at least one `.onnx` file.
pub fn validate_model_dir(model_dir: &Path) -> Result<(), SherpaOnnxError> {
    if !model_dir.exists() {
        return Err(SherpaOnnxError::ModelNotFound(
            model_dir.display().to_string(),
        ));
    }
    for file in SENSEVOICE_REQUIRED_FILES {
        if !model_dir.join(file).exists() {
            return Err(SherpaOnnxError::ModelNotFound(
                model_dir.join(file).display().to_string(),
            ));
        }
    }
    // Also verify at least one .onnx file exists
    find_onnx_model_file(model_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_empty_dir() {
        let dir = std::env::temp_dir().join(format!("sherpa_disc_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let models = discover_models(&dir);
        assert!(models.is_empty(), "Empty dir should return empty list");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_missing_tokens() {
        let dir = std::env::temp_dir().join(format!("sherpa_val_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.int8.onnx"), "dummy").ok();
        let result = validate_model_dir(&dir);
        assert!(result.is_err(), "Missing tokens.txt should error");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_complete_dir() {
        let dir = std::env::temp_dir().join(format!("sherpa_comp_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.int8.onnx"), "dummy").ok();
        std::fs::write(dir.join("tokens.txt"), "a b c").ok();
        let result = validate_model_dir(&dir);
        assert!(result.is_ok(), "Complete dir should validate OK");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_onnx_model_file_found() {
        let dir = std::env::temp_dir().join(format!("sherpa_find_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.int8.onnx"), "dummy").ok();
        let result = find_onnx_model_file(&dir);
        assert!(result.is_ok(), "Should find .onnx file");
        assert_eq!(
            result.unwrap().file_name().unwrap(),
            "model.int8.onnx",
            "Should return the exact .onnx file"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_onnx_model_file_missing() {
        let dir = std::env::temp_dir().join(format!("sherpa_nofind_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("tokens.txt"), "a b c").ok();
        let result = find_onnx_model_file(&dir);
        assert!(result.is_err(), "No .onnx file should error");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
