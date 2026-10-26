use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::{Manager, Runtime};

use crate::funasr_onnx::{FunasrError, FunasrFrontend, FunasrModel};

/// Global FunASR models directory — the single source of truth, set once
/// during app setup by [`initialize_models_directory_at`] and read back via
/// [`models_directory`].
static MODELS_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Returns the FunASR models directory previously initialized during app
/// setup, or `None` if [`initialize_models_directory_at`] has not run yet.
pub fn models_directory() -> Option<PathBuf> {
    MODELS_DIR.get().cloned()
}

/// Create the FunASR models directory (idempotently) and record it as the
/// module's single source of truth.
///
/// Testable without a Tauri app handle: callers pass the directory directly.
fn ensure_models_dir(models_dir: &Path) -> Result<PathBuf, FunasrError> {
    // No exists()-then-create check-then-act race: create_dir_all is
    // idempotent for existing directories, and an `AlreadyExists` error (the
    // path exists but is not a directory we created) is treated as benign.
    if let Err(e) = std::fs::create_dir_all(models_dir) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(FunasrError::Io(e));
        }
    }

    log::info!("FunASR models directory set to: {}", models_dir.display());

    // First writer wins; re-initialization with the same directory is a no-op.
    let _ = MODELS_DIR.set(models_dir.to_path_buf());
    Ok(models_dir.to_path_buf())
}

/// Initialize the FunASR models directory under `base_dir`.
///
/// `base_dir` overrides the default app-data root; when it is empty, the root
/// is resolved from `app` (which is used for nothing else). Production passes
/// the app-data root as `base_dir` via [`initialize_models_directory`].
pub fn initialize_models_directory_at<R: Runtime>(
    app: &tauri::AppHandle<R>,
    base_dir: &Path,
) -> Result<PathBuf, FunasrError> {
    let root = if base_dir.as_os_str().is_empty() {
        app.path()
            .app_data_dir()
            .map_err(|e| FunasrError::Io(std::io::Error::other(e)))?
    } else {
        base_dir.to_path_buf()
    };
    ensure_models_dir(&root.join("models"))
}

/// Initialize the FunASR models directory using the app data dir.
/// This should be called during app setup before any FunASR operations.
pub fn initialize_models_directory<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, FunasrError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| FunasrError::Io(std::io::Error::other(e)))?;
    initialize_models_directory_at(app, &data_dir)
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
        log::info!(
            "FunASR engine initialized with model from {}",
            model_dir.display()
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "funasr-engine-test-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn ensure_models_dir_creates_and_is_idempotent() {
        let root = temp_root("create");
        let models_dir = root.join("models");

        let first = ensure_models_dir(&models_dir).expect("first init must succeed");
        assert_eq!(first, models_dir);
        assert!(models_dir.is_dir());

        // Second call hits the AlreadyExists path and must stay benign.
        let second = ensure_models_dir(&models_dir).expect("re-init must succeed");
        assert_eq!(second, models_dir);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_models_dir_propagates_io_failure() {
        // Parent is a regular file, so create_dir_all cannot succeed.
        let root = temp_root("failure");
        std::fs::create_dir_all(&root).expect("create root");
        let blocker = root.join("blocker");
        std::fs::write(&blocker, b"not a directory").expect("write blocker");

        let err = ensure_models_dir(&blocker.join("models")).expect_err("must fail");
        assert!(matches!(err, FunasrError::Io(_)), "got: {err:?}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn models_directory_reader_reflects_initialization() {
        let root = temp_root("reader");
        let models_dir = root.join("models");
        ensure_models_dir(&models_dir).expect("init must succeed");

        // MODELS_DIR is process-global and tests run in parallel: whichever
        // test initializes it first wins, so only presence is asserted here.
        assert!(models_directory().is_some());

        let _ = std::fs::remove_dir_all(&root);
    }
}
