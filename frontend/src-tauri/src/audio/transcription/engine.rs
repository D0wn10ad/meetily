// audio/transcription/engine.rs
//
// TranscriptionEngine enum and model initialization/validation logic.

use super::provider::TranscriptionProvider;
use log::{info, warn};
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{AppHandle, Manager, Runtime};

// ============================================================================
// LOCK HELPERS
// ============================================================================

/// Lock a mutex, recovering from poisoning instead of panicking.
///
/// A poisoned mutex still holds a valid inner value — only the poison flag
/// is set — so the guard is recovered via `into_inner()` and the recovery
/// is logged once.
fn lock_unpoison<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    match m.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Recovered from a poisoned lock in the transcription engine");
            poisoned.into_inner()
        }
    }
}

// ============================================================================
// CONFIG RESOLUTION (single fetch + single fallback)
// ============================================================================

/// Apply the transcript-config fallback to a fetched config result.
///
/// Pure seam for tests: takes the fetch result as an argument so no
/// `AppHandle` (or database) is needed. This is the ONLY place in this
/// module that contains the "default to parakeet" fallback:
/// - `Ok(Some(config))` → use the saved config as-is;
/// - `Ok(None)` → default to the parakeet provider + default model;
/// - `Err(e)` → warn once and use the same default.
fn resolve_config_from_fetch(
    fetched: Result<Option<crate::api::api::TranscriptConfig>, String>,
) -> crate::api::api::TranscriptConfig {
    let config = match fetched {
        Ok(Some(config)) => {
            info!(
                "📝 Found transcript config - provider: {}, model: {}",
                config.provider, config.model
            );
            return config;
        }
        Ok(None) => {
            info!("📝 No transcript config found, defaulting to parakeet");
        }
        Err(e) => {
            warn!(
                "⚠️ Failed to get transcript config: {}, defaulting to parakeet",
                e
            );
        }
    };
    crate::api::api::TranscriptConfig {
        provider: "parakeet".to_string(),
        model: crate::config::DEFAULT_PARAKEET_MODEL.to_string(),
        api_key: None,
    }
}

/// Fetch the transcript configuration and apply the fallback exactly once.
async fn resolve_transcription_config<R: Runtime>(
    app: &AppHandle<R>,
) -> crate::api::api::TranscriptConfig {
    let fetched =
        crate::api::api::api_get_transcript_config(app.clone(), app.clone().state(), None).await;
    resolve_config_from_fetch(fetched)
}

// ============================================================================
// TRANSCRIPTION ENGINE ENUM
// ============================================================================

// Transcription engine abstraction to support multiple providers
pub enum TranscriptionEngine {
    Whisper(Arc<crate::whisper_engine::WhisperEngine>), // Direct access (backward compat)
    Parakeet(Arc<crate::parakeet_engine::ParakeetEngine>), // Direct access (backward compat)
    Provider(Arc<dyn TranscriptionProvider>),           // Trait-based (preferred for new code)
}

impl TranscriptionEngine {
    /// Check if the engine has a model loaded
    pub async fn is_model_loaded(&self) -> bool {
        match self {
            Self::Whisper(engine) => engine.is_model_loaded().await,
            Self::Parakeet(engine) => engine.is_model_loaded().await,
            Self::Provider(provider) => provider.is_model_loaded().await,
        }
    }

    /// Get the current model name
    pub async fn get_current_model(&self) -> Option<String> {
        match self {
            Self::Whisper(engine) => engine.get_current_model().await,
            Self::Parakeet(engine) => engine.get_current_model().await,
            Self::Provider(provider) => provider.get_current_model().await,
        }
    }

    /// Get the provider name for logging
    pub fn provider_name(&self) -> &str {
        match self {
            Self::Whisper(_) => "Whisper (direct)",
            Self::Parakeet(_) => "Parakeet (direct)",
            Self::Provider(provider) => provider.provider_name(),
        }
    }
}

// ============================================================================
// MODEL VALIDATION AND INITIALIZATION
// ============================================================================

/// Validate that transcription models (Whisper or Parakeet) are ready before starting recording
pub async fn validate_transcription_model_ready<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    // Check transcript configuration to determine which engine to validate
    let config = resolve_transcription_config(app).await;

    // Validate based on provider
    match config.provider.as_str() {
        "localWhisper" => {
            info!("🔍 Validating Whisper model...");
            // Ensure whisper engine is initialized first
            if let Err(init_error) = crate::whisper_engine::commands::whisper_init().await {
                warn!("❌ Failed to initialize Whisper engine: {}", init_error);
                return Err(format!(
                    "Failed to initialize speech recognition: {}",
                    init_error
                ));
            }

            // Call the whisper validation command with config support
            match crate::whisper_engine::commands::whisper_validate_model_ready_with_config(app)
                .await
            {
                Ok(model_name) => {
                    info!(
                        "✅ Whisper model validation successful: {} is ready",
                        model_name
                    );
                    Ok(())
                }
                Err(e) => {
                    warn!("❌ Whisper model validation failed: {}", e);
                    Err(e)
                }
            }
        }
        "parakeet" => {
            info!("🔍 Validating Parakeet model...");
            // Ensure parakeet engine is initialized first
            if let Err(init_error) = crate::parakeet_engine::commands::parakeet_init().await {
                warn!("❌ Failed to initialize Parakeet engine: {}", init_error);
                return Err(format!(
                    "Failed to initialize Parakeet speech recognition: {}",
                    init_error
                ));
            }

            // Use the validation command that includes auto-discovery and loading
            // This matches the Whisper behavior for consistency
            match crate::parakeet_engine::commands::parakeet_validate_model_ready_with_config(app)
                .await
            {
                Ok(model_name) => {
                    info!(
                        "✅ Parakeet model validation successful: {} is ready",
                        model_name
                    );
                    Ok(())
                }
                Err(e) => {
                    warn!("❌ Parakeet model validation failed: {}", e);
                    Err(e)
                }
            }
        }
        "funasr" => {
            info!("🔍 Validating FunASR model...");
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            let model_name = if config.model.is_empty() { "paraformer-large" } else { &config.model };
            let model_dir = app_data_dir
                .join("models")
                .join("funasr")
                .join(model_name);
            if !model_dir.exists() {
                return Err(format!(
                    "FunASR model not found at {}. Please place model.onnx, tokens.json, and am.mvn in this directory.",
                    model_dir.display()
                ));
            }
            // Load the model for real: an existence-only check passes corrupt
            // or truncated models, so construct the provider (which builds the
            // ONNX session) on the blocking thread pool.
            let dir = model_dir.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                crate::audio::transcription::FunasrProvider::new(&dir)
            })
            .await
            .map_err(|e| format!("FunASR model validation task failed: {}", e))?;
            loaded
                .map(|_| ())
                .map_err(|e| format!("FunASR model validation failed: {}", e))?;
            info!("✅ FunASR model validated at {}", model_dir.display());
            Ok(())
        }
        "sherpa-onnx" => {
            info!("🔍 Validating Sherpa-ONNX model...");
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            let model_name = if config.model.is_empty() {
                "sensevoice"
            } else {
                &config.model
            };
            let model_dir = app_data_dir
                .join("models")
                .join("sherpa-onnx")
                .join(model_name);
            if !model_dir.exists() {
                return Err(format!(
                    "Sherpa-ONNX model not found at {}. Please place model.onnx and tokens.txt in this directory.",
                    model_dir.display()
                ));
            }
            // Load the model for real: structure-only validation passes
            // corrupt models, so construct the provider (which builds the
            // recognizer) on the blocking thread pool.
            let dir = model_dir.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                crate::audio::transcription::SherpaOnnxProvider::new(&dir)
            })
            .await
            .map_err(|e| format!("Sherpa-ONNX model validation task failed: {}", e))?;
            loaded
                .map(|_| ())
                .map_err(|e| format!("Sherpa-ONNX model validation failed: {}", e))?;
            info!(
                "✅ Sherpa-ONNX model validated at {}",
                model_dir.display()
            );
            Ok(())
        }
        other => {
            warn!(
                "❌ Unsupported transcription provider for local recording: {}",
                other
            );
            Err(format!(
                "Provider '{}' is not supported for local transcription. Please select 'localWhisper', 'parakeet', 'funasr', or 'sherpa-onnx'.",
                other
            ))
        }
    }
}

/// Get or initialize the appropriate transcription engine based on provider configuration
pub async fn get_or_init_transcription_engine<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<TranscriptionEngine, String> {
    // Get provider configuration from API (single fetch + single fallback)
    let config = resolve_transcription_config(app).await;

    // Initialize the appropriate engine based on provider
    match config.provider.as_str() {
        "parakeet" => {
            info!("🦜 Initializing Parakeet transcription engine");

            // Get Parakeet engine
            let engine = {
                let guard =
                    lock_unpoison(&crate::parakeet_engine::commands::PARAKEET_ENGINE);
                guard.as_ref().cloned()
            };

            match engine {
                Some(engine) => {
                    // Check if model is loaded
                    if engine.is_model_loaded().await {
                        let model_name = engine
                            .get_current_model()
                            .await
                            .unwrap_or_else(|| "unknown".to_string());
                        info!("✅ Parakeet model '{}' already loaded", model_name);
                        Ok(TranscriptionEngine::Parakeet(engine))
                    } else {
                        Err("Parakeet engine initialized but no model loaded. This should not happen after validation.".to_string())
                    }
                }
                None => Err(
                    "Parakeet engine not initialized. This should not happen after validation."
                        .to_string(),
                ),
            }
        }
        "funasr" => {
            info!("🎯 Initializing FunASR transcription engine");
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            let model_name = if config.model.is_empty() { "paraformer-large" } else { &config.model };
            let model_dir = app_data_dir
                .join("models")
                .join("funasr")
                .join(model_name);
            let provider = crate::audio::transcription::FunasrProvider::new(&model_dir)
                .map_err(|e| format!("Failed to initialize FunASR: {}", e))?;
            Ok(TranscriptionEngine::Provider(Arc::new(provider)))
        }
        "sherpa-onnx" => {
            info!("🎯 Initializing Sherpa-ONNX transcription engine");
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            let model_name = if config.model.is_empty() {
                "sensevoice"
            } else {
                &config.model
            };
            let model_dir = app_data_dir
                .join("models")
                .join("sherpa-onnx")
                .join(model_name);
            let provider = crate::audio::transcription::SherpaOnnxProvider::new(&model_dir)
                .map_err(|e| format!("Failed to initialize Sherpa-ONNX: {}", e))?;
            Ok(TranscriptionEngine::Provider(Arc::new(provider)))
        }
        "localWhisper" | _ => {
            info!("🎤 Initializing Whisper transcription engine");
            let whisper_engine = get_or_init_whisper(app).await?;
            Ok(TranscriptionEngine::Whisper(whisper_engine))
        }
    }
}

/// Get or initialize transcription engine using API configuration
/// Returns Whisper engine if provider is localWhisper, otherwise returns error for non-Whisper providers
pub async fn get_or_init_whisper<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Arc<crate::whisper_engine::WhisperEngine>, String> {
    // Check if engine already exists and has a model loaded
    let existing_engine = {
        let engine_guard =
            lock_unpoison(&crate::whisper_engine::commands::WHISPER_ENGINE);
        engine_guard.as_ref().cloned()
    };

    if let Some(engine) = existing_engine {
        // Check if a model is already loaded
        if engine.is_model_loaded().await {
            let current_model = engine
                .get_current_model()
                .await
                .unwrap_or_else(|| "unknown".to_string());

            // Check if loaded model matches saved config (single fetch + fallback)
            let config = resolve_transcription_config(app).await;
            let configured_model = if config.provider == "localWhisper" && !config.model.is_empty()
            {
                Some(config.model)
            } else {
                None
            };

            // If loaded model matches config, reuse it
            if let Some(ref expected_model) = configured_model {
                if current_model == *expected_model {
                    info!(
                        "✅ Loaded model '{}' matches saved config, reusing",
                        current_model
                    );
                    return Ok(engine);
                } else {
                    info!(
                        "🔄 Loaded model '{}' doesn't match saved config '{}', reloading correct model...",
                        current_model, expected_model
                    );
                    // Unload the incorrect model
                    engine.unload_model().await;
                    info!("📉 Unloaded incorrect model '{}'", current_model);
                    // Continue to model loading logic below
                }
            } else {
                // No specific config saved, accept currently loaded model
                info!(
                    "✅ No specific model configured, using currently loaded model: '{}'",
                    current_model
                );
                return Ok(engine);
            }
        } else {
            info!("🔄 Whisper engine exists but no model loaded, will load model from config");
        }
    }

    // Initialize new engine if needed
    info!("Initializing Whisper engine");

    // First ensure the engine is initialized
    if let Err(e) = crate::whisper_engine::commands::whisper_init().await {
        return Err(format!("Failed to initialize Whisper engine: {}", e));
    }

    // Get the engine reference
    let engine = {
        let engine_guard =
            lock_unpoison(&crate::whisper_engine::commands::WHISPER_ENGINE);
        engine_guard
            .as_ref()
            .cloned()
            .ok_or("Failed to get initialized engine")?
    };

    // Get model configuration from API (single fetch + fallback)
    let config = resolve_transcription_config(app).await;
    if config.provider != "localWhisper" {
        // Non-Whisper provider (e.g., parakeet) - this function shouldn't be called
        return Err(format!(
            "Cannot initialize Whisper engine: Config uses '{}' provider. This is a bug in the transcription task initialization.",
            config.provider
        ));
    }
    let model_to_load = config.model;

    info!("Selected model to load: {}", model_to_load);

    // Discover available models to check if the desired model is downloaded
    let models = engine
        .discover_models()
        .await
        .map_err(|e| format!("Failed to discover models: {}", e))?;

    info!("Discovered {} models", models.len());
    for model in &models {
        info!(
            "Model: {} - Status: {:?} - Path: {}",
            model.name,
            model.status,
            model.path.display()
        );
    }

    // Check if the desired model is available
    let model_info = models.iter().find(|model| model.name == model_to_load);

    if model_info.is_none() {
        info!(
            "Model '{}' not found in discovered models. Available models: {:?}",
            model_to_load,
            models.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
    }

    match model_info {
        Some(model) => {
            match model.status {
                crate::whisper_engine::ModelStatus::Available => {
                    info!("Loading model: {}", model_to_load);
                    engine
                        .load_model(&model_to_load)
                        .await
                        .map_err(|e| format!("Failed to load model '{}': {}", model_to_load, e))?;
                    info!("✅ Model '{}' loaded successfully", model_to_load);
                }
                crate::whisper_engine::ModelStatus::Missing => {
                    return Err(format!(
                        "Model '{}' is not downloaded. Please download it first from the settings.",
                        model_to_load
                    ));
                }
                crate::whisper_engine::ModelStatus::Downloading { progress } => {
                    return Err(format!("Model '{}' is currently downloading ({}%). Please wait for it to complete.", model_to_load, progress));
                }
                crate::whisper_engine::ModelStatus::Error(ref err) => {
                    return Err(format!("Model '{}' has an error: {}. Please check the model or try downloading it again.", model_to_load, err));
                }
                crate::whisper_engine::ModelStatus::Corrupted { .. } => {
                    return Err(format!("Model '{}' is corrupted. Please delete it and download again from the settings.", model_to_load));
                }
            }
        }
        None => {
            // Check if we have any available models and try to load the first one
            let available_models: Vec<_> = models
                .iter()
                .filter(|m| matches!(m.status, crate::whisper_engine::ModelStatus::Available))
                .collect();

            if let Some(fallback_model) = available_models.first() {
                warn!(
                    "Model '{}' not found, falling back to available model: '{}'",
                    model_to_load, fallback_model.name
                );
                engine.load_model(&fallback_model.name).await.map_err(|e| {
                    format!(
                        "Failed to load fallback model '{}': {}",
                        fallback_model.name, e
                    )
                })?;
                info!(
                    "✅ Fallback model '{}' loaded successfully",
                    fallback_model.name
                );
            } else {
                return Err(format!("Model '{}' is not supported and no other models are available. Please download a model from the settings.", model_to_load));
            }
        }
    }

    Ok(engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_config_defaults_on_none() {
        let cfg = resolve_config_from_fetch(Ok(None));
        assert_eq!(cfg.provider, "parakeet");
        assert_eq!(cfg.model, crate::config::DEFAULT_PARAKEET_MODEL);
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn test_resolve_config_defaults_on_err() {
        let cfg = resolve_config_from_fetch(Err("database unavailable".to_string()));
        assert_eq!(cfg.provider, "parakeet");
        assert_eq!(cfg.model, crate::config::DEFAULT_PARAKEET_MODEL);
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn test_resolve_config_uses_fetched_config() {
        let cfg = resolve_config_from_fetch(Ok(Some(crate::api::api::TranscriptConfig {
            provider: "funasr".to_string(),
            model: "paraformer-large".to_string(),
            api_key: None,
        })));
        assert_eq!(cfg.provider, "funasr");
        assert_eq!(cfg.model, "paraformer-large");
    }

    #[test]
    fn test_lock_unpoison_recovers_from_poisoned_mutex() {
        let mutex = Mutex::new(41usize);
        *mutex.lock().expect("setup lock") += 1; // value = 42

        // Poison the mutex by panicking while holding the lock.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().expect("poisoning lock");
            panic!("intentional poison");
        }));
        assert!(result.is_err(), "the inner panic must poison the mutex");

        // lock_unpoison must recover the value instead of panicking.
        let value = *lock_unpoison(&mutex);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_lock_unpoison_returns_guard_on_unpoisoned_mutex() {
        let mutex = Mutex::new("intact".to_string());
        let guard = lock_unpoison(&mutex);
        assert_eq!(*guard, "intact");
    }
}
