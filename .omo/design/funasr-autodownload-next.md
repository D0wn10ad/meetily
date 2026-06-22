# FunASR Auto-Download — Next Implementation Batch

## Current MVP Status
- FunASR engine: ✅ Complete (5 Rust files + provider + lib.rs wiring)
- TranscriptSettings UI: ✅ FunASR selectable in provider dropdown
- Model placement: Manual at `<app_data>/models/funasr/paraformer-large/`
- GPU acceleration: ❌ CPU-only (planned for next batch)

## Next Batch: Auto-Download + DirectML

Full detailed design at `.omo/plans/funasr-autodownload.md` (10 todos, 374 lines).

### Feature Group 1: Rust Download Commands + TS API
- `funasr_onnx/commands.rs`: `#[command] async fn funasr_download_model` and `funasr_cancel_download`
- Mirror logic: HuggingFace (`funasr/Paraformer-large`) → ModelScope (`damo/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-onnx`)
- Files: `model_quant.onnx→model.onnx` (238 MB), `am.mvn` (~10 KB), `tokens.json` (~300 KB)
- Weighted progress: 95/2.5/2.5, emits `funasr-model-download-progress` events
- Cancellation via `static CANCEL_DOWNLOAD: AtomicBool`
- `funasr_onnx/mod.rs`: Add `pub mod commands;`
- `lib.rs`: Register commands in `generate_handler![]`
- `frontend/src/lib/funasr.ts`: TypeScript API class with `downloadModel()` and `cancelDownload()`

### Feature Group 2: Download UI
- `frontend/src/components/FunasrModelManager.tsx`: Progress bar, status states, single Download button
- No multi-model selection (only paraformer-large)
- Event listeners for `funasr-model-download-*`

### Feature Group 3: DirectML GPU Acceleration
- `frontend/src-tauri/Cargo.toml`: Change `ort` dep to `ort = { version = "2.0.0-rc.10", features = ["directml"] }`
- Add feature flag `ort-dml = []`
- `parakeet_engine/model.rs`: Add `DirectMLExecutionProvider` with CPU fallback, `#[cfg]`-gated
- `funasr_onnx/model.rs`: Same DirectML + CPU fallback
- Note: `DirectMLExecutionProvider` requires `features = ["directml"]` on `ort` crate (currently `#[cfg(any(feature = "load-dynamic", feature = "directml"))]` gated)

### Key Design Decisions
- HF first, ModelScope fallback (automatic, no user choice)
- File renaming: `model_quant.onnx` → `model.onnx` on save
- Files validated by existence check only (no content verification)
- No new Cargo dependencies (uses existing `reqwest`, `serde_json`)
