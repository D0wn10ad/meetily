use thiserror::Error;

/// Errors that can occur in the FunASR ONNX module.
#[derive(Error, Debug)]
pub enum FunasrError {
    #[error("ONNX Runtime error: {0}")]
    Ort(#[from] ort::Error),
    #[error("Model not found at {0}")]
    ModelNotFound(String),
    #[error("Failed to load token file: {0}")]
    InvalidTokenFile(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Audio too short: need at least {min} samples, got {actual}")]
    AudioTooShort { min: usize, actual: usize },
    #[error("Empty audio input")]
    EmptyAudio,
    #[error("Invalid CMVN file: {0}")]
    InvalidCmvnFile(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub mod frontend;
pub use frontend::FunasrFrontend;

pub mod model;
pub use model::FunasrModel;

pub mod decoder;
pub use decoder::*;

pub mod engine;
pub use engine::FunasrEngine;

pub mod commands;
pub use commands::*;
