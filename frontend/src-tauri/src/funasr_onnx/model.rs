use std::path::Path;
use std::sync::Mutex;

use ort::inputs;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;

use crate::funasr_onnx::FunasrError;

/// Wraps a Paraformer ONNX model for inference.
///
/// Expects a directory containing:
/// - `model.onnx` — the exported Paraformer model
/// - `tokens.json` — JSON array of token strings
pub struct FunasrModel {
    session: Mutex<Session>,
    token_list: Vec<String>,
}

impl FunasrModel {
    /// Access the token list (vocabulary) for decoding.
    pub fn token_list(&self) -> &[String] {
        &self.token_list
    }
}

impl FunasrModel {
    /// Load model from directory containing `model.onnx` and `tokens.json`.
    pub fn new(model_dir: &Path) -> Result<Self, FunasrError> {
        let model_path = model_dir.join("model.onnx");
        if !model_path.exists() {
            return Err(FunasrError::ModelNotFound(model_path.display().to_string()));
        }

        log::info!("Loading FunASR Paraformer model from {}...", model_path.display());

        let gpu_type = crate::audio::hardware_detector::HardwareProfile::detect().gpu_type;
        let providers = crate::audio::onnx_provider::get_onnx_providers(gpu_type);
        for p in &providers {
            log::info!("  └─ EP: {:?}", p);
        }
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_execution_providers(providers)?
            .with_parallel_execution(true)?
            .commit_from_file(&model_path)?;

        log::info!("FunASR model loaded successfully from {}", model_path.display());

        // Log input/output shapes for debugging
        for input in &session.inputs {
            log::info!(
                "FunASR model input: name={}, type={:?}",
                input.name,
                input.input_type
            );
        }
        for output in &session.outputs {
            log::info!(
                "FunASR model output: name={}, type={:?}",
                output.name,
                output.output_type
            );
        }

        let token_path = model_dir.join("tokens.json");
        if !token_path.exists() {
            return Err(FunasrError::InvalidTokenFile(
                token_path.display().to_string(),
            ));
        }
        let token_content = std::fs::read_to_string(&token_path)?;
        let token_list: Vec<String> = serde_json::from_str(&token_content)?;

        log::info!("FunASR token list loaded: {} tokens", token_list.len());

        Ok(Self {
            session: Mutex::new(session),
            token_list,
        })
    }

    /// Run inference on pre-processed features.
    ///
    /// # Arguments
    ///
    /// * `features` — flat float32 features, shape `(1, T, 560)` in row-major order
    /// * `feats_len` — the time-dimension `T` as `int32`
    ///
    /// # Returns
    ///
    /// * `logits` — flat float32 logits from the `"logits"` output
    /// * `token_num` — the scalar `int32` token count from the `"token_num"` output
    pub fn transcribe(
        &self,
        features: &[f32],
        feats_len: i32,
    ) -> Result<(Vec<f32>, i32), FunasrError> {
        let t = feats_len as usize;

        // Input "speech": shape (1, T, 560), float32
        let speech = ndarray::Array3::from_shape_vec((1, t, 560), features.to_vec())
            .map_err(|e| FunasrError::DecodeError(e.to_string()))?;
        let speech_tensor = TensorRef::from_array_view(speech.view())?;

        // Input "speech_lengths": shape (1,), int32
        let lengths = ndarray::Array1::from_shape_vec(1, vec![feats_len])
            .map_err(|e| FunasrError::DecodeError(e.to_string()))?;
        let lengths_tensor = TensorRef::from_array_view(lengths.view())?;

        // Run inference
        let mut session_guard = self.session.lock().unwrap();
        let outputs = session_guard.run(inputs![
            "speech" => speech_tensor,
            "speech_lengths" => lengths_tensor,
        ])?;

        // Extract "logits" output: float32, shape (1, T, vocab_size)
        let logits: Vec<f32> = outputs["logits"]
            .try_extract_array()?
            .iter()
            .copied()
            .collect();

        // Extract "token_num" output: int32, shape (1,)
        let token_num: i32 = *outputs["token_num"]
            .try_extract_array::<i32>()?
            .first()
            .ok_or_else(|| FunasrError::DecodeError("missing token_num output".into()))?;

        Ok((logits, token_num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_not_found_error() {
        let result = FunasrModel::new(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        match result {
            Err(FunasrError::ModelNotFound(_)) => assert!(true),
            _ => panic!("Expected ModelNotFound error"),
        }
    }

    #[test]
    fn test_token_list_size_structure() {
        // Unit test for token list structure without needing a real model.
        let json = r#"["<blank>", "<s>", "</s>", "a", "b", "c"]"#;
        let list: Vec<String> = serde_json::from_str(json).unwrap();
        assert_eq!(list.len(), 6);
        assert_eq!(list[0], "<blank>");
        assert_eq!(list[1], "<s>");
        assert_eq!(list[2], "</s>");
    }

    #[test]
    fn test_invalid_token_file_error() {
        // Create a temp dir with no model.onnx — should fail with ModelNotFound.
        let dir = std::env::temp_dir().join("funasr_test_invalid_tokens");
        std::fs::create_dir_all(&dir).ok();
        let result = FunasrModel::new(&dir);
        assert!(result.is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
