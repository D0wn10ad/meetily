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
            return Err(FunasrError::ModelNotFound(model_path));
        }

        log::info!(
            "Loading FunASR Paraformer model from {}...",
            model_path.display()
        );

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

        log::info!(
            "FunASR model loaded successfully from {}",
            model_path.display()
        );

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
            return Err(FunasrError::InvalidTokenFile {
                path: token_path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "file does not exist"),
            });
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

        // Input "speech": shape (1, T, 560), float32 — viewed in place, zero copy.
        let speech = ndarray::ArrayView3::<f32>::from_shape((1, t, 560), features).map_err(|e| {
            FunasrError::DecodeError {
                message: format!("input feature shape mismatch: {e:?}"),
                source: Some(Box::new(e)),
            }
        })?;
        let speech_tensor = TensorRef::from_array_view(speech)?;

        // Input "speech_lengths": shape (1,), int32
        let lengths = ndarray::Array1::from_shape_vec(1, vec![feats_len]).map_err(|e| {
            FunasrError::DecodeError {
                message: format!("speech_lengths shape mismatch: {e:?}"),
                source: Some(Box::new(e)),
            }
        })?;
        let lengths_tensor = TensorRef::from_array_view(lengths.view())?;

        // Run inference. The guard is poison-tolerant (a panicked writer must not
        // panic every subsequent reader) and is held for the minimum span the ort
        // API allows: `Session::run` takes `&mut self` and the returned
        // `SessionOutputs` borrows the session, so the guard necessarily spans
        // run + extraction (verified against ort 2.0.0-rc.10).
        let mut session_guard = lock_poison_tolerant(&self.session);
        let outputs = session_guard.run(inputs![
            "speech" => speech_tensor,
            "speech_lengths" => lengths_tensor,
        ])?;

        // Extract "logits" output: float32, shape (1, T, vocab_size)
        let logits: Vec<f32> = require_output(|name| outputs.get(name), "logits")?
            .try_extract_array()?
            .iter()
            .copied()
            .collect();

        // Extract "token_num" output: int32, shape (1,)
        let token_num: i32 = *require_output(|name| outputs.get(name), "token_num")?
            .try_extract_array::<i32>()?
            .first()
            .ok_or_else(|| FunasrError::DecodeError {
                message: "token_num output is empty".into(),
                source: None,
            })?;

        Ok((logits, token_num))
    }
}

/// Locks `mutex`, recovering the inner value if a previous writer panicked and
/// poisoned it. Inference state is self-contained per call, so recovery is safe
/// and strictly better than panicking every subsequent reader.
fn lock_poison_tolerant<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Looks up a required model output by name, returning a typed error that names
/// the missing key instead of panicking (string-indexing the outputs map panics
/// on a missing key).
fn require_output<'v, F>(get: F, name: &str) -> Result<&'v ort::value::DynValue, FunasrError>
where
    F: FnOnce(&str) -> Option<&'v ort::value::DynValue>,
{
    get(name).ok_or_else(|| FunasrError::DecodeError {
        message: format!("missing required model output '{name}'"),
        source: None,
    })
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

    #[test]
    fn hardware_profile_detect_returns_single_cached_instance() {
        // T11 item 1 (verify-only): the GPU probe is already cached process-wide by the
        // `OnceLock` in `audio/hardware_detector.rs:49` (`get_or_init` at :53), and
        // `model.rs` calls `detect()` in exactly one place. Two calls must yield the
        // same `&'static` instance — proving the probe runs at most once and that no
        // second cache was introduced here.
        let a = crate::audio::hardware_detector::HardwareProfile::detect();
        let b = crate::audio::hardware_detector::HardwareProfile::detect();
        assert!(
            std::ptr::eq(a, b),
            "detect() must return the same cached &'static HardwareProfile"
        );
    }

    #[test]
    fn missing_output_key_returns_err_not_panic() {
        // T11 item 5: a missing output key must produce a typed Err (naming the key),
        // never a panic. `catch_unwind` proves the lookup path is panic-free.
        let getter = |_name: &str| -> Option<&'static ort::value::DynValue> { None };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            require_output(getter, "token_num")
        }));
        match result {
            Ok(Err(FunasrError::DecodeError { message, .. })) => {
                assert!(
                    message.contains("token_num"),
                    "error must name the missing key, got: {message}"
                );
            }
            Ok(Ok(_)) => panic!("expected Err for a missing output key"),
            Ok(Err(other)) => panic!("expected DecodeError, got: {other:?}"),
            Err(_) => panic!("require_output must not panic on a missing output key"),
        }
    }

    #[test]
    fn lock_poison_tolerant_recovers_after_poison() {
        // T11 item 6: a mutex poisoned by a panicked writer must be recovered via
        // `into_inner()` instead of panicking the reader thread.
        let mutex = std::sync::Arc::new(Mutex::new(41i32));
        let poisoned = std::sync::Arc::clone(&mutex);
        let handle = std::thread::spawn(move || {
            let _guard = poisoned.lock().unwrap();
            panic!("deliberate poison");
        });
        assert!(handle.join().is_err(), "writer thread panicked, poisoning the mutex");
        let guard = lock_poison_tolerant(&mutex);
        assert_eq!(*guard, 41, "inner value must be recovered from the poisoned mutex");
    }

    #[test]
    fn test_session_creation_with_dynamic_providers() {
        for gpu in [
            crate::audio::hardware_detector::GpuType::None,
            crate::audio::hardware_detector::GpuType::Metal,
            crate::audio::hardware_detector::GpuType::Cuda,
            crate::audio::hardware_detector::GpuType::Vulkan,
            crate::audio::hardware_detector::GpuType::OpenCL,
        ] {
            let providers = crate::audio::onnx_provider::get_onnx_providers(gpu);
            assert!(
                !providers.is_empty(),
                "Must always have at least CPU fallback for {:?}",
                gpu
            );
        }
    }
}
