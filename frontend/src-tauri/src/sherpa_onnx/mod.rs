//! Sherpa-ONNX transcription module.
//!
//! Provides SenseVoice model support via the sherpa-onnx Rust crate.
//! GPU acceleration is auto-detected at runtime by sherpa-onnx.

use std::fmt;

/// Errors that can occur during Sherpa-ONNX operations.
#[derive(Debug, Clone)]
pub enum SherpaOnnxError {
    /// Underlying sherpa-onnx error
    SherpaOnnx(String),
    /// Model file not found at the given path
    ModelNotFound(String),
    /// I/O error
    Io(String),
    /// Invalid model directory structure
    InvalidModelDir(String),
    /// Empty audio input provided
    EmptyAudio,
}

impl fmt::Display for SherpaOnnxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SherpaOnnxError::SherpaOnnx(msg) => write!(f, "sherpa-onnx error: {}", msg),
            SherpaOnnxError::ModelNotFound(path) => write!(f, "Model not found at {}", path),
            SherpaOnnxError::Io(msg) => write!(f, "I/O error: {}", msg),
            SherpaOnnxError::InvalidModelDir(msg) => write!(f, "Invalid model directory: {}", msg),
            SherpaOnnxError::EmptyAudio => write!(f, "Empty audio input"),
        }
    }
}

impl std::error::Error for SherpaOnnxError {}

impl From<std::io::Error> for SherpaOnnxError {
    fn from(e: std::io::Error) -> Self {
        SherpaOnnxError::Io(e.to_string())
    }
}

/// Re-exports for submodules (will be uncommented as modules are created)
// pub mod model;
// pub mod engine;
// pub mod commands;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_empty_audio() {
        let err = SherpaOnnxError::EmptyAudio;
        let msg = format!("{}", err);
        assert!(msg.contains("Empty audio"), "Display: {}", msg);
    }

    #[test]
    fn test_error_model_not_found() {
        let err = SherpaOnnxError::ModelNotFound("/tmp/models".into());
        let msg = format!("{}", err);
        assert!(msg.contains("not found"), "Display: {}", msg);
    }

    #[test]
    fn test_error_sherpa_onnx() {
        let err = SherpaOnnxError::SherpaOnnx("fail".into());
        let msg = format!("{}", err);
        assert!(msg.contains("sherpa-onnx"), "Display: {}", msg);
    }

    #[test]
    fn test_error_invalid_model_dir() {
        let err = SherpaOnnxError::InvalidModelDir("missing tokens".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid model directory"), "Display: {}", msg);
    }

    #[test]
    fn test_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: SherpaOnnxError = io_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("I/O error"), "Display: {}", msg);
    }

    #[test]
    fn test_error_is_std_error() {
        fn check_impl<T: std::error::Error>() {}
        check_impl::<SherpaOnnxError>();
    }
}
