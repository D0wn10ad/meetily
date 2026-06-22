---
slug: funasr-wiring-mvp
status: awaiting-approval
intent: clear
pending-action: write .omo/plans/funasr-wiring-mvp.md
approach: Single-file edit to TranscriptSettings.tsx to add 'funasr' provider option; create a design doc for future auto-download feature.
---

# Draft: funasr-wiring-mvp

## Components (topology ledger)
<!-- Lock the SHAPE before depth. One row per top-level component that can succeed or fail independently. -->
- id: transcript-settings-wiring | outcome: Add 'funasr' provider to TranscriptSettings dropdown | status: active | evidence path: .
- id: design-doc | outcome: Document auto-download design (Rust commands + frontend UI + DirectML) for future implementation | status: active | evidence path: .omo/plans/funasr-autodownload.md
- id: engine-integration | outcome: FunASR engine already fully wired on Rust side (5 files + provider + lib.rs) | status: deferred (done) | evidence path: verified in lib.rs:45,466-473; engine.rs:150-168,252-265; funasr_provider.rs

## Open assumptions (announced defaults)
<!-- Record any default you adopt instead of asking, so the user can veto it at the gate. -->
- Model directory is fixed: `<app_data>/models/funasr/paraformer-large/` (engine requires it, user has placed files there)
- Provider string is `"funasr"` (matches engine.rs case at lines 150, 252)
- No model selection needed — always `paraformer-large` (single model, unlike Whisper/Parakeet)
- No API key needed — fully local (differs from cloud providers like Deepgram/OpenAI)

## Findings (cited - path:lines)
- TranscriptSettings.tsx (worktree): No 'funasr' in provider union (line 13), no SelectItem (lines 124-130), no modelOptions entry (lines 53-60), no requiresApiKey exclusion (line 61)
- The onValueChange handler (line 112-118) calls fetchApiKey for non-local providers — 'funasr' must be excluded
- The model selector dropdown (line 133-150) shows for non-localWhisper/parakeet — 'funasr' must be excluded
- useEffect apiKey clearer (line 36-40) clears for localWhisper/parakeet only — 'funasr' must be added
- engine.rs validates model at `<app_data>/models/funasr/paraformer-large/model.onnx` (line 160)
- engine.rs creates FunasrProvider::new(&model_dir) (line 262) — works with already-placed model files
- Full auto-download design already documented at `.omo/plans/funasr-autodownload.md` (10 todos, 374 lines, covers Rust commands + frontend UI + DirectML)

## Decisions (with rationale)
- MVP is a single-file change: TranscriptSettings.tsx only — no Rust changes needed (engine is complete and builds)
- Provider icon: use emoji that fits local provider style — `parakeet` uses ⚡, `localWhisper` uses 🏠. Recommend `🔊` or `🎯` for FunASR — the existing plan uses 🎯
- Design doc already exists! `.omo/plans/funasr-autodownload.md` covers the full scope. Just reference it.
- The existing 10-todo plan has Rust commands.rs, mod.rs, lib.rs registration, funasr.ts, FunasrModelManager.tsx, TranscriptSettings (which we're doing now), Cargo.toml DML, parakeet DirectML, funasr DirectML, verification. All still valid for future.

## Scope IN
- TranscriptSettings.tsx: +'funasr' in provider union, modelOptions, SelectItem, exclude from apiKey/model-select conditions
- Design doc: reference existing auto-download plan, add brief "next steps" summary

## Scope OUT (Must NOT have)
- No commands.rs creation (download not needed for MVP with manually placed models)
- No funasr.ts or FunasrModelManager.tsx (part of next bunch)
- No DirectML or Cargo.toml changes
- No changes to any Rust file

## Open questions
- None — all forks resolved by exploration

## Approval gate
status: awaiting-approval
pending-action: write .omo/plans/funasr-wiring-mvp.md
<!-- When exploration is exhausted and unknowns are answered, set status: awaiting-approval. -->
<!-- That durable record is the loop guard: on a later turn read it and resume at the gate instead of re-running exploration. -->
