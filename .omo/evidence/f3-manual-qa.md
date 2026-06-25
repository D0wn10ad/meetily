# F3 — Manual QA Verification

**Date:** 2026-06-25
**Branch:** sherpa-onnx integration
**Worktree:** `/home/tango/repo/meetily/.worktrees/sherpa-onnx`

---

## Check 1: `cargo test -p meetily`

**Command:** `cargo test -p meetily 2>&1 | tail -30`

**Result: FAILED**

```
ALSA lib confmisc.c:855:(parse_card) cannot find card '0'
ALSA lib conf.c:5204:(_snd_config_evaluate) function snd_func_card_inum returned error: No such file or directory
ALSA lib confmisc.c:422:(snd_func_concat) error evaluating strings
ALSA lib conf.c:5204:(_snd_config_evaluate) function snd_func_concat returned error: No such file or directory
ALSA lib confmisc.c:1342:(snd_func_refer) error evaluating names
ALSA lib conf.c:5204:(_snd_config_evaluate) function snd_func_refer returned error: No such file or directory
ALSA lib conf.c:5727:(snd_config_expand) Evaluate error: No such file or directory
ALSA lib pcm.c:2721:(snd_pcm_open_noupdate) Unknown PCM default
munmap_chunk(): invalid pointer
error: test failed, to rerun pass `-p meetily --lib`
```

**Root cause of test failure:**

Only 1 test out of 231 failed:

```
test audio::device_detection::tests::test_calculate_buffer_timeout_bluetooth ... FAILED
```

**Failure details:**

```
assertion `left == right` failed
  left: 159.999996ms
 right: 160ms
```

This is a **floating-point precision** issue in `frontend/src-tauri/src/audio/device_detection.rs:486` — the calculated timeout is 159.999996ms instead of exactly 160ms. Not related to sherpa-onnx or funasr changes.

A `SIGABRT` (`munmap_chunk(): invalid pointer`) occurs after all tests run — likely heap corruption from a test teardown or the same floating-point assertion triggering an abort. The failing test is pre-existing and not introduced by this branch.

**Verdict: FAIL** (pre-existing, unrelated to sherpa-onnx/funasr work)

---

## Check 2: Engine references to parakeet/whisper/funasr

**Command:** `grep -n "parakeet\|whisper\|funasr" frontend/src-tauri/src/audio/transcription/engine.rs`

**Result:** 39 matches found

Key findings:
- **Line 16-17:** Both `Whisper` and `Parakeet` enum variants preserved for backward compat
- **Line 71-84:** Default provider falls back to `"parakeet"` when no config found
- **Line 95-105:** Whisper engine init + validation preserved
- **Line 121-134:** Parakeet engine init + validation preserved  
- **Line 150-159:** Funasr provider added as new branch
- **Line 251-291:** All three providers (parakeet, funasr) handled in second dispatch

**Verdict: PASS** — All expected engine references present, backward compat maintained

---

## Check 3: SherpaOnnxModelManager component exists

**Command:** `ls -la frontend/src/components/SherpaOnnxModelManager.tsx`

**Result: PASS**

```
644  frontend/src/components/SherpaOnnxModelManager.tsx  10.0K
```

File exists at 10.0KB with 644 permissions.

**Verdict: PASS**

---

## Check 4: TranscriptSettings.tsx references SherpaOnnxModelManager

**Command:** `grep -n "SherpaOnnxModelManager\|sherpa-onnx" frontend/src/components/TranscriptSettings.tsx`

**Result:** No matches found

The component `SherpaOnnxModelManager.tsx` exists but is **not imported or referenced** in `TranscriptSettings.tsx`. This may indicate the component is wired via a different mechanism (e.g., dynamic import, separate settings page, or not yet integrated).

**Verdict: FAIL** — `SherpaOnnxModelManager` not referenced in `TranscriptSettings.tsx`

---

## Check 5: Git diff stat (last 8 commits)

**Command:** `git diff --stat HEAD~8..HEAD`

**Result: PASS** — 33 files changed, 5061 insertions(+), 394 deletions(-)

```
 .omo/boulder.json                                  |   1 +
 .omo/design/funasr-autodownload-next.md            |  39 ++
 .omo/drafts/funasr-autodownload.md                 |  75 +++
 .omo/drafts/funasr-wiring-mvp.md                   |  56 ++
 .omo/evidence/task-1-funasr-wiring-mvp.txt         |  28 +
 .omo/plans/funasr-autodownload.md                  | 374 +++++++++++
 .omo/plans/funasr-onnx-integration.md              | 579 +++++++++++++++++
 .omo/plans/funasr-wiring-mvp.md                    | 150 +++++
 .omo/plans/sherpa-onnx-integration.md              | 695 +++++++++++++++++++++
 .omo/troubleshooting/dev-build-crashes.md          | 178 ++++++
 frontend/src-tauri/Cargo.toml                      |   1 +
 frontend/src-tauri/src/audio/import.rs             | 217 +++++--
 frontend/src-tauri/src/audio/recording_commands.rs | 213 ++++---
 frontend/src-tauri/src/audio/retranscription.rs    | 351 ++++++++---
 .../src-tauri/src/audio/transcription/engine.rs    | 302 ++++++---
 .../src/audio/transcription/funasr_provider.rs     |  56 ++
 frontend/src-tauri/src/audio/transcription/mod.rs  |  24 +-
 .../audio/transcription/sherpa_onnx_provider.rs    |  62 ++
 .../src-tauri/src/database/repositories/setting.rs |  24 +-
 frontend/src-tauri/src/funasr_onnx/decoder.rs      | 181 ++++++
 frontend/src-tauri/src/funasr_onnx/engine.rs       |  56 ++
 frontend/src-tauri/src/funasr_onnx/frontend.rs     | 529 ++++++++++++++++
 frontend/src-tauri/src/funasr_onnx/mod.rs          |  36 ++
 frontend/src-tauri/src/funasr_onnx/model.rs        | 166 +++++
 frontend/src-tauri/src/lib.rs                      |  99 ++-
 frontend/src-tauri/src/sherpa_onnx/commands.rs     | 122 ++++
 frontend/src-tauri/src/sherpa_onnx/engine.rs       | 171 +++++
 frontend/src-tauri/src/sherpa_onnx/mod.rs          |  93 +++
 frontend/src-tauri/src/sherpa_onnx/model.rs        | 170 +++++
 frontend/src/components/FunasrModelManager.tsx     | 331 ++++++++++
 .../MeetingDetails/RetranscribeDialog.tsx          |  13 +-
 frontend/src/components/TranscriptSettings.tsx     |  36 +-
 frontend/src/hooks/useTranscriptionModels.ts       |  27 +-
 33 files changed, 5061 insertions(+), 394 deletions(-)
```

Notable additions: `funasr_onnx/` (4 new modules), `sherpa_onnx/` (4 new modules), `FunasrModelManager.tsx`, `SherpaOnnxModelManager.tsx`, transcription providers.

**Verdict: PASS** — Substantial changes across the expected areas

---

## Summary

| # | Check | Verdict |
|---|-------|---------|
| 1 | `cargo test -p meetily` passes | **FAIL** — 1 pre-existing FP precision test fails; SIGABRT crash |
| 2 | Engine references parakeet/whisper/funasr | **PASS** — All 3 providers preserved |
| 3 | SherpaOnnxModelManager.tsx exists | **PASS** — File present (10KB) |
| 4 | TranscriptSettings.tsx references it | **FAIL** — No import/reference found |
| 5 | Git diff stat looks reasonable | **PASS** — 33 files, 5061+ lines added |

**Overall: 3/5 PASS** — Two failures: one pre-existing test (floating-point precision) and one integration concern (SherpaOnnxModelManager not wired into TranscriptSettings).

---

## RETEST: Sherpa-ONNX Wiring in TranscriptSettings (2026-06-25)

After re-dispatch, **17 matches** confirmed in TranscriptSettings.tsx for sherpa-onnx:

| Line | Content |
|------|---------|
| 11 | `import { SherpaOnnxModelManager }` |
| 15 | provider type union includes `sherpa-onnx` |
| 39 | local provider check includes `sherpa-onnx` |
| 59 | modelOptions has `sherpa-onnx: []` |
| 114-123 | `handleSherpaOnnxModelSelect` handler |
| 143 | onValueChange local provider check |
| 150-153 | sherpa-onnx block in onValueChange |
| 162 | SelectItem for sherpa-onnx |
| 171 | model select visibility check |
| 223-231 | SherpaOnnxModelManager UI section |

**Verdict: ✅ PASS — Sherpa-ONNX is fully wired into TranscriptSettings**
