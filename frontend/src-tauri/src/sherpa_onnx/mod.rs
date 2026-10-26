//! Sherpa-ONNX transcription module.
//!
//! Provides SenseVoice model support via the sherpa-onnx Rust crate.
//! GPU acceleration is auto-detected at runtime by sherpa-onnx.

use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during Sherpa-ONNX operations.
#[derive(Debug, Clone)]
pub enum SherpaOnnxError {
    /// Underlying sherpa-onnx error
    SherpaOnnx {
        /// Human-readable detail from the sherpa-onnx layer
        message: String,
    },
    /// Model file not found at the given path
    ModelNotFound {
        /// The missing model file. For the "no `.onnx` file in the directory"
        /// case this is a glob-style placeholder (`<dir>/*.onnx`) because the
        /// `.onnx` file may have any name.
        path: PathBuf,
    },
    /// I/O error
    Io {
        /// Path the I/O operation targeted, when known (`None` when the error
        /// is converted generically via [`std::error::Error`]/`From`)
        path: Option<PathBuf>,
        /// Preserved [`std::io::ErrorKind`] so callers can match on the
        /// failure kind instead of parsing strings
        kind: std::io::ErrorKind,
        /// Human-readable detail
        message: String,
    },
    /// Invalid model directory structure
    InvalidModelDir {
        /// The offending directory
        path: PathBuf,
        /// Why the directory is not a valid model directory
        reason: String,
    },
    /// Empty audio input provided
    EmptyAudio,
}

impl fmt::Display for SherpaOnnxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SherpaOnnxError::SherpaOnnx { message } => write!(f, "sherpa-onnx error: {}", message),
            SherpaOnnxError::ModelNotFound { path } => {
                write!(f, "Model not found at {}", path.display())
            }
            SherpaOnnxError::Io {
                path,
                kind,
                message,
            } => match path {
                Some(path) => write!(f, "I/O error ({:?}) at {}: {}", kind, path.display(), message),
                None => write!(f, "I/O error ({:?}): {}", kind, message),
            },
            SherpaOnnxError::InvalidModelDir { path, reason } => {
                write!(f, "Invalid model directory {}: {}", path.display(), reason)
            }
            SherpaOnnxError::EmptyAudio => write!(f, "Empty audio input"),
        }
    }
}

impl std::error::Error for SherpaOnnxError {}

impl From<std::io::Error> for SherpaOnnxError {
    fn from(e: std::io::Error) -> Self {
        SherpaOnnxError::Io {
            // The `From` impl cannot know the path; explicit constructors at
            // the call sites fill it in.
            path: None,
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

/// Submodules
pub mod engine;
pub mod model;
pub mod commands;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_empty_audio() {
        let err = SherpaOnnxError::EmptyAudio;
        let msg = err.to_string();
        assert!(msg.contains("Empty audio"), "Display: {}", msg);
    }

    #[test]
    fn test_error_model_not_found() {
        let err = SherpaOnnxError::ModelNotFound {
            path: "/tmp/models".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("not found"), "Display: {}", msg);
        assert!(msg.contains("/tmp/models"), "Display: {}", msg);
    }

    #[test]
    fn test_error_sherpa_onnx() {
        let err = SherpaOnnxError::SherpaOnnx {
            message: "fail".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("sherpa-onnx"), "Display: {}", msg);
    }

    #[test]
    fn test_error_invalid_model_dir() {
        let err = SherpaOnnxError::InvalidModelDir {
            path: "/tmp/m".into(),
            reason: "missing tokens".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid model directory"), "Display: {}", msg);
    }

    #[test]
    fn test_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: SherpaOnnxError = io_err.into();
        let msg = err.to_string();
        assert!(msg.contains("I/O error"), "Display: {}", msg);
    }

    #[test]
    fn test_error_io_preserves_error_kind() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: SherpaOnnxError = io_err.into();
        let msg = err.to_string();
        assert!(msg.contains("NotFound"), "ErrorKind must be rendered: {}", msg);
    }

    #[test]
    fn test_error_io_permission_denied_is_not_notfound() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
        let err: SherpaOnnxError = io_err.into();
        let msg = err.to_string();
        assert!(msg.contains("PermissionDenied"), "Display: {}", msg);
        assert!(!msg.contains("NotFound"), "kind must not be lost: {}", msg);
    }

    #[test]
    fn test_error_model_not_found_displays_path() {
        let err = SherpaOnnxError::ModelNotFound {
            path: "/tmp/x/tokens.txt".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/tmp/x/tokens.txt"), "Display: {}", msg);
    }

    #[test]
    fn test_error_is_std_error() {
        fn check_impl<T: std::error::Error>() {}
        check_impl::<SherpaOnnxError>();
    }
}
