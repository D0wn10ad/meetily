//! FunASR ONNX speech-recognition engine.
//!
//! # Public surface
//!
//! The module is organised around one lifecycle: load a model once, then
//! transcribe many utterances.
//!
//! * [`FunasrEngine`] — high-level engine. `FunasrEngine::new(model_dir)` loads
//!   `model.onnx`, `am.mvn`, and `tokens.json` from a model directory;
//!   `transcribe` / `transcribe_file` turn 16 kHz mono `f32` PCM into text.
//! * [`FunasrFrontend`] — log-mel fbank + LFR + CMVN feature extraction.
//! * [`FunasrModel`] — ONNX Runtime session wrapper used by the engine.
//! * [`decode`] — token-logit decoder used by the engine.
//! * [`commands`] — Tauri commands that download and cancel the FunASR model.
//!
//! # Error contract
//!
//! Every fallible entry point returns [`FunasrError`]. The type:
//!
//! * implements [`std::error::Error`] and is `Send + Sync`, so it can cross
//!   `.await` points and be returned from `spawn_blocking`;
//! * preserves the underlying failure through [`std::error::Error::source`]
//!   where one exists (ONNX Runtime, I/O, JSON, UTF-8, float parsing);
//! * carries file locations as [`std::path::PathBuf`], never pre-rendered
//!   strings, so callers can match on the path.
//!
//! Variants are intentionally not `Clone`/`PartialEq`: they wrap non-cloneable
//! runtime errors, and error equality is not part of the contract.

use thiserror::Error;

/// Errors produced by the FunASR ONNX module.
///
/// See the [module documentation](self) for the error contract.
#[derive(Error, Debug)]
pub enum FunasrError {
    /// ONNX Runtime failure, propagated unchanged.
    #[error("ONNX Runtime error: {0}")]
    Ort(#[from] ort::Error),

    /// The model file does not exist at the given path.
    #[error("Model not found at {0:?}")]
    ModelNotFound(std::path::PathBuf),

    /// The token file is missing or unusable.
    #[error("Failed to load token file {path:?}: {source}")]
    InvalidTokenFile {
        /// Path to the token file that failed.
        path: std::path::PathBuf,
        /// Underlying I/O failure.
        #[source]
        source: std::io::Error,
    },

    /// Filesystem failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Feature or token decoding failed.
    #[error("Decode error: {message}")]
    DecodeError {
        /// Human-readable description of the decoding failure.
        message: String,
        /// Underlying failure, when the error originated in another layer.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },

    /// Audio is shorter than one analysis frame.
    #[error("Audio too short: need at least {expected} samples, got {got}")]
    AudioTooShort {
        /// Minimum number of samples required.
        expected: usize,
        /// Number of samples actually supplied.
        got: usize,
    },

    /// Empty audio input.
    #[error("Empty audio input")]
    EmptyAudio,

    /// The CMVN file is malformed.
    #[error("Invalid CMVN file {path:?}: {message}")]
    InvalidCmvnFile {
        /// Path to the CMVN file that failed to parse.
        path: std::path::PathBuf,
        /// Description of the parse failure.
        message: String,
        /// Underlying failure, when one exists.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },

    /// JSON deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub mod frontend;
pub use frontend::FunasrFrontend;

pub mod model;
pub use model::FunasrModel;

pub mod decoder;
pub use decoder::decode;

pub mod engine;
pub use engine::FunasrEngine;

pub mod commands;
pub use commands::{funasr_cancel_download, funasr_download_model, DownloadProgress};

#[cfg(test)]
mod tests {
    use super::FunasrError;
    use std::path::PathBuf;

    /// Compile-time proof that the public error can cross thread boundaries.
    ///
    /// This is the compile-time counterpart to the plan item
    /// `#[derive(Debug, Send, Sync)]`; neither `Send` nor `Sync` is a derivable
    /// trait, so the bounds are enforced here instead. Depends only on the
    /// auto-trait implementations of the field types.
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn funasr_error_is_send_and_sync() {
        assert_send_sync::<FunasrError>();
    }

    #[test]
    fn decode_error_preserves_source_chain() {
        let inner = std::io::Error::new(std::io::ErrorKind::InvalidData, "inner failure");
        let err = FunasrError::DecodeError {
            message: "could not decode".to_string(),
            source: Some(Box::new(inner)),
        };

        assert_eq!(err.to_string(), "Decode error: could not decode");
        let source = std::error::Error::source(&err).expect("source must be preserved");
        assert_eq!(source.to_string(), "inner failure");
    }

    #[test]
    fn decode_error_without_source_reports_none() {
        let err = FunasrError::DecodeError {
            message: "missing token_num output".to_string(),
            source: None,
        };
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn variant_without_source_reports_none() {
        let err = FunasrError::EmptyAudio;
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn path_payloads_are_path_bufs() {
        // Compile-time proof of the payload types plus runtime round-trip.
        let path = PathBuf::from("/tmp/model.onnx");
        let err = FunasrError::ModelNotFound(path.clone());
        match err {
            FunasrError::ModelNotFound(p) => assert_eq!(p, path),
            other => panic!("unexpected variant: {other:?}"),
        }

        let err = FunasrError::InvalidCmvnFile {
            path: PathBuf::from("/tmp/am.mvn"),
            message: "bad".to_string(),
            source: Some(Box::new(std::io::Error::other("cause"))),
        };
        assert!(std::error::Error::source(&err).is_some());
    }
}
