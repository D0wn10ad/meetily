# Draft: funasr-autodownload

## Intent routing
**CLEAR** - user explicitly wants auto-download for FunASR models with Hugging Face + ModelScope mirrors.

## Session context (June 20 2026)
- Worktree: `.worktrees/feat/funasr-onnx/` on branch `feat/funasr-onnx`
- Main repo: `/home/tango/repo/meetily/`
- FunASR backend already implemented in worktree (6 Rust files), but frontend NOT wired up and NO download command.

## Exploration findings

### Worktree state (feat/funasr-onnx branch)
| Area | Status |
|------|--------|
| `funasr_onnx/` module (5 files) | ✅ EXISTS - mod.rs, frontend.rs, model.rs, decoder.rs, engine.rs |
| `funasr_provider.rs` | ✅ EXISTS |
| `lib.rs` mod declaration | ✅ `pub mod funasr_onnx;` at line 45 |
| `lib.rs` model dir setup | ✅ Lines 466-473 creates `models/funasr/paraformer-large/` |
| `engine.rs` "funasr" arms | ✅ Lines 150, 252 |
| `transcription/mod.rs` | ✅ `pub mod funasr_provider;` |
| `TranscriptSettings.tsx` | ❌ NOT modified - no `'funasr'` in provider union, no UI |

### Model files needed (download targets)
| File | Size | Source location | Code name |
|------|------|----------------|-----------|
| `model_quant.onnx` | 238 MB | `funasr/Paraformer-large` (HF) / `damo/...-onnx` (MS) | `model.onnx` (renamed) |
| `am.mvn` | ~10 KB | Same models | `am.mvn` |
| `tokens.json` | ~300 KB | `funasr/paraformer-zh` (HF) / `damo/...-onnx` (MS) | `tokens.json` |

### Download URLs
**HuggingFace:** `https://huggingface.co/funasr/Paraformer-large/resolve/main/{file}`
- `model_quant.onnx`, `am.mvn` present
- `tokens.json` NOT present - must fetch from `funasr/paraformer-zh`

**ModelScope:** `https://modelscope.cn/api/v1/models/damo/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-onnx/repo?Revision=master&FilePath={file}`
- ALL 3 files present (confirmed via file listing API)
- File list endpoint: `/repo/files?Recursive=true`

### Parakeet download pattern (template)
- `parakeet_engine/commands.rs` line 387: `parakeet_download_model` command
- Uses `app_handle.emit("parakeet-model-download-progress", ...)` for progress
- Cancellation via `cancel_download_flag`
- Frontend: `ParakeetModelManager.tsx` (~628 LOC) with `listen()` for events
- TS API: `ParakeetAPI.downloadModel()` in `src/lib/parakeet.ts`
- `TranscriptSettings.tsx` renders `<ParakeetModelManager>` when `uiProvider === 'parakeet'`

## Decisions
1. **Command location**: New `funasr_onnx/commands.rs` (parallel to `parakeet_engine/commands.rs`)
2. **Mirror strategy**: Try HF first, fallback to ModelScope on per-file basis
3. **Event names**: `funasr-model-download-progress`, `funasr-model-download-complete`, `funasr-model-download-error`
4. **Progress weighting**: model.onnx=95%, am.mvn=2.5%, tokens.json=2.5%
5. **File renaming**: `model_quant.onnx` → `model.onnx` during download
6. **Frontend component**: New `FunasrModelManager.tsx` following `ParakeetModelManager.tsx` pattern
7. **Frontend wiring**: TranscriptSettings needs `'funasr'` added to provider union + SelectItem + FunasrModelManager render (was missed in initial implementation)
8. **Cancellation**: Add `funasr_cancel_download` command with AtomicBool flag like Parakeet
9. **DirectML GPU**: Add `dml` feature to `ort` crate + opt-in `ort-dml` feature flag (NOT in defaults)
10. **DirectML scope**: Both `parakeet_engine/model.rs` + `funasr_onnx/model.rs` get DirectML with CPU fallback on Windows
11. **DirectML gating**: Use `#[cfg(target_os = "windows")]` — Linux builds remain CPU-only

## Status
`plan-written` - approved, plan at `.omo/plans/funasr-autodownload.md`

## Plan summary
7 todos across 3 waves + final verification:
1. (Wave 1) `commands.rs` - Rust download command with HF→MS fallback
2. (Wave 2) `mod.rs` - export commands module
3. (Wave 2) `lib.rs` - register in generate_handler![]
4. (Wave 1) `funasr.ts` - TypeScript API wrapper
5. (Wave 2) `FunasrModelManager.tsx` - download UI component
6. (Wave 3) `TranscriptSettings.tsx` - wire funasr into provider selection
7. (Wave 1.5) `Cargo.toml` — add `dml` feature to `ort` crate + opt-in `ort-dml` feature
8. (Wave 1.5) `parakeet_engine/model.rs` — DirectML EP on Windows with CPU fallback
9. (Wave 1.5) `funasr_onnx/model.rs` — DirectML EP on Windows with CPU fallback
10. (Wave 4) Verification wave (5 checks + DirectML check)
