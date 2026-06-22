# funasr-autodownload - Work Plan

## TL;DR (For humans)
<!-- Fill this LAST, after the detailed plan below is written, so it summarizes the REAL plan. -->
<!-- Plain English for a non-engineer: NO file paths, NO todo numbers, NO wave/agent/tool names. -->

**What you'll get:** A "Download FunASR Model" button inside the transcript settings panel. Click it and Meetily automatically downloads the Paraformer-large ONNX model (~238 MB) plus its vocabulary and normalization files — from Hugging Face first, and if that fails it tries ModelScope in China. You watch the download progress (MB, speed, %) in real time, and when it's done the FunASR provider is ready to transcribe. Plus, ONNX models (both Parakeet and FunASR) get DirectML GPU acceleration on Windows — using your Intel or AMD GPU via DirectX 12 with automatic CPU fallback.

**Why this approach:** Follow Parakeet's proven pattern (streaming HTTP with progress events) rather than inventing new download infrastructure. Dual-mirror design (HF → ModelScope fallback) works around regional access issues without adding UI complexity. DirectML is the natural GPU path on Windows (no extra SDK needed, built into DX12).

**What it will NOT do:** Download any model other than paraformer-large. Install Python dependencies. Require user to pick a mirror. Re-download if files already exist. Enable DirectML by default (opt-in via `--features ort-dml`).

**Effort:** Medium
**Risk:** Low - follows established Parakeet pattern end-to-end; `#[cfg]` gating keeps Linux safe
**Decisions to sanity-check:** Mirror fallback logic (HF first, then MS); progress weighting split (95/2.5/2.5); DirectML as opt-in feature (not default)

Your next move: approve, or run a high-accuracy Momus review. Full execution detail follows below.

---

> TL;DR (machine): Medium effort, Low risk. 10 todo items across 4 waves. Create Rust download command (HF→MS fallback), DirectML GPU support for ORT models, TS API wrapper, download UI component, wire into TranscriptSettings. Verify with cargo build + UI render test.

## Scope
### Must have
- Rust download command that downloads 3 model files from HF with ModelScope fallback
- Streaming HTTP download with progress events emitted to frontend
- `FunasrModelManager.tsx` component showing download progress (bar, MB, speed)
- Wire FunASR into TranscriptSettings.tsx (provider union, dropdown, model manager)
- Cancel download via `funasr_cancel_download` command
- File renaming: `model_quant.onnx` → `model.onnx`
- DirectML GPU acceleration for ONNX Runtime models (Parakeet + FunASR) on Windows
- `Cargo.toml`: add `dml` feature to `ort` crate + `ort-dml` feature flag
- `parakeet_engine/model.rs`: add DirectML execution provider with CPU fallback on Windows
- `funasr_onnx/model.rs`: add DirectML execution provider with CPU fallback on Windows

### Must NOT have (guardrails, anti-slop, scope boundaries)
- No new Rust dependencies (uses existing `reqwest`, `serde_json`, `tauri`)
- No configurable mirror selection UI (automatic fallback only)
- No download resume/retry beyond simple HTTP Range (keep it KISS like Parakeet)
- No changes to `tauri.conf.json` or `package.json`
- No testing on actual FunASR model weights (unit tests only)
- No refactoring existing FunASR backend code (except adding DirectML EP)
- No model verification after download (just file-existence check)

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: tests-after (download commands are async, hard to unit test without HTTP mocks; focus on compilation + UI rendering)
- Evidence: `.omo/evidence/task-<N>-funasr-autodownload.txt`

## Execution strategy
### Parallel execution waves
- **Wave 1** (parallel: Rust + TS): Task 1 (commands.rs) + Task 4 (funasr.ts) — independent
- **Wave 1.5** (parallel with Wave 1, independent): Task 7 (Cargo.toml dml) + Task 8 (parakeet DirectML) + Task 9 (funasr DirectML) — all in one Rust subagent
- **Wave 2** (sequential on Wave 1): Task 2 (mod.rs) + Task 3 (lib.rs register) + Task 5 (FunASRModelManager.tsx)
- **Wave 3** (depends on Wave 2): Task 6 (TranscriptSettings.tsx wiring)
- **Wave 4** (final, depends on Wave 1.5): Task 10 (verification)

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1. Commands.rs | — | 2, 3 | 4, 7, 8, 9 |
| 2. mod.rs | 1 | — | — |
| 3. lib.rs register | 1 | — | — |
| 4. funasr.ts | — | — | 1, 7, 8, 9 |
| 5. FunasrModelManager.tsx | — | 6 | 1, 2, 3, 4, 7, 8, 9 |
| 6. TranscriptSettings.tsx | 5 | 10 | — |
| 7. Cargo.toml dml | — | 8, 9 | 1, 4 |
| 8. parakeet DirectML | 7 | — | 1, 4 |
| 9. funasr DirectML | 7 | — | 1, 4 |
| 10. Verification | 6, 8, 9 | — | — |

## Todos
> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->
- [ ] 1. `funasr_onnx/commands.rs`: Create download command with HF→ModelScope mirror fallback
  What to do / Must NOT do:
  - Create `frontend/src-tauri/src/funasr_onnx/commands.rs`
  - Define `#[command] async fn funasr_download_model<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String>`
  - Download 3 files to `<app_data>/models/funasr/paraformer-large/`:
    - `model_quant.onnx` → save as `model.onnx` (238 MB)
    - `am.mvn` (save as-is, ~10 KB)
    - `tokens.json` (save as-is, ~300 KB)
  - Mirror logic: Try HuggingFace first (`https://huggingface.co/funasr/Paraformer-large/resolve/main/{file}`). For `tokens.json` on HF, use `funasr/paraformer-zh` model. If any file fails, retry with ModelScope (`https://modelscope.cn/api/v1/models/damo/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-onnx/repo?Revision=master&FilePath={file}`)
  - Use streaming `reqwest::get().await` with `response.bytes_stream()` and `futures_util::StreamExt`
  - Weighted progress emission: model.onnx=95%, am.mvn=2.5%, tokens.json=2.5%
  - Emit `funasr-model-download-progress` with payload: `{ progress: u8, downloaded_mb: f64, total_mb: f64, speed_mbps: f64, status: "downloading"|"completed"|"error"|"cancelled" }`
  - Emit `funasr-model-download-complete` on success
  - Emit `funasr-model-download-error` on failure
  - Define `static CANCEL_DOWNLOAD: AtomicBool` for cancellation
  - Define `#[command] async fn funasr_cancel_download() -> Result<(), String>` that sets flag
  - Must NOT add new Cargo dependencies
  - Must NOT import anything from `parakeet_engine/` (no code sharing across engines)
  - Must NOT write to `target/` or any non-app-data path
  - Store `DownloadProgress` struct locally (don't import from parakeet_engine)
  
  Parallelization: Wave 1 | Blocked by: — | Blocks: 2, 3
  References (executor has NO interview context - be exhaustive):
  - Template: `frontend/src-tauri/src/parakeet_engine/commands.rs:387-469` (download_model pattern)
  - Template: `frontend/src-tauri/src/parakeet_engine/parakeet_engine.rs:38-50` (DownloadProgress struct)
  - Current funasr module: `frontend/src-tauri/src/funasr_onnx/` (mod.rs, frontend.rs, model.rs, decoder.rs, engine.rs)
  - Model dir: `lib.rs:466-473` (path resolution pattern)
  - Event emission: `parakeet_engine/commands.rs:413-427` (emit pattern)
  - Cancellation: `parakeet_engine/commands.rs:596-618` (cancel_download pattern)
  - HF URLs: `funasr/Paraformer-large` has `model_quant.onnx` + `am.mvn`; `funasr/paraformer-zh` has `tokens.json`
  - MS URL: `damo/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-onnx` (has ALL 3 files)
  - MS download pattern: `https://modelscope.cn/api/v1/models/{model_id}/repo?Revision=master&FilePath={file}`
  - MS file listing: same URL with `/repo/files?Recursive=true`
  
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` passes with no errors (only pre-existing warnings allowed)
  - `grep -n "funasr_download_model" frontend/src-tauri/src/funasr_onnx/commands.rs` returns 2+ lines
  
  QA scenarios (name the exact tool + invocation): happy + failure, Evidence .omo/evidence/task-1-funasr-autodownload.txt
  - Happy: `cargo check -p meetily` exits 0
  - Failure: remove required function → `cargo check` fails with appropriate error
  
  Commit: N (batched at end)

- [ ] 2. `funasr_onnx/mod.rs`: Export commands module
  What to do / Must NOT do:
  - Add `pub mod commands;` after existing module declarations
  - Do NOT change anything else in the file
  
  Parallelization: Wave 2 | Blocked by: 1 | Blocks: —
  References: `frontend/src-tauri/src/funasr_onnx/mod.rs:26-36` (existing module layout)
  
  Acceptance criteria: `cargo check -p meetily` passes
  
  QA scenarios: compilation check, Evidence .omo/evidence/task-2-funasr-autodownload.txt
  
  Commit: N (batched at end)

- [ ] 3. `lib.rs`: Register funasr_download_model command
  What to do / Must NOT do:
  - In `frontend/src-tauri/src/lib.rs`, add `funasr_onnx::commands::funasr_download_model,` to the `generate_handler![]` macro list
  - Add `funasr_onnx::commands::funasr_cancel_download,` nearby
  - Place them near the parakeet commands section (around line 590-600) with a comment `// FunASR engine commands`
  - Do NOT change `pub mod funasr_onnx;` (already exists at line 45)
  - Do NOT add model dir setup (already exists at lines 466-473)
  
  Parallelization: Wave 2 | Blocked by: 1 | Blocks: —
  References: `frontend/src-tauri/src/lib.rs:590-600` (parakeet command registration)
  
  Acceptance criteria: `cargo check -p meetily` passes; `grep -n "funasr_download_model" frontend/src-tauri/src/lib.rs` returns match
  
  QA scenarios: compilation check, Evidence .omo/evidence/task-3-funasr-autodownload.txt
  
  Commit: N (batched at end)

- [ ] 4. `frontend/src/lib/funasr.ts`: Create TypeScript API class
  What to do / Must NOT do:
  - Create `frontend/src/lib/funasr.ts`
  - Export `interface FunasrDownloadProgress { progress: number; downloaded_mb: number; total_mb: number; speed_mbps: number; status: string; }`
  - Export `class FunasrAPI` with:
    - `static async downloadModel(): Promise<void>` — calls `invoke('funasr_download_model')`
    - `static async cancelDownload(): Promise<void>` — calls `invoke('funasr_cancel_download')`
  - Follow the same pattern as `frontend/src/lib/parakeet.ts:183-190`
  - Must NOT import anything from `./parakeet`
  - Must NOT import from `@tauri-apps/api/event` (listeners go in the component)
  
  Parallelization: Wave 1 | Blocked by: — | Blocks: 5
  References: `frontend/src/lib/parakeet.ts:183-190` (ParakeetAPI.downloadModel pattern)
  
  Acceptance criteria: File exists with correct exports; `bun test` doesn't break (if any TS tests exist)
  
  QA scenarios: file existence + grep check, Evidence .omo/evidence/task-4-funasr-autodownload.txt
  
  Commit: N (batched at end)

- [ ] 5. `FunasrModelManager.tsx`: Create download UI component
  What to do / Must NOT do:
  - Create `frontend/src/components/FunasrModelManager.tsx`
  - Follow `frontend/src/components/ParakeetModelManager.tsx` pattern (~628 LOC)
  - Key differences from Parakeet:
    - No model discovery/listing (only one model: paraformer-large)
    - No `modelOptions` or multi-model selection
    - Single button: "Download FunASR Model (Paraformer-large, 238 MB)"
    - Progress bar showing percentage, downloaded MB / 238 MB, speed Mbps
    - Status states: "Not Downloaded" → "Downloading..." → "Ready"
    - On completion: automatically select funasr as the provider
  - Use `listen()` from `@tauri-apps/api/event` for `funasr-model-download-*` events
  - Use `<AnimatePresence>` and `<motion.div>` for smooth transitions (same as Parakeet)
  - Interface: `FunasrModelManagerProps { selectedModel?: string; onModelSelect?: (name: string) => void; autoSave?: boolean; }`
  - Must NOT import from `../lib/parakeet` or `ParakeetModelManager`
  - Must NOT use `ModelCard` component (simpler than Parakeet's)
  - Must NOT show model size info beyond what's relevant
  - When download completes, call `onModelSelect('paraformer-large')` (or the name the provider expects)
  
  Parallelization: Wave 2 (but can start early since it only needs funasr.ts) | Blocked by: 4 | Blocks: 6
  References:
  - UI pattern: `frontend/src/components/ParakeetModelManager.tsx` (full file, 628 lines)
  - Progress bar: `ParakeetModelManager.tsx:400-450` (progress display pattern)
  - Event listeners: `ParakeetModelManager.tsx:74-120` (listen setup pattern)
  - Event names (frontend): `funasr-model-download-progress`, `funasr-model-download-complete`, `funasr-model-download-error`
  - TS API: `frontend/src/lib/funasr.ts` (created in task 4)
  
  Acceptance criteria: File exists; imports resolve without TS errors
  QA scenarios: file existence + grep check for key patterns, Evidence .omo/evidence/task-5-funasr-autodownload.txt
  Commit: N (batched at end)

- [ ] 6. `TranscriptSettings.tsx`: Wire FunASR provider into the UI
  What to do / Must NOT do:
  - Add `import { FunasrModelManager } from './FunasrModelManager';`
  - Add `'funasr'` to the provider union type: `'localWhisper' | 'parakeet' | 'funasr' | 'deepgram' | ...`
  - Add `<SelectItem value="funasr">🎯 FunASR (Paraformer - Chinese)</SelectItem>` to the provider SelectContent
  - Add `funasr: []` to the `modelOptions` object (no model selection needed; handled by FunasrModelManager)
  - Add render block after the parakeet block:
    ```tsx
    {uiProvider === 'funasr' && (
      <div className="mt-6">
        <FunasrModelManager
          selectedModel={transcriptModelConfig.provider === 'funasr' ? transcriptModelConfig.model : undefined}
          onModelSelect={handleParakeetModelSelect}
          autoSave={true}
        />
      </div>
    )}
    ```
  - Update `requiresApiKey` condition to exclude `funasr`:
    `const requiresApiKey = transcriptModelConfig.provider !== 'localWhisper' && transcriptModelConfig.provider !== 'parakeet' && transcriptModelConfig.provider !== 'funasr' && ...`
  - Update any other `uiProvider !== 'localWhisper' && uiProvider !== 'parakeet'` conditions to also exclude `'funasr'`
  - Must NOT duplicate the ParakeetModelManager behavior
  - Must NOT change the `handleWhisperModelSelect` or `handleParakeetModelSelect` functions
  
  Parallelization: Wave 3 | Blocked by: 5 | Blocks: 7
  References:
  - Current file: `frontend/src/components/TranscriptSettings.tsx` (in worktree, 234 lines)
  - Provider union: line 13
  - SelectItem block: lines 124-130
  - Parakeet render block: lines 165-173
  - modelOptions: lines 53-60
  - requiresApiKey: line 61
  
  Acceptance criteria: `bun run lint` passes (from frontend/); TypeScript compilation passes
  QA scenarios: lint check, Evidence .omo/evidence/task-6-funasr-autodownload.txt
  Commit: N (batched at end)

- [ ] 7. `Cargo.toml`: Add `dml` feature for `ort` crate
  What to do / Must NOT do:
  - In `frontend/src-tauri/Cargo.toml`, change ORT dependency to enable DirectML:
    ```toml
    ort = { version = "2.0.0-rc.10", features = ["dml"] }
    ```
  - Add a new feature flag to the `[features]` section:
    ```toml
    ort-dml = []  # DirectML GPU acceleration for ONNX Runtime on Windows
    ```
    (Can be used standalone or composed into `platform-default` later — for now just define it)
  - Must NOT change any other dependency or feature
  - Must NOT change `tauri.conf.json` or `package.json`
  - Do NOT add the feature to `platform-default` or `default` (user opts in explicitly)
  
  Parallelization: Wave 1.5 | Blocked by: — | Blocks: 8, 9
  References: `frontend/src-tauri/Cargo.toml:106` (current `ort` dep), `frontend/src-tauri/Cargo.toml:38-53` (features section)
  Acceptance criteria: `cargo check -p meetily --features ort-dml` passes; `grep -n '"dml"' Cargo.toml` returns match
  QA scenarios: compilation check with and without `--features ort-dml`, Evidence .omo/evidence/task-7-funasr-autodownload.txt
  Commit: N (batched at end)

- [ ] 8. `parakeet_engine/model.rs`: Add DirectML execution provider on Windows
  What to do / Must NOT do:
  - Import DirectML execution provider:
    ```rust
    #[cfg(target_os = "windows")]
    use ort::execution_providers::DirectMLExecutionProvider;
    ```
  - In `init_session()`, change the providers vec to conditionally use DirectML:
    ```rust
    #[cfg(target_os = "windows")]
    let providers: Vec<Box<dyn ort::ExecutionProvider>> = vec![
        Box::new(DirectMLExecutionProvider::default().build()),
        Box::new(CPUExecutionProvider::default().build()),
    ];
    #[cfg(not(target_os = "windows"))]
    let providers = vec![CPUExecutionProvider::default().build()];
    ```
  - Must NOT change any other logic in the file
  - Must NOT remove CPU fallback (critical for systems without D3D12)
  - Must NOT import `DirectMLExecutionProvider` on non-Windows (use `#[cfg]`)
  
  Parallelization: Wave 1.5 | Blocked by: 7 | Blocks: —
  References: `frontend/src-tauri/src/parakeet_engine/model.rs:3` (current CPU EP import), `frontend/src-tauri/src/parakeet_engine/model.rs:92` (providers vec creation), `frontend/src-tauri/src/parakeet_engine/model.rs:115-124` (session builder)
  Acceptance criteria: `cargo check -p meetily --features ort-dml` passes
  QA scenarios: compilation on Linux (cfg-gated, CPU only) + on Windows (DirectML + CPU), Evidence .omo/evidence/task-8-funasr-autodownload.txt
  Commit: N (batched at end)

- [ ] 9. `funasr_onnx/model.rs`: Add DirectML execution provider on Windows
  What to do / Must NOT do:
  - Same pattern as task 8 but in `funasr_onnx/model.rs`
  - Find the session builder call (lines 36-49 of model.rs) and add providers
  - Currently in `FunasrModel::new()`:
    ```rust
    let session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_parallel_execution(true)?
        .commit_from_file(model_path)?;
    ```
  - Add execution providers before `.commit_from_file()`:
    ```rust
    #[cfg(target_os = "windows")]
    let session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_execution_providers(vec![
            Box::new(DirectMLExecutionProvider::default().build()),
            Box::new(CPUExecutionProvider::default().build()),
        ])?
        .with_parallel_execution(true)?
        .commit_from_file(model_path)?;
    
    #[cfg(not(target_os = "windows"))]
    let session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_execution_providers(vec![
            CPUExecutionProvider::default().build(),
        ])?
        .with_parallel_execution(true)?
        .commit_from_file(model_path)?;
    ```
  - Add imports:
    ```rust
    use ort::execution_providers::CPUExecutionProvider;
    #[cfg(target_os = "windows")]
    use ort::execution_providers::DirectMLExecutionProvider;
    ```
  - Must NOT change existing `.transcribe()` method or anything else
  - Must NOT break Linux builds (must use `#[cfg]` gating)
  
  Parallelization: Wave 1.5 | Blocked by: 7 | Blocks: —
  References: `frontend/src-tauri/src/funasr_onnx/model.rs:36-49` (session builder), `frontend/src-tauri/src/funasr_onnx/model.rs:1-12` (current imports)
  Acceptance criteria: `cargo check -p meetily --features ort-dml` passes; `grep -n "DirectML" funasr_onnx/model.rs` returns 2+ lines
  QA scenarios: compilation on Linux (cfg-gated, CPU only) + on Windows (DirectML + CPU), Evidence .omo/evidence/task-9-funasr-autodownload.txt
  Commit: N (batched at end)

## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
- [ ] F1. Plan compliance audit — Every todo completed, scope satisfied, no extra files or scope creep
- [ ] F2. Code quality review — No duplicated logic, follows codebase conventions, no dead code
- [ ] F3. Build verification — `cargo build -p meetily` passes (0 errors, only pre-existing warnings)
- [ ] F4. Scope fidelity — Only files listed in Must have are touched; Must NOT have is respected
- [ ] F5. DirectML check — `parakeet_engine/model.rs` and `funasr_onnx/model.rs` use DirectML EP on Windows with CPU fallback

## Commit strategy
Single atomic commit after all verification passes:
```
feat(funasr): add auto-download from HuggingFace/ModelScope + DirectML GPU

- New funasr_onnx/commands.rs with download + cancel commands
- Dual mirror support: HuggingFace primary, ModelScope fallback
- Streaming HTTP download with real-time progress events
- FunasrModelManager UI component showing download progress
- Wire FunASR into TranscriptSettings provider selection
- TypeScript API wrapper in lib/funasr.ts
- Add DirectML GPU execution provider for ONNX Runtime on Windows
- DirectML acceleration for both Parakeet and FunASR models
```

Files to commit:
- `frontend/src-tauri/src/funasr_onnx/commands.rs` (new)
- `frontend/src-tauri/src/funasr_onnx/mod.rs` (modified)
- `frontend/src-tauri/src/funasr_onnx/model.rs` (modified — DirectML EP)
- `frontend/src-tauri/src/lib.rs` (modified)
- `frontend/src-tauri/Cargo.toml` (modified — `dml` feature on `ort`)
- `frontend/src-tauri/src/parakeet_engine/model.rs` (modified — DirectML EP)
- `frontend/src/lib/funasr.ts` (new)
- `frontend/src/components/FunasrModelManager.tsx` (new)
- `frontend/src/components/TranscriptSettings.tsx` (modified)

## Success criteria
- User can select "FunASR (Paraformer - Chinese)" in Transcript Settings
- Clicking the download button downloads all 3 model files automatically
- HF → ModelScope fallback works if HF is unavailable
- Progress updates shown in real time in the UI
- After download, FunASR is ready to transcribe
- ONNX models use DirectML GPU acceleration on Windows (with CPU fallback)
- `cargo build -p meetily` passes with 0 errors (with and without `--features ort-dml`)
