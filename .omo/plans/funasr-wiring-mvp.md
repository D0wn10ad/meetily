# funasr-wiring-mvp - Work Plan

## TL;DR (For humans)

**What you'll get:** "FunASR (Paraformer - Chinese)" appears as a selectable option in the Transcript Settings dropdown — alongside Whisper and Parakeet. Your already-placed model files at `%APPDATA%\com.meetily.ai\models\funasr\paraformer-large\` are used directly. No download UI yet (that's the next batch). Plus a written design doc showing how auto-download will work in the future.

**Why this approach:** The Rust engine is already complete and compiling — only the frontend settings page was never updated to let you select FunASR. One file change unlocks the whole feature. The auto-download design is already fully documented in the existing `.omo/plans/funasr-autodownload.md` plan.

**What it will NOT do:** Download models from HuggingFace (manual still). Show a progress bar. Wire into a FunasrModelManager component. Add DirectML GPU support.

**Effort:** Quick
**Risk:** Low — single file change, strictly additive (no deletions), follows existing Parakeet/localWhisper pattern
**Decisions to sanity-check:** Provider emoji (🎯), modelOptions entry (empty array — no multi-model selection)

Your next move: approve, or run a high-accuracy Momus review. Full execution detail follows below.

---

> TL;DR (machine): Quick, Low risk. One file edit (TranscriptSettings.tsx) + reference the existing auto-download design plan. No Rust changes.

## Scope
### Must have
- TranscriptSettings.tsx: Add `'funasr'` to the provider union type
- TranscriptSettings.tsx: Add `<SelectItem value="funasr">🎯 FunASR (Paraformer - Chinese)</SelectItem>` to the provider dropdown
- TranscriptSettings.tsx: Add `funasr: []` to `modelOptions` (no model selection)
- TranscriptSettings.tsx: Exclude `'funasr'` from `requiresApiKey`, from model-selector dropdown, from `fetchApiKey` calls, and from the API-key-clearing `useEffect`
- Create `.omo/design/funasr-autodownload-next.md` referencing the existing full plan and summarizing next steps

### Must NOT have (guardrails, anti-slop, scope boundaries)
- No changes to any Rust file (lib.rs, engine.rs, mod.rs, commands.rs — all untouched)
- No creation of funasr.ts, FunasrModelManager.tsx, or any new frontend files
- No changes to tauri.conf.json, package.json, Cargo.toml
- No changes to ParakeetModelManager or WhisperModelManager
- No refactoring of existing conditions in TranscriptSettings (additive only)
- No model auto-detection or status display (model download status is next batch)

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: tests-after (TypeScript compilation + lint)
- Evidence: `.omo/evidence/task-1-funasr-wiring-mvp.txt`

## Execution strategy
### Parallel execution waves
- Wave 1 (sequential): Task 1 (TranscriptSettings edit) → Task 2 (design doc)

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1. TranscriptSettings.tsx | — | 2 | — |
| 2. Design doc | 1 | — | — |

## Todos
> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->
- [x] 1. `TranscriptSettings.tsx`: Add 'funasr' provider option
  What to do / Must NOT do:
  - Edit `frontend/src/components/TranscriptSettings.tsx`
  - **Provider union** (line 13): Change `'localWhisper' | 'parakeet' | 'deepgram'` to `'localWhisper' | 'parakeet' | 'funasr' | 'deepgram'`
  - **modelOptions** (lines 53-60): Add `funasr: [],` after `parakeet: [],`
  - **requiresApiKey** (line 61): Add `&& transcriptModelConfig.provider !== 'funasr'` to the condition
  - **useEffect apiKey clearer** (line 37): Change `provider === 'localWhisper' || provider === 'parakeet'` to also `|| provider === 'funasr'`
  - **onValueChange handler** (line 115): Change `provider !== 'localWhisper' && provider !== 'parakeet'` to also `&& provider !== 'funasr'`
  - **SelectContent** (lines 123-130): Add `<SelectItem value="funasr">🎯 FunASR (Paraformer - Chinese)</SelectItem>` after the parakeet SelectItem
  - **Model selector dropdown** (line 133): Change `uiProvider !== 'localWhisper' && uiProvider !== 'parakeet'` to also `&& uiProvider !== 'funasr'`
  - Note: Do NOT add a FunasrModelManager render block (next batch)
  - Do NOT change handleWhisperModelSelect, handleParakeetModelSelect, or any other function
  - Do NOT remove or modify the parakeet/localWhisper entries

  Parallelization: Wave 1 | Blocked by: — | Blocks: 2
  References:
  - Current file: `frontend/src/components/TranscriptSettings.tsx` (234 lines, in worktree)
  - Provider union: line 13
  - modelOptions: lines 53-60
  - requiresApiKey: line 61
  - useEffect: lines 36-40
  - onValueChange: lines 112-118
  - SelectContent: lines 123-130
  - Model selector condition: line 133
  - Engine provider string: `frontend/src-tauri/src/audio/transcription/engine.rs:150,252` (expects `"funasr"`)
  - Pattern reference: Parakeet SelectItem at line 124 (same structure)

  Acceptance criteria (agent-executable):
  - `grep -n "'funasr'" frontend/src/components/TranscriptSettings.tsx` returns 5+ matches (provider union, modelOptions, requiresApiKey, useEffect, onValueChange)
  - `grep -n "FunASR" frontend/src/components/TranscriptSettings.tsx` returns 1+ match (SelectItem)
  - `pnpm run lint` passes with 0 errors from frontend/ directory

  QA scenarios:
  - Happy: `pnpm run lint` exits 0, `bun run tsc --noEmit` passes (TypeScript)
  - Failure: revert one change → `pnpm run lint` fails with type error
  - Evidence: `.omo/evidence/task-1-funasr-wiring-mvp.txt`

  Commit: N (batched at end)

- [x] 2. `.omo/design/funasr-autodownload-next.md`: Create auto-download design document referencing existing full plan
  What to do / Must NOT do:
  - Create `.omo/design/funasr-autodownload-next.md`
  - Summarize: MVP status (wired + manually placed models), next feature (auto-download)
  - Reference the existing full plan at `.omo/plans/funasr-autodownload.md` (10 todos, 374 lines)
  - List the 3 remaining feature groups:
    1. Rust download commands (commands.rs, mod.rs, lib.rs) + funasr.ts
    2. FunasrModelManager.tsx download UI
    3. DirectML GPU acceleration (Cargo.toml, parakeet model.rs, funasr model.rs)
  - Note key design decisions: HF→ModelScope fallback, weighted progress (95/2.5/2.5), file renaming model_quant.onnx→model.onnx, cancellation via AtomicBool
  - Must NOT repeat the full plan content (just reference + summary)
  - Must NOT include implementation details better covered by the existing plan

  Parallelization: Wave 1 | Blocked by: 1 | Blocks: —
  References:
  - Full design: `.omo/plans/funasr-autodownload.md` (374 lines, all 10 todos with exact file paths and patterns)
  - Current state: `.omo/drafts/funasr-wiring-mvp.md` (scope decisions for this MVP)

  Acceptance criteria:
  - File exists at `.omo/design/funasr-autodownload-next.md`
  - Contains references to `.omo/plans/funasr-autodownload.md`
  - Lists all 3 remaining feature groups

  QA scenarios:
  - File existence + content grep check
  - Evidence: `.omo/evidence/task-2-funasr-wiring-mvp.txt`

  Commit: N (batched at end)

## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
- [x] F1. Plan compliance audit — Only TranscriptSettings.tsx changed, no Rust/files outside scope
- [x] F2. Code quality review — Follows existing parakeet/localWhisper pattern, additive only
- [x] F3. Build verification — `cargo build -p meetily` passes (0 errors), `lint` passes
- [x] F4. Scope fidelity — No FunasrModelManager, no funasr.ts, no Rust changes, no DirectML

## Commit strategy
Single atomic commit after MVP verification passes:
```
feat(funasr): wire FunASR provider into TranscriptSettings UI

- Add 'funasr' to provider union, dropdown, and conditions
- FunASR engine already complete on Rust side
- Manual model placement at %APPDATA%/Meetily/models/funasr/paraformer-large/
- Auto-download design documented for next batch
```

Files to commit:
- `frontend/src/components/TranscriptSettings.tsx` (modified)
- `.omo/design/funasr-autodownload-next.md` (new)

## Success criteria
- User can select "FunASR (Paraformer - Chinese)" in Transcript Settings dropdown
- No API key field shown for FunASR (local provider)
- No model selection dropdown shown for FunASR (single model)
- Rust backend already uses model files from the manually-placed directory
- Design doc ready for next implementation batch: auto-download + DirectML
