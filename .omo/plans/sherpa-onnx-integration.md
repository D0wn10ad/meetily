# sherpa-onnx-integration — Work Plan

**Branch**: `feat/sherpa-onnx`

## TL;DR (For humans)

**What you'll get:** A new ASR option "Sherpa-ONNX (SenseVoice)" alongside "Whisper", "Parakeet", and "FunASR" in the app settings. When selected, Meetily will transcribe meetings using sherpa-onnx's OfflineRecognizer with SenseVoice models — bringing built-in ITN (punctuation and English spacing), emotion markers (`[laughter]`, `[applause]`), and multi-language support (CN/EN/JP/KR/YUE) — all locally, all in-process, no sidecars.

**Why this approach:** sherpa-onnx is a mature Rust crate that bundles SenseVoice, Paraformer, and many other ASR models. It handles GPU auto-detection (CUDA/Vulkan/Metal/CoreML) at runtime, has built-in ITN, and follows the exact same `Provider(Arc<dyn TranscriptionProvider>)` pattern as FunASR. No new FFT, no new frontend DSP, no custom decoding — sherpa-onnx gives us all of this out of the box.

**What it will NOT do:** No auto-download (Phase 1 — manual model placement). No speaker diarization (deferred). No Paraformer via sherpa-onnx (Phase 2). No changes to Whisper, Parakeet, or FunASR providers. No new `TranscriptionEngine` variant — uses existing `Provider(Arc<dyn TranscriptionProvider>)`.

**Effort:** Medium
**Risk:** Low-Medium — follows the `Provider(Arc<dyn TranscriptionProvider>)` pattern established by FunASR, but introduces a new external crate dependency (sherpa-onnx)
**Decisions to sanity-check:** The `sense-voice` feature on sherpa-onnx must be selected; GPU is auto-detected at runtime upstream so no special handling needed; model directory path is `$APPDATA/Meetily/models/sherpa-onnx/sensevoice/`

Your next move: Approve the plan so I write the code, or request edits.

---

> **TL;DR (machine)**: Medium effort, low-medium risk. Add sherpa-onnx crate with SenseVoice via `Provider(Arc<...>)` pattern. 11 implementation todos + 1 final verification wave.

## Scope

### Must have
1. `Cargo.toml`: add `sherpa-onnx = { version = "1.13", features = ["sense-voice"] }` dependency
2. Create `sherpa_onnx/mod.rs` — error enum (`SherpaOnnxError`), re-exports
3. Create `sherpa_onnx/model.rs` — `ModelInfo`, `discover_models()`, validation (manual placement check)
4. Create `sherpa_onnx/engine.rs` — `SherpaOnnxEngine` wrapping sherpa-onnx `OfflineRecognizer`
5. Create `sherpa_onnx/commands.rs` — Tauri command `api_scan_sherpa_onnx_models`
6. Create `audio/transcription/sherpa_onnx_provider.rs` — impl `TranscriptionProvider`
7. Wire into `audio/transcription/engine.rs` — add `"sherpa-onnx"` match arms for validate + init
8. Register module in `lib.rs` — `mod sherpa_onnx`, register commands, log model dir
9. Create frontend `components/SherpaOnnxModelManager.tsx` — scan models, show status, select
10. Add `"sherpa-onnx"` to `TranscriptSettings.tsx` provider type union + `<SelectItem>`
11. Add `"sherpa-onnx"` to frontend hook for model config (`useModelConfiguration.ts`)

### Must NOT have (guardrails, anti-slop, scope boundaries)
- NO diarization (Phase 2 or later)
- NO auto-download of models (Phase 2)
- NO changes to existing providers (Whisper, Parakeet, FunASR)
- NO new `TranscriptionEngine` variant — use existing `Provider(Arc<dyn TranscriptionProvider>)`
- NO changes to `TranscriptionProvider` trait or `TranscriptResult` struct
- NO `ort`/`realfft` dependency changes (sherpa-onnx bundles its own ONNX runtime)
- NO new Tauri events (only the existing `transcript-update` path via `get_or_init_transcription_engine()`)
- NO changes to `tauri.conf.json` or CSP

## Key Details

| Parameter | Value |
|-----------|-------|
| Provider string | `"sherpa-onnx"` |
| Rust module name | `sherpa_onnx` (underscores for Rust, hyphens for DB/config) |
| Model directory | `$APPDATA/Meetily/models/sherpa-onnx/sensevoice/` |
| Model files | `sensevoice-encoder-2.5-192-768.onnx`, `sensevoice-decoder-2.5-192-768.onnx`, `tokens.txt` |
| GPU | Auto-detected by sherpa-onnx at runtime — no special handling needed |
| ITN | Always-on via `use_itn = true` config on `OfflineRecognizer` |
| Emotion | Inline markers `[laughter]`, `[applause]` — no `TranscriptResult` changes |
| Engine init | On-demand via `Provider(Arc<...>)` — NOT a global singleton |
| Model download | None in Phase 1 — just model scanning via `discover_models()` |

## Data Flow

```
VAD speech segment
    → TranscriptionEngine::Provider(SherpaOnnxProvider)
    → SherpaOnnxEngine.transcribe_audio(audio)
    → sherpa-onnx OfflineRecognizer.run(audio)
    → text with ITN applied + emotion events inline
    → TranscriptResult { text, confidence: None, is_partial: false }
    → transcript-update event (existing path)
```

## Module Structure

```
frontend/src-tauri/src/
├── sherpa_onnx/
│   ├── mod.rs            — SherpaOnnxError enum, re-exports
│   ├── engine.rs         — SherpaOnnxEngine: wraps sherpa-onnx OfflineRecognizer
│   ├── model.rs          — ModelInfo, discover_models(), validation
│   └── commands.rs       — Tauri api_scan_sherpa_onnx_models command
├── audio/transcription/
│   └── sherpa_onnx_provider.rs  — impl TranscriptionProvider for SherpaOnnxEngine
```

```
frontend/src/
├── components/
│   └── SherpaOnnxModelManager.tsx  — model selection & status UI
```

## Configuration & Routing

### In `engine.rs`

The `get_or_init_transcription_engine()` and `validate_transcription_model_ready()` functions grow a new arm:

```rust
"sherpa-onnx" => {
    // 1. Validate model directory exists
    // 2. Create SherpaOnnxProvider
    // 3. Return TranscriptionEngine::Provider(Arc::new(provider))
}
```

### DB config (no schema changes)

```json
{
  "provider": "sherpa-onnx",
  "model": "sensevoice"
}
```

The existing transcript config table already stores `provider` and `model`.

## Execution strategy

### Execution order
Todos 1-5 are independent (Wave 1). Todo 6 depends on 4. Todos 7-11 depend on 6.

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1. Cargo.toml | — | 2-11 | — (blocks all compilation) |
| 2. `sherpa_onnx/mod.rs` | — | 3, 4, 5 | — |
| 3. `sherpa_onnx/model.rs` | 2 | 4 | 5 |
| 4. `sherpa_onnx/engine.rs` | 2, 3 | 6 | — |
| 5. `sherpa_onnx/commands.rs` | 2 | 8 | 3 |
| 6. `sherpa_onnx_provider.rs` | 4 | 7, 8 | — |
| 7. `engine.rs` wiring | 6 | 8 | — |
| 8. `lib.rs` registration | 5, 6, 7 | 9-11 | — |
| 9. `SherpaOnnxModelManager.tsx` | 8 | 10 | — |
| 10. `TranscriptSettings.tsx` | 8 | — | 11 |
| 11. `useModelConfiguration.ts` | 8 | — | 10 |

### Parallel waves
- **Wave 1**: Todos 3, 5 (parallel)
- **Wave 2**: Todos 4, 6 (sequential — engine needs model, provider needs engine)
- **Wave 3**: Todos 7, 8 (wiring)
- **Wave 4**: Todos 9, 10, 11 (frontend, parallel)
- **Final**: F1-F4

## Todos

- [ ] 1. Add sherpa-onnx dependency to `Cargo.toml`

  **What to do**: Add the sherpa-onnx crate with the `sense-voice` feature flag.

  **File to modify**:
  - `frontend/src-tauri/Cargo.toml`

  **Addition** (in the `[dependencies]` section, alphabetically):
  ```toml
  sherpa-onnx = { version = "1.13", features = ["sense-voice"] }
  ```

  **Must NOT do**:
  - Do NOT add any other features (no `paraformer`, `offline-stream`, etc. — Phase 2)
  - Do NOT remove or modify any existing dependencies
  - Do NOT add `ort` or alter any `ort` feature flags — sherpa-onnx bundles its own ONNX runtime

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes after adding the dependency
  2. The `sherpa-onnx` crate is resolvable and the `sense-voice` feature compiles

  **Parallelization**: Wave 1 | Blocked by: none | Blocks: all subsequent todos

- [ ] 2. Create `sherpa_onnx/mod.rs` — error enum and re-exports

  **What to do**: Create the module entrypoint with the `SherpaOnnxError` enum and re-exports for all submodules.

  **File to create**:
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs`

  **Error enum**:
  ```rust
  pub enum SherpaOnnxError {
      #[error("sherpa-onnx error: {0}")]
      SherpaOnnx(String),
      #[error("Model not found at {0}")]
      ModelNotFound(String),
      #[error("I/O error: {0}")]
      Io(#[from] std::io::Error),
      #[error("Invalid model directory: {0}")]
      InvalidModelDir(String),
      #[error("Empty audio input")]
      EmptyAudio,
  }
  ```
  (Derive `thiserror::Error` — already in the crate's dependency tree.)

  **Re-exports** (added after modules are created):
  ```rust
  pub mod model;
  pub mod engine;
  pub mod commands;
  pub use engine::SherpaOnnxEngine;
  pub use model::{ModelInfo, discover_models};
  ```

  **Must NOT do**:
  - Do NOT add any FunASR/Paraformer-related types
  - Do NOT expose internals of `sherpa-onnx` crate publicly

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. `SherpaOnnxError` enum compiles with all variants

  **Parallelization**: Wave 1 | Blocked by: 1 | Blocks: 3, 4, 5

- [ ] 3. Create `sherpa_onnx/model.rs` — model discovery and validation

  **What to do**: Implement `ModelInfo` struct and `discover_models()` function that scans the model directory for valid SenseVoice model files.

  **File to create**:
  - `frontend/src-tauri/src/sherpa_onnx/model.rs`

  **Struct definition**:
  ```rust
  pub struct ModelInfo {
      pub name: String,        // Display name, e.g. "SenseVoice"
      pub model_type: String,  // "sensevoice"
      pub path: PathBuf,       // Full path to model directory
      pub status: ModelStatus, // Ready or Missing with details
  }

  pub enum ModelStatus {
      Ready,
      Missing { details: String },
  }
  ```

  **Functions**:
  ```rust
  /// Scan models/sherpa-onnx/ directory for valid model directories.
  /// Returns a list of ModelInfo (one per subdirectory with required files).
  pub fn discover_models(models_dir: &Path) -> Vec<ModelInfo>;

  /// Validate that a specific model directory has all required files.
  pub fn validate_model_dir(model_dir: &Path) -> Result<(), SherpaOnnxError>;
  ```

  **Required files for SenseVoice**:
  ```
  <model_dir>/
    sensevoice-encoder-2.5-192-768.onnx
    sensevoice-decoder-2.5-192-768.onnx
    tokens.txt
  ```

  **Must NOT do**:
  - Do NOT implement model download (Phase 2)
  - Do NOT implement model deletion
  - Do NOT call Tauri APIs from this module (pure data/logic)

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. Unit test: `discover_models()` with empty dir → empty Vec
  3. Unit test: `discover_models()` with valid model dir → Vec with one Ready entry
  4. Unit test: `validate_model_dir()` with missing encoder → `Err(SherpaOnnxError::ModelNotFound)`

  **Parallelization**: Wave 1 | Blocked by: 2 | Blocks: 4

- [ ] 4. Create `sherpa_onnx/engine.rs` — SherpaOnnxEngine wrapping OfflineRecognizer

  **What to do**: Implement `SherpaOnnxEngine` that wraps sherpa-onnx's `OfflineRecognizer` and exposes a `transcribe_audio()` method.

  **File to create**:
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs`

  **Struct definition**:
  ```rust
  use sherpa_onnx::OfflineRecognizer;

  pub struct SherpaOnnxEngine {
      recognizer: OfflineRecognizer,
  }

  impl SherpaOnnxEngine {
      pub fn new(model_dir: &Path) -> Result<Self, SherpaOnnxError>;
      /// Input: 16kHz mono f32 PCM audio samples.
      /// Returns transcribed text with ITN applied and emotion markers inline.
      pub fn transcribe_audio(&self, audio: &[f32]) -> Result<String, SherpaOnnxError>;
  }
  ```

  **`new()` implementation**:
  1. Validate model directory via `crate::sherpa_onnx::model::validate_model_dir()`
  2. Build `OfflineRecognizerConfig` with `sense-voice` model config:
     - `feat_config.sample_rate = 16000`
     - `feat_config.feature_dim = 80`
     - `model_config.sense_voice.model = encoder_path.to_string_lossy()`
     - `model_config.sense_voice.tokens = tokens_path.to_string_lossy()`
     - `model_config.sense_voice.use_itn = true`
     - `model_config.sense_voice.num_threads = 2`
  3. Create `OfflineRecognizer::new(config)?`

  **`transcribe_audio()` implementation**:
  1. Create `OfflineStream` from the recognizer
  2. Accept the f32 PCM audio (16kHz mono)
  3. Feed audio to the stream
  4. Call `recognizer.decode(stream)` to get the result
  5. Extract text from the result
  6. Return the text string

  **Must NOT do**:
  - Do NOT create a global singleton — engine is created on-demand
  - Do NOT handle GPU config manually — sherpa-onnx auto-detects
  - Do NOT add streaming/partial results support
  - Do NOT modify the audio f32 slice (no resampling — pipeline already outputs 16kHz)

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. Unit test: `SherpaOnnxEngine::new()` with valid model dir → returns `Ok`
  3. Unit test: `SherpaOnnxEngine::new()` with missing model → returns `Err(SherpaOnnxError::ModelNotFound)`
  4. Integration test (manual): With real model on disk → `transcribe_audio()` returns non-empty text

  **Parallelization**: Wave 2 | Blocked by: 2, 3 | Blocks: 6

- [ ] 5. Create `sherpa_onnx/commands.rs` — Tauri command for frontend model scanning

  **What to do**: Expose a Tauri command `api_scan_sherpa_onnx_models` that the frontend calls to discover available Sherpa-ONNX models.

  **File to create**:
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs`

  **Tauri command**:
  ```rust
  #[tauri::command]
  pub async fn api_scan_sherpa_onnx_models(
      app: tauri::AppHandle,
  ) -> Result<Vec<ModelInfoResponse>, String> {
      // Resolve models/sherpa-onnx/ from app data dir
      // Call discover_models()
      // Map to serializable response
  }
  ```

  **Response type** (serializable):
  ```rust
  #[derive(Serialize)]
  pub struct ModelInfoResponse {
      pub name: String,
      pub model_type: String,
      pub path: String,
      pub ready: bool,
      pub details: Option<String>,
  }
  ```

  **Must NOT do**:
  - Do NOT add download commands (Phase 2)
  - Do NOT add delete/modify commands
  - Do NOT add any command that modifies disk state

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. The command compiles and registers in Tauri's command inventory
  3. Returns empty list when no models directory exists

  **Parallelization**: Wave 1 | Blocked by: 2 | Blocks: 8

- [ ] 6. Create `audio/transcription/sherpa_onnx_provider.rs` — TranscriptionProvider impl

  **What to do**: Implement the `TranscriptionProvider` trait for `SherpaOnnxEngine`, following the same pattern as `funasr_provider.rs`.

  **File to create**:
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs`

  **Implementation**:
  ```rust
  use super::provider::{TranscriptionProvider, TranscriptionError, TranscriptResult};
  use crate::sherpa_onnx::SherpaOnnxEngine;

  pub struct SherpaOnnxProvider {
      engine: SherpaOnnxEngine,
  }

  impl SherpaOnnxProvider {
      pub fn new(model_dir: &Path) -> Result<Self, SherpaOnnxError>;
  }

  #[async_trait]
  impl TranscriptionProvider for SherpaOnnxProvider {
      fn provider_name(&self) -> &str { "Sherpa-ONNX (SenseVoice)" }

      async fn is_model_loaded(&self) -> bool { true }

      async fn get_current_model(&self) -> Option<String> { Some("sensevoice".to_string()) }

      async fn transcribe(
          &self,
          audio: Vec<f32>,
          _language: Option<String>,
      ) -> Result<TranscriptResult, TranscriptionError> {
          let text = self.engine.transcribe_audio(&audio)
              .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;
          Ok(TranscriptResult {
              text: text.trim().to_string(),
              confidence: None,    // sherpa-onnx doesn't expose confidence
              is_partial: false,   // Non-streaming (OfflineRecognizer)
          })
      }
  }
  ```

  **Must NOT do**:
  - Do NOT add confidence scoring (not available from sherpa-onnx OfflineRecognizer)
  - Do NOT add streaming/partial results
  - Do NOT modify the `TranscriptionProvider` trait or `TranscriptResult` struct
  - Do NOT parse or transform emotion markers (they come inline from sherpa-onnx)

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. Unit test: `SherpaOnnxProvider::new()` with valid dir → Ok
  3. Unit test: `SherpaOnnxProvider::new()` with invalid dir → Err
  4. Unit test: `transcribe(vec![0.0; 16000], None)` → `TranscriptResult` with `confidence: None`, `is_partial: false`

  **Parallelization**: Wave 2 | Blocked by: 4 | Blocks: 7, 8

- [ ] 7. Wire into `audio/transcription/engine.rs` — add `"sherpa-onnx"` match arms

  **What to do**: Add `"sherpa-onnx"` routing in `validate_transcription_model_ready()` and `get_or_init_transcription_engine()` in `engine.rs`. Follow the exact same pattern as the FunASR provider.

  **File to modify**:
  - `frontend/src-tauri/src/audio/transcription/engine.rs`

  **In `validate_transcription_model_ready()`** (after FunASR arm, before `other =>` catch-all):
  ```rust
  "sherpa-onnx" => {
      info!("🔍 Validating Sherpa-ONNX model...");
      let app_data_dir = app.path().app_data_dir()
          .map_err(|e| format!("Failed to get app data dir: {}", e))?;
      let model_dir = app_data_dir.join("models").join("sherpa-onnx").join("sensevoice");
      if !model_dir.join("sensevoice-encoder-2.5-192-768.onnx").exists() {
          return Err(format!(
              "Sherpa-ONNX model not found at {}. Please place sensevoice-encoder-2.5-192-768.onnx, \
               sensevoice-decoder-2.5-192-768.onnx, and tokens.txt in this directory.",
              model_dir.display()
          ));
      }
      info!("✅ Sherpa-ONNX model directory found at {}", model_dir.display());
      Ok(())
  }
  ```

  **In `get_or_init_transcription_engine()`** (after FunASR arm, before `other =>` catch-all):
  ```rust
  "sherpa-onnx" => {
      info!("🎯 Initializing Sherpa-ONNX transcription engine");
      let app_data_dir = app.path().app_data_dir()
          .map_err(|e| format!("Failed to get app data dir: {}", e))?;
      let model_dir = app_data_dir.join("models").join("sherpa-onnx").join("sensevoice");
      let provider = crate::sherpa_onnx::SherpaOnnxProvider::new(&model_dir)
          .map_err(|e| format!("Failed to initialize Sherpa-ONNX: {}", e))?;
      Ok(TranscriptionEngine::Provider(Arc::new(provider)))
  }
  ```

  **Update the error catch-all** to include `"sherpa-onnx"` in the list of valid providers:
  ```rust
  other => {
      warn!("❌ Unsupported transcription provider for local recording: {}", other);
      Err(format!(
          "Provider '{}' is not supported for local transcription. \
           Please select 'localWhisper', 'parakeet', 'funasr', or 'sherpa-onnx'.",
          other
      ))
  }
  ```

  **Must NOT do**:
  - Do NOT create a new `TranscriptionEngine` variant (use `Provider(Arc<...>)`)
  - Do NOT modify existing provider match arms
  - Do NOT add global static state

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. Setting provider to `"sherpa-onnx"` resolves correctly in both validate and init functions
  3. The error message for invalid providers includes `"sherpa-onnx"` as an option

  **Parallelization**: Wave 3 | Blocked by: 6 | Blocks: 8

- [ ] 8. Register module in `lib.rs`

  **What to do**: Register the `sherpa_onnx` module, register the Tauri command, and log the model directory location at startup.

  **File to modify**:
  - `frontend/src-tauri/src/lib.rs`

  **Changes**:

  1. Add `mod sherpa_onnx;` and re-export the provider:
  ```rust
  pub mod sherpa_onnx;
  // Re-export for engine.rs access
  pub use audio::transcription::sherpa_onnx_provider::SherpaOnnxProvider;
  ```

  2. Register the Tauri command (in the `.invoke_handler(tauri::generate_handler![...])` list):
  ```rust
  crate::sherpa_onnx::commands::api_scan_sherpa_onnx_models,
  ```

  3. Log model directory at startup (after existing model directory logging):
  ```rust
  // Log Sherpa-ONNX model directory location
  let sherpa_onnx_dir = _app.handle().path().app_data_dir()
      .map(|d| d.join("models").join("sherpa-onnx").join("sensevoice"))
      .unwrap_or_default();
  std::fs::create_dir_all(&sherpa_onnx_dir).ok();
  log::info!("📁 Sherpa-ONNX model directory: {}", sherpa_onnx_dir.display());
  ```

  **Must NOT do**:
  - Do NOT add any global static singletons
  - Do NOT modify existing module registrations or command registrations

  **Acceptance criteria**:
  1. `cargo build -p meetily` succeeds
  2. `cargo check -p meetily` succeeds
  3. The `api_scan_sherpa_onnx_models` command is registered and invocable

  **Parallelization**: Wave 3 | Blocked by: 5, 6, 7 | Blocks: 9, 10, 11

- [ ] 9. Create `frontend/src/components/SherpaOnnxModelManager.tsx`

  **What to do**: Create a React component that scans for available Sherpa-ONNX models, shows their status, and allows selecting the active model. Mirrors the pattern of `FunasrModelManager.tsx`.

  **File to create**:
  - `frontend/src/components/SherpaOnnxModelManager.tsx`

  **Component API**:
  ```tsx
  interface ModelInfo {
    name: string;
    modelType: string;
    path: string;
    ready: boolean;
    details: string | null;
  }

  interface SherpaOnnxModelManagerProps {
    // Controlled by parent settings component
    currentModel: string;
    onModelSelect: (modelType: string) => void;
  }
  ```

  **Behavior**:
  1. On mount, invokes `invoke("api_scan_sherpa_onnx_models")` to discover models
  2. Displays a list of found models with Ready/Missing status indicators
  3. Shows a select/radio UI to pick the active model
  4. No download button (Phase 1 — manual placement only)
  5. Gracefully handles the case where no models directory exists yet

  **UI elements**:
  - Section heading: "Sherpa-ONNX Models"
  - Status description text explaining manual placement
  - Model list with status badges (green Ready / red Missing)
  - A disabled state when no models are available

  **Must NOT do**:
  - Do NOT add a download button or file picker (Phase 2)
  - Do NOT add model deletion UI
  - Do NOT use any external UI library beyond existing shadcn/ui components

  **Acceptance criteria**:
  1. Component renders without errors
  2. Calls `api_scan_sherpa_onnx_models` on mount
  3. Displays model status correctly
  4. Calls `onModelSelect` when user selects a model

  **Parallelization**: Wave 4 | Blocked by: 8 | Blocks: none

- [ ] 10. Add `"sherpa-onnx"` to `TranscriptSettings.tsx`

  **What to do**: Add `"sherpa-onnx"` to the provider type union and add a `<SelectItem>` in the provider selection dropdown.

  **File to modify**:
  - `frontend/src/app/transcription/TranscriptSettings.tsx`

  **Changes**:

  1. Add to the `TranscriptProvider` type union (around line 13):
  ```typescript
  type TranscriptProvider = 'localWhisper' | 'parakeet' | 'funasr' | 'deepgram' | 'elevenLabs' | 'groq' | 'openai' | 'sherpa-onnx';
  ```

  2. Add `<SelectItem>` in the provider selection UI (after the FunASR item):
  ```tsx
  <SelectItem value="sherpa-onnx">🎯 Sherpa-ONNX (SenseVoice)</SelectItem>
  ```

  **Must NOT do**:
  - Do NOT change the component's internal state management pattern
  - Do NOT add conditional rendering specific to sherpa-onnx in this file (defer to `SherpaOnnxModelManager.tsx`)

  **Acceptance criteria**:
  1. The `"sherpa-onnx"` string is present in the `TranscriptProvider` type
  2. The dropdown shows "Sherpa-ONNX (SenseVoice)" as a selectable option
  3. Selecting it from the dropdown triggers the same provider-change flow

  **Parallelization**: Wave 4 | Blocked by: 8 | Blocks: none

- [ ] 11. Add `"sherpa-onnx"` to frontend hook for model config

  **What to do**: Add `"sherpa-onnx"` to the provider list in the frontend hook that manages model configuration.

  **File to modify**:
  - `frontend/src/hooks/meeting-details/useModelConfiguration.ts`

  **Changes**:

  1. Add `"sherpa-onnx"` to the provider list or type union within the hook
  2. If there's provider-specific model path resolution, add a case for `"sherpa-onnx"` mapping to `"models/sherpa-onnx/sensevoice/"`
  3. Ensure the hook's exposed state includes the sherpa-onnx option

  **Must NOT do**:
  - Do NOT add model download logic
  - Do NOT change the hook's public API signature

  **Acceptance criteria**:
  1. `pnpm run lint` passes in `frontend/`
  2. TypeScript compilation succeeds
  3. The hook returns valid configuration when `"sherpa-onnx"` is selected

  **Parallelization**: Wave 4 | Blocked by: 8 | Blocks: none

## Final verification wave

- [x] F1. Plan compliance audit
  - Verify: All 11 todos completed, exactly the scoped deliverables exist
  - Verify: No scope creep — no auto-download, no diarization, no Paraformer, no new Tauri events
  - Verify: No `TranscriptionEngine::SherpaOnnx` variant added (must use `Provider(Arc<...>)`)
  - Verify: No changes to existing providers (Whisper, Parakeet, FunASR)
  - Verify: No changes to `TranscriptionProvider` trait or `TranscriptResult`
  - Evidence: `.omo/evidence/f1-plan-compliance.md`

- [x] F2. Code quality review
  - Verify: `cargo clippy -p meetily` clean (no warnings introduced)
  - Verify: No `unsafe` blocks in `sherpa_onnx/` module
  - Verify: All public items documented with doc comments
  - Verify: `SherpaOnnxError` enum has `#[error(...)]` on all variants
  - Verify: Each file ≤250 LOC (Cargo.toml changes excepted)
  - Verify: No `unwrap()` or `expect()` in production paths (tests may use them)
  - Evidence: `.omo/evidence/f2-code-quality.md`

- [x] F3. Real manual QA
  - Verify: `cargo test -p meetily` all tests pass
  - Verify: Building with `cargo build -p meetily` completes without errors
  - Verify: Integration test with real SenseVoice model + real audio → meaningful text output
  - Verify: All existing providers (Whisper, Parakeet, FunASR) still work unchanged
  - Verify: Retranscription with Sherpa-ONNX works (via `get_or_init_transcription_engine()`)
  - Evidence: `.omo/evidence/f3-manual-qa.md`

- [x] F4. Scope fidelity
  - Verify: `git diff --stat` shows only expected files changed
  - Verify: No changes to `parakeet_engine/`, `whisper_engine/`, or `funasr_onnx/` directories
  - Verify: No changes to `TranscriptionProvider` trait or `TranscriptResult`
  - Verify: No new Tauri events registered
  - Verify: No changes to `tauri.conf.json` or CSP
  - Evidence: `.omo/evidence/f4-scope-fidelity.md`

## Commit strategy

- Single commit: `feat: add Sherpa-ONNX transcription provider for SenseVoice models`
- Files created:
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs` (new)
  - `frontend/src-tauri/src/sherpa_onnx/model.rs` (new)
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs` (new)
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs` (new)
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs` (new)
  - `frontend/src/components/SherpaOnnxModelManager.tsx` (new)
- Files modified:
  - `frontend/src-tauri/Cargo.toml` (add sherpa-onnx dependency)
  - `frontend/src-tauri/src/audio/transcription/engine.rs` (add "sherpa-onnx" match arms + update error msg)
  - `frontend/src-tauri/src/lib.rs` (add `mod sherpa_onnx`, register command, log model dir)
  - `frontend/src/app/transcription/TranscriptSettings.tsx` (add provider type + SelectItem)
  - `frontend/src/hooks/meeting-details/useModelConfiguration.ts` (add provider)

## Success criteria

1. `cargo build -p meetily` passes
2. `cargo clippy -p meetily` passes with no new warnings
3. `cargo test -p meetily` all tests pass
4. `"sherpa-onnx"` is a selectable provider in app settings UI
5. Selecting Sherpa-ONNX and recording produces a non-empty transcription with ITN
6. Emotion markers (`[laughter]`, `[applause]`) appear inline in transcript text when present in audio
7. All existing providers (Whisper, Parakeet, FunASR) continue to work unchanged
8. `TranscriptionEngine` enum has no new variant (uses existing `Provider(Arc<...>)`)
9. No new Tauri events, no changes to `tauri.conf.json` or CSP
10. Model scanning via `api_scan_sherpa_onnx_models` returns correct status
