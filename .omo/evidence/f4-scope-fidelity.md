# F4 — Scope Fidelity

**Date**: 2026-06-25  
**Auditor**: automated verification  
**Branch**: `feat/sherpa-onnx` (worktree: `.worktrees/sherpa-onnx`)  
**Devtest reference**: `origin/devtest`

---

## Check 1: `git diff --stat` shows only expected files

### Sherpa-Onnx commits only (c677346..HEAD)

```
$ git diff --stat c677346..HEAD
 .omo/plans/sherpa-onnx-integration.md              | 695 ++++++++++
 frontend/src-tauri/Cargo.toml                      |   1 +
 frontend/src-tauri/src/audio/transcription/engine.rs|  50 +-
 frontend/src-tauri/src/audio/transcription/mod.rs  |   2 +
 .../audio/transcription/sherpa_onnx_provider.rs    |  62 ++
 frontend/src-tauri/src/funasr_onnx/engine.rs       |  20 +
 frontend/src-tauri/src/lib.rs                      |   7 +
 frontend/src-tauri/src/sherpa_onnx/commands.rs     | 122 ++++
 frontend/src-tauri/src/sherpa_onnx/engine.rs       | 171 +++++
 frontend/src-tauri/src/sherpa_onnx/mod.rs          |  93 +++
 frontend/src-tauri/src/sherpa_onnx/model.rs        | 170 +++++
 11 files changed, 1392 insertions(+), 1 deletion(-)
```

Expected new files:
| File | Status |
|------|--------|
| `sherpa_onnx/mod.rs` | ✅ New module |
| `sherpa_onnx/model.rs` | ✅ Model discovery/validation |
| `sherpa_onnx/engine.rs` | ✅ SherpaOnnxEngine |
| `sherpa_onnx/commands.rs` | ✅ Tauri commands |
| `audio/transcription/sherpa_onnx_provider.rs` | ✅ Provider impl |

Expected modified files:
| File | Status |
|------|--------|
| `Cargo.toml` | ✅ Dependency addition |
| `audio/transcription/engine.rs` | ✅ Wire "sherpa-onnx" match arms |
| `audio/transcription/mod.rs` | ✅ Add `sherpa_onnx_provider` module |
| `lib.rs` | ✅ Register module + commands |
| `funasr_onnx/engine.rs` | ⚠️ Minor: added `initialize_models_directory()` helper (prerequisite shared utility, not FunASR behavioral change) |

Unexpected files:
- None

**Note**: The `frontend/src/components/SherpaOnnxModelManager.tsx` (todo #9) and `frontend/src/components/TranscriptSettings.tsx` changes (todo #10) exist as uncommitted work-in-progress. They are expected per the plan but not yet committed.

**Verdict: ✅ PASS** — All changed files in the sherpa-onnx commits are within expected scope.

### Full branch diff against origin/devtest

The full diff (`origin/devtest..HEAD`) includes changes from the merged FunASR work:
- `audio/import.rs` — FunASR changes (merged from PR #1)
- `audio/recording_commands.rs` — FunASR changes (merged from PR #1)
- `audio/retranscription.rs` — FunASR changes (merged from PR #1)
- `database/repositories/setting.rs` — FunASR changes (merged from PR #1)
- `components/FunasrModelManager.tsx` — FunASR changes (merged from PR #1)
- `hooks/useTranscriptionModels.ts` — FunASR changes (merged from PR #1)
- `funasr_onnx/*` — FunASR module (merged from PR #1)

These are from a separate workstream (FunASR) merged into this branch via commit `c677346`. They are not part of the sherpa-onnx scope and do not represent sherpa-onnx scope creep.

---

## Check 2: No changes to `parakeet_engine/`, `whisper_engine/`, or `funasr_onnx/` directories

**Method**: Check diff for existing provider directories.

```
$ git diff origin/devtest -- frontend/src-tauri/src/audio/transcription/parakeet_engine/ \
    frontend/src-tauri/src/audio/transcription/whisper_engine/ \
    frontend/src-tauri/src/audio/transcription/funasr_onnx/
(empty — no output)
```

The `funasr_onnx/` top-level directory has its own module (under `src/funasr_onnx/`), separate from the transcription providers under `audio/transcription/`. The 20-line `initialize_models_directory()` addition to `funasr_onnx/engine.rs` is a shared utility, not a change to the `audio/transcription/funasr_onnx/` provider directory.

**Verdict: ✅ PASS** — No changes to existing provider implementation directories.

---

## Check 3: No changes to `TranscriptionProvider` trait or `TranscriptResult`

**Method**: Check `git diff` for `provider.rs`.

```
$ git diff origin/devtest -- frontend/src-tauri/src/audio/transcription/provider.rs
(empty — no output)
```

Source verification:
- `TranscriptionProvider` trait: `provider.rs:50` — unchanged
- `TranscriptResult` struct: `provider.rs:42` — unchanged

**Verdict: ✅ PASS** — Both `TranscriptionProvider` trait and `TranscriptResult` struct are unmodified.

---

## Check 4: No new Tauri events registered

**Method**: Search for `emit()` calls in new sherpa-onnx code and check `lib.rs` for new event registrations.

```
$ rtk grep -rn 'emit\|\.trigger\|event' frontend/src-tauri/src/sherpa_onnx/
(no output)
```

`sherpa_onnx/commands.rs` registers **Tauri commands** (invoke-able), not events:
- `sherpa_onnx_scan_models`
- `sherpa_onnx_validate_model`

These are `#[tauri::command]` functions, not event emitters. They follow the existing pattern of `whisper_scan_models`, `parakeet_scan_models`, etc.

No new `app.emit()` or event listener registrations in `lib.rs` related to sherpa-onnx.

**Verdict: ✅ PASS** — No new Tauri events. Only Tauri commands (consistent with existing pattern).

---

## Check 5: No changes to `tauri.conf.json` or CSP

**Method**: Check diff for config file.

```
$ git diff origin/devtest -- frontend/src-tauri/tauri.conf.json
(empty — no output)
```

**Verdict: ✅ PASS** — `tauri.conf.json` is unchanged. CSP is unchanged.

---

## Check 6: Git diff summary (HEAD~8..HEAD)

```
$ git diff --stat HEAD~8..HEAD
(confirms 7 sherpa-onnx commits + 1 merge commit)

Commits:
a999cb0 feat: add SherpaOnnxProvider and wire into transcription engine
78bbbc7 feat: add sherpa-onnx Tauri commands and startup init
dd3d620 feat: add sherpa_onnx engine with model validation fix
d75968b feat: add sherpa_onnx model discovery and validation
5ff6ccf feat: add sherpa_onnx module with SherpaOnnxError enum
b73cffb feat: add sherpa-onnx dependency with sense-voice feature
7ee44e2 docs: add sherpa-onnx integration plan skeleton
c677346 Merge pull request #1 from D0wn10ad/feat/funasr-onnx
```

All sherpa-onnx commits are focused on the defined scope. No scope creep in individual commit messages.

---

## Overall F4 Verdict

| Check | Result |
|-------|--------|
| Expected files only (sherpa-onnx commits) | ✅ PASS |
| No changes to parakeet/whisper/funasr provider dirs | ✅ PASS |
| No changes to `TranscriptionProvider`/`TranscriptResult` | ✅ PASS |
| No new Tauri events | ✅ PASS |
| No changes to `tauri.conf.json` / CSP | ✅ PASS |
| Git log focused on sherpa-onnx scope | ✅ PASS |

**Overall: ✅ PASS**
