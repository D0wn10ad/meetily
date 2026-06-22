# funasr-onnx-integration - Work Plan

## TL;DR (For humans)

**What you'll get:** A new ASR option "FunASR" alongside "Whisper" and "Parakeet" in the app settings. When selected, Meetily will transcribe meetings using Paraformer ONNX models (dramatically better Chinese accuracy) instead of Parakeet — all locally, all in-process, no cloud.

**Why this approach:** FunASR models (Paraformer) are available as standard ONNX files, and Meetily already uses the `ort` crate for ONNX inference — plus `realfft` for FFT and all the audio processing infra we need. This means we add FunASR as pure Rust with zero sidecars or Python wrappers, using the exact same architecture as the existing Parakeet engine.

**What it will NOT do:** It won't include SenseVoiceSmall support (that's follow-up work). It won't touch the Parakeet or Whisper engines — you can switch between them freely. It won't have a model download UI in v1 (models go on disk manually). It won't support streaming or partial results.

**Effort:** Short
**Risk:** Low — follows the exact same patterns as the existing Parakeet engine, reuses existing crate dependencies
**Decisions to sanity-check:** Frontend FBank parameters (16kHz, Hamming 25ms/10ms, 80 mel bins, LFR m=7/n=6) are hardcoded to match Paraformer's training config; `"funasr"` provider string must be added to both Rust AND frontend TS code

Your next move: Approve the plan so I write the code, or request edits.

---

> **TL;DR (machine)**: Short effort, low risk. Add Paraformer ONNX via existing `ort`+`realfft` crates. 4 implementation todos + 1 final verification wave.

## Scope
### Must have
1. Pure Rust FBank+LFR+CMVN frontend (ported from funasr_onnx Python) as `funasr_onnx/frontend.rs`
2. Paraformer ONNX model inference via `ort` crate as `funasr_onnx/model.rs`
3. Token decoding (argmax + blank/eos filter + tokens.json lookup) as `funasr_onnx/decoder.rs`
4. `FunasrProvider` implementing `TranscriptionProvider` trait, registered via existing `Provider(Arc<dyn TranscriptionProvider>)` variant
5. Registration in engine selection (`engine.rs` + `lib.rs` + `transcription/mod.rs`), model directory setup in `lib.rs`
6. Frontend: Add `"funasr"` to provider type union + `<SelectItem>` in `TranscriptSettings.tsx`
7. Tests: frontend matches Python reference output, end-to-end transcription works

### Must NOT have (guardrails, anti-slop, scope boundaries)
- NO SenseVoiceSmall or multi-lingual support (deferred)
- NO streaming/partial results (Paraformer is single-pass)
- NO model download UI or fancy model management in v1
- NO changes to `TranscriptionProvider` trait itself
- NO changes to Parakeet or Whisper engine code
- NO new Tauri commands or events
- NO new FFT crate (`realfft` is already in Cargo.toml and used by existing code)
- NO new enum variant on `TranscriptionEngine` — use existing `Provider(Arc<dyn TranscriptionProvider>)`

## Verification strategy
- Test decision: tests-after (we have Python reference to test against)
- Unit tests for frontend: synthetic sine wave → known FBank output
- Integration test: transcribe real audio file → verify text is non-empty
- **Test fixtures**: A 1-second 440Hz sine wave WAV at 16kHz (generated in test via `generate_sine_wave()` helper) → frontend output compared against manually checked shape/dims.
- Python reference comparison: Run `python3 scripts/generate_fbank_fixture.py <sine_wav> --output fixtures/fbank_ref.bin` to capture reference FBank output from FunASR Python. Assert `abs_diff(fbank_rust, fbank_python) < 1e-4`. (This is a manual setup step, not part of automated CI.)
- Evidence: `.omo/evidence/task-*-funasr-onnx-integration.txt`

## Execution strategy
### Parallel execution waves
Wave 1: Todos 1, 2, 3 (no shared mutable state — all independent)
Wave 2: Todo 4 (needs 1+2+3 for integration)
Final Wave: F1-F4 (needs all todos)

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1. Frontend (FBank+LFR+CMVN) | — | 4 | 2, 3 |
| 2. Model (ONNX sessions) | — | 4 | 1, 3 |
| 3. Decoder (token decoding) | — | 4 | 1, 2 |
| 4. Provider + registration | 1, 2, 3 | — | — |

## Todos

- [ ] 1. Create `funasr_onnx/frontend.rs` — FBank + LFR + CMVN preprocessing

  **What to do**: Port the `WavFrontend` class from `funasr_onnx/utils/frontend.py` to pure Rust. This is the core signal-processing module.

  **Files to create**:
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs`
  - `frontend/src-tauri/src/funasr_onnx/mod.rs` (initial — just declares `mod frontend;`)

  **Struct definition**:
  ```rust
  pub struct FunasrFrontend {
      cmvn_means: Vec<f32>,   // Loaded from am.mvn
      cmvn_vars: Vec<f32>,    // Loaded from am.mvn
  }

  impl FunasrFrontend {
      pub fn new(cmvn_path: &Path) -> Result<Self, FunasrError>;
      /// Process 16kHz mono f32 PCM → (features_560d, feats_len)
      pub fn process(&self, audio: &[f32]) -> Result<(Vec<f32>, i32), FunasrError>;
  }
  ```

  **Algorithm to implement (step by step)**:

  1. **Scaling**: `waveform * 32768.0` (int16 range)
  2. **FBank** — using `realfft` crate (already in Cargo.toml) and manual mel filterbank:
     - Hamming window (25ms = 400 samples at 16kHz): `w[n] = 0.54 - 0.46 * cos(2πn/N)`, N=400
     - Frame shift 10ms = 160 samples
     - FFT: `RealFftPlanner::new().plan_fft_forward(512)` (pad to power of 2)
     - Power spectrum: `real[0]^2 + imag[0]^2` for first 257 bins (512/2 + 1)
     - 80 Mel filterbank: Triangular filters equally spaced on mel scale
       - `mel(f) = 2595 * log10(1 + f/700)` for hz→mel
       - `mel_to_hz(m) = 700 * (10^(m/2595) - 1)` for mel→hz
       - Each filter: `minHz = bin_i`, `centerHz = bin_i+1`, `maxHz = bin_i+2`
       - Magnitude response: linear ramp up to center, linear ramp down
     - Log energy: `log(max(mel_energy, eps))` where `eps = 1e-10`
     - Output shape: `(T_fbank, 80)` where `T_fbank = floor((N_audio - 400) / 160) + 1`
  3. **LFR (Low Frame Rate)**:
     - `lfr_m = 7, lfr_n = 6` (must be module-level consts)
     - Left-pad: tile first frame `(lfr_m-1)//2 = 3` times
     - For each output frame i: stack frames `[i*n, i*n + m)` → concat to `(1, 560)` → `Vec::push`
     - Output: `(T_lfr, 560)` where `T_lfr = ceil(T_fbank / 6)`
  4. **CMVN**:
     - Parse `am.mvn`: binary Kaldi format with `<AddShift>` and `<Rescale>` markers
     - Binary format: `<AddShift> <N>` then N float32 LE values (negative means), then `<Rescale> <N>` then N float32 LE values (inverse stds)
     - Apply: `out = (feat + neg_mean) * inv_std` element-wise along feature dim (560)
  5. **Return**: Flat `Vec<f32>` of shape `(T_lfr * 560)` + `feats_len: i32 = T_lfr as i32`

  **Constants** (define at module level):
  ```rust
  pub const SAMPLE_RATE: u32 = 16000;
  pub const FRAME_LENGTH: usize = 400;     // 25ms
  pub const FRAME_SHIFT: usize = 160;      // 10ms
  pub const N_MELS: usize = 80;
  pub const LFR_M: usize = 7;
  pub const LFR_N: usize = 6;
  pub const FEAT_DIM: usize = 560;         // N_MELS * LFR_M
  pub const FFT_SIZE: usize = 512;         // Next power of 2 >= 400
  ```

  **Cargo.toml additions**: Only `realfft = "3.4.0"` if not already present (check first — it's already used in `audio/audio_processing.rs`). **Do NOT add `rustfft` or `num-traits`.**

  **Must NOT do**:
  - Do NOT add batch padding logic (single-input only for v1 — batching is unused)
  - Do NOT use `rustfft` or any new FFT crate — `realfft` is already a dependency
  - Do NOT use `unsafe` code
  - Do NOT add `num-traits`

  **References**:
  - Python reference: `funasr_onnx/utils/frontend.py` (GitHub: modelscope/FunASR/runtime/python/onnxruntime/funasr_onnx/utils/frontend.py)
  - Meetily's existing `realfft` usage: `audio/audio_processing.rs:4-5` (imports `realfft::RealFftPlanner`)
  - CMVN format: Kaldi binary format — `<AddShift>` uint32 then float32 array, `<Rescale>` uint32 then float32 array
  - Module layout: follow `parakeet_engine/model.rs` pattern for module structure

  **Parallelization**: Wave 1 | Blocked by: none | Blocks: 4

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes after adding module + frontend.rs
  2. Unit test: Feed 1-second 440Hz sine wave at 16kHz (generate in test via `generate_sine_wave(samples=16000, freq=440.0, sample_rate=16000)`) → verify output shape is `(T_lfr, 560)` and T_lfr > 0
  3. Unit test: Very short audio (<400 samples = 1 frame) → returns `Err(FunasrError::AudioTooShort)`, no panic
  4. Unit test: CMVN with known test data → verify numerical output matches formula `(x + neg_mean) * inv_std`
  5. Unit test: Empty audio input → returns `Err(FunasrError::EmptyAudio)`, no panic

  **QA scenarios**:
  - Happy: Known audio → correct shape + non-NaN values
  - Error: Empty audio → descriptive error
  - Error: Very short audio → descriptive error
  - Edge: Audio exactly N=400 samples (one frame) → produces correct single-frame output
  - Evidence: `.omo/evidence/task-1-funasr-onnx-integration.txt`

  **Commit**: N (part of larger feature commit)

- [ ] 2. Create `funasr_onnx/model.rs` — Paraformer ONNX session management

  **What to do**: Implement `FunasrModel` that loads and runs Paraformer ONNX models via the `ort` crate. Mirrors the `ParakeetModel` pattern from `parakeet_engine/model.rs`.

  **Files to create**:
  - `frontend/src-tauri/src/funasr_onnx/model.rs`
  - Update `funasr_onnx/mod.rs` to add `mod model;`

  **Struct definition**:
  ```rust
  pub struct FunasrModel {
      session: Session,        // The ONNX Runtime session
      token_list: Vec<String>, // Loaded from tokens.json
  }

  impl FunasrModel {
      pub fn new(model_dir: &Path) -> Result<Self, FunasrError>;
      /// Run inference: features (flat Vec<f32>, shape (1, T, 560)), feats_len (i32)
      /// Returns (logits flat Vec<f32>, token_num i32)
      pub fn transcribe(&self, features: &[f32], feats_len: i32) -> Result<(Vec<f32>, i32), FunasrError>;
  }
  ```

  **Session creation** (exact same pattern as `parakeet_engine/model.rs:115-126`):
  ```rust
  let providers = vec![CPUExecutionProvider::default().build()];

  let session = Session::builder()?
      .with_optimization_level(GraphOptimizationLevel::Level3)?
      .with_execution_providers(providers)?
      .with_parallel_execution(true)?
      .commit_from_file(model_dir.join("model.onnx"))?;
  ```

  **Input tensors** (from export_meta.py):
  ```rust
  // "speech": shape (1, feats_len, 560), float32
  let speech_shape = [1i64, feats_len as i64, 560i64];
  // "speech_lengths": shape (1,), int32
  let speech_lengths = [feats_len]; // int32, shape [1]
  ```

  **Output tensors**:
  ```rust
  // "logits": shape (1, T, 8404), float32 — token logits per timestep
  // "token_num": shape (1,), int32 — predicted number of valid tokens
  ```

  **Model files expected on disk**:
  ```
  <model_dir>/
    model.onnx           (or model_quant.onnx for INT8 quantized)
    tokens.json          (Paraformer vocabulary — JSON array of 8404 strings)
    am.mvn               (CMVN stats — used by frontend)
  ```

  **Model directory resolution**: The model directory is passed at construction time. The caller (`FunasrProvider` / `lib.rs` setup) resolves the path from app data dir.

  **Error type** (define in `funasr_onnx/mod.rs`):
  ```rust
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
  ```
  (Derive `thiserror::Error` — already in the crate's dependency tree.)

  **Must NOT do**:
  - Do NOT add GPU execution providers yet (CPU-only like existing Parakeet)
  - Do NOT add model download logic
  - Do NOT add `unsafe` code
  - Do NOT call any Tauri API from this module

  **References**:
  - Parakeet model pattern: `parakeet_engine/model.rs` lines 115-126 (session build), 128-134 (input introspection), 184-292 (inference calls)
  - FunASR Python model loading: `paraformer_bin.py` → `OrtInferSession.__init__()` + `infer()`
  - ONNX export meta: `funasr/models/paraformer/export_meta.py` — input/output names and shapes
  - `tokens.json` format: JSON array of 8404 strings, starting with `"<blank>"`, `"<s>"`, `"</s>"`, then Chinese characters

  **Parallelization**: Wave 1 | Blocked by: none | Blocks: 4

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. Unit test: `FunasrModel::new()` with valid model directory + dummy session (mock or on-disk model) → returns `Ok`
  3. Unit test: `FunasrModel::new()` with nonexistent directory → returns `Err(FunasrError::ModelNotFound)`
  4. Unit test: `FunasrModel::new()` with invalid tokens.json → returns `Err(FunasrError::InvalidTokenFile)`
  5. Unit test: Token list loading from real tokens.json → correct len (8404) + first 3 items are blank/sos/eos

  **QA scenarios**:
  - Happy: Valid model dir → session loads, inputs/outputs inspectable
  - Error: Missing model.onnx → clear error message
  - Error: Corrupted tokens.json → clear error message
  - Notes: This test requires a real ONNX model on disk. For CI, use a mock/marker test. Document that full tests need model files manually placed in `frontend/models/funasr/paraformer-large/`.
  - Evidence: `.omo/evidence/task-2-funasr-onnx-integration.txt`

  **Commit**: N (part of larger feature commit)

- [ ] 3. Create `funasr_onnx/decoder.rs` — Paraformer token decoding

  **What to do**: Implement the decoder that converts ONNX output logits into readable text. Port from `paraformer_bin.py` `decode_one()` and `sentence_postprocess()`.

  **Files to create**:
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs`
  - Update `funasr_onnx/mod.rs` to add `mod decoder;`

  **Function signature**:
  ```rust
  /// Decode model output logits into text.
  /// logits: flat Vec<f32>, shape (T, vocab_size=8404)
  /// token_num: predicted valid token count from model
  /// token_list: vocabulary loaded from tokens.json
  /// Returns decoded text string
  pub fn decode(
      logits: &[f32],
      token_num: i32,
      token_list: &[String],
  ) -> Result<String, FunasrError>;
  ```

  **Decoding algorithm**:
  1. **Reshape**: `logits` as `(T, 8404)` where `T = logits.len() / 8404`
  2. **Argmax**: For each timestep t (0..T), find `max_idx = argmax(logits[t * 8404 .. (t+1) * 8404])` along the vocab dimension
  3. **Filter special tokens**:
     - Remove `blank_id = 0` (`` <blank> ``)
     - Remove `eos_id = 2` (`</s>`)
     - The first token (sos_id = 1) is prepended by the model and should also be skipped
  4. **Trim to valid length**: `valid_len = (token_num - 1) as usize` (pred_bias = 1 from model config). Take first `valid_len` tokens after filtering.
  5. **Token lookup**: `token_list[token_id]` for each remaining ID
  6. **Post-processing** (port from `postprocess_utils.py` `sentence_postprocess()`):
     - Join decoded tokens into a string
     - Remove any remaining `<s>`, `</s>`, `<unk>` substrings
     - For CJK: characters are single tokens, join directly
     - For mixed content: no special handling needed for Chinese-only models

  **Must NOT do**:
  - Do NOT implement beam search or any search beyond argmax (Paraformer is non-autoregressive)
  - Do NOT add SentencePiece dependency (that's for SenseVoiceSmall, deferred)
  - Do NOT add timestamp decoding
  - Do NOT use regex crate — simple string operations suffice

  **References**:
  - Python decoding: `paraformer_bin.py` method `decode_one()` — argmax → filter → ids2tokens → trim
  - Paraformer-large token list: 8404 entries in `tokens.json` (exported from model)
  - Post-processing: `funasr_onnx/utils/postprocess_utils.py` → `sentence_postprocess()`
  - ID convention: blank=0, sos=1, eos=2 (standard Paraformer export)

  **Parallelization**: Wave 1 | Blocked by: none | Blocks: 4

  **Acceptance criteria**:
  1. `cargo check -p meetily` passes
  2. Unit test: Construct logits where argmax = [1, 345, 678, 2, 0, 0] with token_list having "a", "b" at indices 345, 678 → decode returns "ab"
  3. Unit test: All-blank logits → returns empty string "" (no panic)
  4. Unit test: token_num = 0 → returns empty string "" (no panic)
  5. Unit test: Logits with only sos and eos → returns empty string ""
  6. Unit test: Real tokens.json loaded → token IDs 345, 678 map to expected characters

  **QA scenarios**:
  - Happy: Meaningful logits → correct Chinese text output
  - Error: token_num exceeds T → clamp to T (don't panic)
  - Error: token_id out of token_list bounds → skip that ID (don't panic)
  - Edge: All logits identical → argmax picks first id, which may be 0 (blank) → empty output
  - Evidence: `.omo/evidence/task-3-funasr-onnx-integration.txt`

  **Commit**: N (part of larger feature commit)

- [ ] 4. Create `FunasrProvider` + registration (engine.rs, mod.rs, lib.rs, frontend)

  **What to do**: Wire everything together. This is the biggest single todo — it connects all three previous modules and registers the provider in every integration point.

  **This todo has 6 sub-parts — all must be completed for acceptance.**

  ### 4a. Create `FunasrEngine` orchestrator

  **File**: `funasr_onnx/engine.rs` (or inline in `mod.rs` if small)
  ```rust
  pub struct FunasrEngine {
      model: FunasrModel,
      frontend: FunasrFrontend,
  }

  impl FunasrEngine {
      pub fn new(model_dir: &Path) -> Result<Self, FunasrError>;
      /// Full transcribe: frontend.process(audio) → model.transcribe(features, len) → decoder::decode(logits, token_num, token_list)
      pub fn transcribe_audio(&self, audio: &[f32]) -> Result<String, FunasrError>;
  }
  ```

  **`transcribe_audio()` flow**:
  1. `frontend.process(audio)` → `(features, feats_len)` — audio preprocessing
  2. `model.transcribe(&features, feats_len)` → `(logits, token_num)` — ONNX inference
  3. `decoder::decode(&logits, token_num, &model.token_list)` → `text` — token decoding
  4. Return `text`

  ### 4b. Create `FunasrProvider` implementing `TranscriptionProvider`

  **File**: `audio/transcription/funasr_provider.rs`
  ```rust
  use super::provider::{TranscriptionProvider, TranscriptionError, TranscriptResult};

  pub struct FunasrProvider {
      engine: FunasrEngine,
  }

  impl FunasrProvider {
      pub fn new(model_dir: &Path) -> Result<Self, FunasrError>;
  }

  #[async_trait]
  impl TranscriptionProvider for FunasrProvider {
      /// Input: 16kHz mono f32 PCM (already resampled by VAD pipeline)
      /// Language: ignored (Paraformer defaults to Chinese)
      fn provider_name(&self) -> &str { "FunASR (Paraformer)" }

      async fn is_model_loaded(&self) -> bool { true } // Model loaded at construction

      async fn get_current_model(&self) -> Option<String> { Some("paraformer-large".to_string()) }

      async fn transcribe(
          &self,
          audio: Vec<f32>,
          _language: Option<String>,
      ) -> Result<TranscriptResult, TranscriptionError> {
          let text = self.engine.transcribe_audio(&audio)
              .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;
          Ok(TranscriptResult {
              text: text.trim().to_string(),
              confidence: None,    // Paraformer doesn't produce confidence scores
              is_partial: false,   // Non-streaming model
          })
      }
  }
  ```

  ### 4c. Register in `audio/transcription/mod.rs`

  Edit `frontend/src-tauri/src/audio/transcription/mod.rs`:
  - Add `pub mod funasr_provider;` (line 8, after parakeet_provider)
  - Add `pub use funasr_provider::FunasrProvider;` (line 15, after ParakeetProvider)

  ### 4d. Register in `audio/transcription/engine.rs`

  **Engine initialization pattern**: Unlike Parakeet which uses a global singleton (`PARAKEET_ENGINE` static), FunASR will be created on-demand using the `Provider(Arc<dyn TranscriptionProvider>)` variant. This avoids needing a global static.

  **In `validate_transcription_model_ready()`** (after line 137, before `other =>` catch-all):
  ```rust
  "funasr" => {
      info!("🔍 Validating FunASR model...");
      let app_data_dir = app.path().app_data_dir()
          .map_err(|e| format!("Failed to get app data dir: {}", e))?;
      let model_dir = app_data_dir.join("models").join("funasr").join("paraformer-large");
      if !model_dir.join("model.onnx").exists() {
          return Err(format!(
              "FunASR model not found at {}. Please place model.onnx, tokens.json, and am.mvn in this directory.",
              model_dir.display()
          ));
      }
      info!("✅ FunASR model directory found at {}", model_dir.display());
      Ok(())
  }
  ```

  **In `get_or_init_transcription_engine()`** (after line 214, before `"localWhisper" | _ =>` catch-all):
  ```rust
  "funasr" => {
      info!("🎯 Initializing FunASR transcription engine");
      let app_data_dir = app.path().app_data_dir()
          .map_err(|e| format!("Failed to get app data dir: {}", e))?;
      let model_dir = app_data_dir.join("models").join("funasr").join("paraformer-large");
      let provider = crate::funasr_onnx::FunasrProvider::new(&model_dir)
          .map_err(|e| format!("Failed to initialize FunASR: {}", e))?;
      Ok(TranscriptionEngine::Provider(Arc::new(provider)))
  }
  ```

  **Update the error catch-all** (line 138 in the original `engine.rs:138-143`):
  Change the error message from listing only `localWhisper`/`parakeet` to also mention `funasr`:
  ```rust
  other => {
      warn!("❌ Unsupported transcription provider for local recording: {}", other);
      Err(format!(
          "Provider '{}' is not supported for local transcription. Please select 'localWhisper', 'parakeet', or 'funasr'.",
          other
      ))
  }
  ```

  ### 4e. Add model directory setup in `lib.rs`

  In `lib.rs`, after the Parakeet `set_models_directory()` call (line 465), add FunASR initialization.
  **Note**: FunASR doesn't use a separate `set_models_directory()` / global singleton pattern. Instead, the model path is resolved from `app_data_dir` at engine creation time. However, we should still log the expected location at startup for discoverability.

  Add in `lib.rs` setup (after line 465, before the summary engine init):
  ```rust
  // Log FunASR model directory location
  let funasr_dir = _app.handle().path().app_data_dir()
      .map(|d| d.join("models").join("funasr").join("paraformer-large"))
      .unwrap_or_default();
  std::fs::create_dir_all(&funasr_dir).ok();
  log::info!("📁 FunASR model directory: {}", funasr_dir.display());
  ```

  ### 4f. Add frontend provider selection option

  **File**: `frontend/src/app/transcription/TranscriptSettings.tsx`
  - The TS type union (around line 13): Add `"funasr"` to the `TranscriptProvider` type: `'localWhisper' | 'parakeet' | 'deepgram' | 'elevenLabs' | 'groq' | 'openai' | 'funasr'`
  - The provider selection UI (around lines 124-125): Add `<SelectItem value="funasr">🎯 FunASR (Paraformer - Chinese)</SelectItem>` — placed after the Parakeet item

  **Must NOT do**:
  - Do NOT add a `Funasr` variant to `TranscriptionEngine` enum — use existing `Provider(Arc<dyn TranscriptionProvider>)`
  - Do NOT create a global singleton for FunASR (lazy initialization via Provider variant is cleaner)
  - Do NOT modify `TranscriptionProvider` trait or `TranscriptResult` struct
  - Do NOT modify existing providers (Parakeet, Whisper)
  - Do NOT add new Tauri commands
  - Do NOT modify `tauri.conf.json` or CSP

  **References**:
  - Provider trait: `audio/transcription/provider.rs` — `TranscriptionProvider` + `TranscriptResult`
  - Parakeet provider: `audio/transcription/parakeet_provider.rs` — reference implementation
  - Engine enum: `audio/transcription/engine.rs` — see how Parakeet (direct) differs from Provider (trait-based)
  - Engine validation: `engine.rs:90-145` — match arms for each provider + the catch-all error at line 138-143
  - Engine init: `engine.rs:186-221` — match arms constructing each engine
  - `lib.rs` setup: lines 454-472 — model directory setup pattern for reference
  - Module declarations: `audio/transcription/mod.rs` lines 5-9 — where to add `pub mod funasr_provider;`
  - Frontend provider type: `TranscriptSettings.tsx` ~line 13 — TS union definition
  - Frontend UI: `TranscriptSettings.tsx` ~line 124-125 — `<SelectItem>` list
  - Engine unload: `recording_commands.rs` ~line 648 (search for `parakeet` in recording_commands.rs) — add a `"funasr"` arm if there's a provider-based unload pattern

  **Parallelization**: Wave 2 | Blocked by: 1, 2, 3 | Blocks: none

  **Acceptance criteria**:
  1. `cargo build -p meetily` succeeds (full compile)
  2. `cargo check -p meetily` succeeds
  3. Integration test: `FunasrEngine::new()` with real model dir → `transcribe_audio()` with real audio → returns non-empty String
  4. Provider test: `FunasrProvider::new()` → `transcribe(vec![0.0; 16000], None)` → returns `TranscriptResult` with `confidence: None` and `is_partial: false`
  5. Frontend test: After building, the `"funasr"` string is present in the `TranscriptProvider` type
  6. Config test: Setting provider to `"funasr"` via API → engine resolves to `TranscriptionEngine::Provider(Arc<FunasrProvider>)`

  **QA scenarios**:
  - Happy: Full pipeline with real model + real audio → non-empty Chinese text output
  - Error: Missing model directory → `TranscriptionError::TranscriptionFailed` with descriptive path
  - Config: Provider string "funasr" → resolves correctly in both validate and init functions
  - Config: Invalid provider string → existing catch-all error message includes funasr as an option
  - UI: After rebuild, TranscriptSettings shows "FunASR (Paraformer - Chinese)" option
  - Evidence: `.omo/evidence/task-4-funasr-onnx-integration.txt`

  **Commit**: Y | `feat: add FunASR ONNX transcription provider for Paraformer models`

## Final verification wave

- [ ] F1. Plan compliance audit
  - Verify: All 4 todos completed, exactly the scoped deliverables exist
  - Verify: No scope creep — no SenseVoiceSmall, no partial results, no model download UI, no Tauri commands
  - Verify: No `TranscriptionEngine::Funasr` variant added (must use `Provider(Arc<...>)`)
  - Verify: No new FFT crate (must use `realfft`)
  - Verify: `audio/transcription/mod.rs` has `pub mod funasr_provider;`
  - Evidence: `.omo/evidence/f1-plan-compliance.md`

- [ ] F2. Code quality review
  - Verify: `cargo clippy -p meetily` clean (no warnings introduced)
  - Verify: No `unsafe` blocks in funasr_onnx/ module
  - Verify: All public items documented with doc comments
  - Verify: `FunasrError` enum has `#[error(...)]` on all variants
  - Verify: Each file ≤250 LOC (Cargo.toml changes excepted)
  - Verify: No `unwrap()` or `expect()` in production paths (tests may use them)
  - Evidence: `.omo/evidence/f2-code-quality.md`

- [ ] F3. Real manual QA
  - Verify: `cargo test -p meetily` all tests pass
  - Verify: Building with `cargo build -p meetily` completes without errors
  - Verify: Integration test with real Paraformer model + real audio → meaningful Chinese text output
  - Verify: Parakeet engine still works (select provider "parakeet" → record → transcribe)
  - Verify: Whisper engine still works (select provider "localWhisper" → record → transcribe)
  - Verify: Retranscription with FunASR works (if retranscription uses `get_or_init_transcription_engine()`)
  - Evidence: `.omo/evidence/f3-manual-qa.md`

- [ ] F4. Scope fidelity
  - Verify: `git diff --stat` shows only expected files changed (listed in commit strategy below)
  - Verify: No changes to `parakeet_engine/` or `whisper_engine/` directories
  - Verify: No changes to `TranscriptionProvider` trait or `TranscriptResult`
  - Verify: No new Tauri commands registered in `lib.rs`
  - Verify: No changes to `tauri.conf.json`
  - Evidence: `.omo/evidence/f4-scope-fidelity.md`

## Commit strategy
- Single commit: `feat: add FunASR ONNX transcription provider for Paraformer models`
- Files created:
  - `frontend/src-tauri/src/funasr_onnx/mod.rs` (new)
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs` (new)
  - `frontend/src-tauri/src/funasr_onnx/model.rs` (new)
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs` (new)
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs` (new)
- Files modified:
  - `frontend/src-tauri/src/audio/transcription/mod.rs` (add mod + re-export)
  - `frontend/src-tauri/src/audio/transcription/engine.rs` (add "funasr" match arms + update error msg)
  - `frontend/src-tauri/src/lib.rs` (add `mod funasr_onnx`, model dir log)
  - `frontend/src/app/transcription/TranscriptSettings.tsx` (add provider type + SelectItem)

## Success criteria
1. `cargo build -p meetily` passes
2. `cargo clippy -p meetily` passes with no new warnings
3. `cargo test -p meetily` all tests pass
4. `"funasr"` is a selectable provider in app settings UI
5. Selecting FunASR and recording produces a non-empty Chinese transcription
6. Parakeet and Whisper engines continue to work unchanged
7. `TranscriptionEngine` enum has no new variant (uses existing `Provider(Arc<...>)`)
8. No new Tauri commands, events, or frontend pages introduced
9. No new FFT or numeric trait dependencies added to `Cargo.toml`
