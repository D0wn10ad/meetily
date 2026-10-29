# fork-review-fixes - Work Plan

## TL;DR (For humans)

**What you'll get:** Every one of the 151 problems this fork introduced is fixed across 33 files — nearly every fix guarded by a test that fails before the change and passes after, with both runs written down. The exceptions are stated, not hidden: T32 (no PowerShell runtime on this host), T3/T4 (build-script changes, whose red→green is recorded outside a `cargo test -p meetily` run), the comment/documentation-only todos (verified by a static diff check instead of a test), and T9/T15/T18 (which additionally touch satellite files outside their own). The 10 problems that actually belong to the upstream project are packaged separately as a ready-to-send pull request instead of being mixed into your branch.

**Why this approach:** The work happens in a separate checkout on its own branch, one small commit per touched file (33 commits in total: 32 on the fork branch, 1 on the upstream branch), so any single change can be reviewed or dropped on its own. Problems are grouped into batches that mostly touch different files, so they can be checked in parallel yet still land in a predictable order — the deliberate exceptions are the cross-file todos T9/T15/T18 and T33, which runs last on its own branch (see the dependency matrix). Problems inherited from upstream are deliberately kept out of your branch.

**What it will NOT do:** No dependency versions change. The archived backend folder is not touched. Code is not restructured, renamed or tidied beyond what each problem strictly requires. Nothing is published — both branches stay local for you to push yourself.

**Effort:** Large
**Risk:** Medium - the change surface spans the desktop core, the build scripts and the web UI, but each individual change is small, test-guarded and isolated on its own branch.
**Decisions to sanity-check:**
- The 10 upstream-inherited problems are deferred to a separate upstream pull request rather than fixed in your branch.
- The Windows and PowerShell surfaces can only be checked by reading the code here — no Windows runtime exists on this machine — so their real validation is deferred to the Windows CI job, and this is stated in the record rather than glossed over.
- Download cancellation gains an *optional* identifier, so a specific download can be cancelled while the screen that calls it keeps working unchanged.
- Three complaints turned out to be non-issues: one exported interface that is in fact used elsewhere, and build-script complaints about a format that is valid. These are recorded as not-reproducible rather than "fixed" by rewriting working code.

Your next move: approve the plan, or ask for a high-accuracy second opinion before a single line is written. Full execution detail follows below.

---

> TL;DR (machine): Large effort, Medium risk — 34 todos in 5 waves + F1–F4 on branch `fix/fork-review-fixes` (worktree `.worktrees/fork-review-fixes`); 151 fork-origin findings fixed with strict TDD, 10 upstream-origin findings deferred to `fix/upstream-review-deferrals` with a PR body; nothing pushed.

## Scope

**Reference-accuracy rule (applies to every todo and every check in this plan):**
Every `path:line` or `path:start-end` reference in this plan is an **ANCHOR** into the file **as of commit `5549e15`** (the execution worktree's base). Anchors are advisory; the **quoted construct is authoritative**. When an anchor no longer points at the quoted code, the executor MUST locate the construct **by its quoted text** (e.g. `grep -n 'get_onnx_providers' frontend/src-tauri/src/funasr_onnx/model.rs`) and treat the quoted content — not the line number — as the thing to change. Never refuse a todo because an anchor drifted, and never change a construct the todo does not quote. If a quoted construct cannot be found AT ALL, stop that todo and report it instead of guessing.

### Must have

**Attribution contract (already established, non-negotiable):**
The review scan produced 159 findings (plus 2 additional findings from the same scan that the earlier triage missed — see T34 — for **161 in total**). A provenance analysis (extract the fork's source lines `[start..end]`, strip all whitespace, test as a substring of the whitespace-stripped `git show a2cb62e:<path>`) split them into **151 fork-origin** (fix here) and **10 upstream-origin** (defer to an upstream PR). Files with diff status `A` (added wholesale by the fork) are 100% fork-origin. The 10 upstream findings are listed verbatim in Todo 33 and MUST NOT be edited on the fork branch.

**Deliverables:**
- All **151 fork-origin findings** fixed across **32 primary source files** (one PRIMARY file per todo, plus the explicitly enumerated satellite call-site edits that T9/T15/T18/T34's structural changes force in other files — every satellite edit is listed in the owning todo and must appear in the evidence file).
- **34 todos** total: T1 setup, T2–T34 source fixes (33 files), T33 upstream deferral PR.
- **Strict TDD** for every behavior-changing fix: write the failing test → confirm it fails *for the right reason* → implement → re-run → confirm green.
- Every todo leaves a machine-readable evidence file at `.omo/evidence/task-<N>-fork-review-fixes.txt`.
- Every **code-touching** todo ends in exactly one conventional commit: **32** on branch `fix/fork-review-fixes` (T2–T34, inside worktree `.worktrees/fork-review-fixes`) + **1** on the separate upstream branch (T33) — **33 commits in total**. T1 produces **no** commit (setup only).
- The 10 deferred upstream findings are packaged as a review-ready patch + PR description (Todo 33) on a **separate** branch/worktree based on `upstream/main`, never merged into the fork branch.
- A final parallel verification wave (F1–F4) that must unanimously approve before the work is declared complete.

**Per-file finding counts that must be fully addressed (151 total):**

| File | Findings |
| --- | --- |
| `.github/verify-windows-dlls.ps1` | 9 |
| `frontend/build-gpu.sh` | 3 |
| `frontend/eslint.config.mjs` | 4 |
| `frontend/package.json` | 3 |
| `frontend/src-tauri/Cargo.toml` | 7 |
| `frontend/src-tauri/build/onnxruntime.rs` | 2 |
| `frontend/src-tauri/build/sherpa.rs` | 7 |
| `frontend/src-tauri/src/audio/import.rs` | 5 |
| `frontend/src-tauri/src/audio/mod.rs` | 2 |
| `frontend/src-tauri/src/audio/onnx_provider.rs` | 7 |
| `frontend/src-tauri/src/audio/recording_preferences.rs` | 2 |
| `frontend/src-tauri/src/audio/retranscription.rs` | 9 |
| `frontend/src-tauri/src/audio/transcription/engine.rs` | 7 |
| `frontend/src-tauri/src/audio/transcription/funasr_provider.rs` | 4 |
| `frontend/src-tauri/src/audio/transcription/mod.rs` | 4 |
| `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs` | 5 |
| `frontend/src-tauri/src/database/manager.rs` | 4 |
| `frontend/src-tauri/src/funasr_onnx/commands.rs` | 9 |
| `frontend/src-tauri/src/funasr_onnx/decoder.rs` | 6 |
| `frontend/src-tauri/src/funasr_onnx/engine.rs` | 5 |
| `frontend/src-tauri/src/funasr_onnx/frontend.rs` | 5 |
| `frontend/src-tauri/src/funasr_onnx/mod.rs` | 6 |
| `frontend/src-tauri/src/funasr_onnx/model.rs` | 5 |
| `frontend/src-tauri/src/lib.rs` | 3 |
| `frontend/src-tauri/src/sherpa_onnx/commands.rs` | 6 |
| `frontend/src-tauri/src/sherpa_onnx/engine.rs` | 4 |
| `frontend/src-tauri/src/sherpa_onnx/mod.rs` | 3 |
| `frontend/src-tauri/src/whisper_engine/whisper_engine.rs` | 3 |
| `frontend/src/hooks/useTranscriptionModels.ts` | 4 |
| `frontend/src/lib/funasr.ts` | 4 |
| `llama-helper/Cargo.toml` | 2 |
| `frontend/src/components/FunasrModelManager.tsx` | 1 |
| `frontend/src/components/TranscriptSettings.tsx` | 1 |

`frontend/src-tauri/src/database/repositories/setting.rs` is **excluded** — both of its findings (194-204, 303-307) are upstream-origin and are deferred to Todo 33.

### Deliberate non-changes (in-scope files, lines deliberately left alone)

These are **in-scope files** where a specific line is intentionally NOT touched, so no worker "helpfully" widens the diff:

- **`frontend/src-tauri/src/funasr_onnx/engine.rs:54`** — `log::info!("FunASR transcription: '{}'", text);` echoes the full transcript into the log. This line is **not** one of T10's five findings, so T10 must leave it as-is. The *provider-level* equivalent (`frontend/src-tauri/src/audio/transcription/funasr_provider.rs`) **is** one of T23's four findings, so T23 redacts it there. Result: the provider-level log is redacted while the engine-level log stays — a deliberate, documented consistency extension in T23's evidence file, not an oversight.
- **`frontend/src-tauri/src/funasr_onnx/engine.rs:62-63`** — `current_model()` **already exists** on `FunasrEngine`; do not add a second one (T10 only removes the dead `MODELS_DIR` state and fixes the error handling).

### Known pre-existing issues (not in scope)

These failures/broken tools exist **before** this work starts. They are environment or upstream conditions, **not** product defects, and must never be counted as regressions nor "fixed" by a todo:

- `test_calculate_buffer_timeout_bluetooth` — a float-precision assertion failure that already fails on the baseline.
- ONNX-runtime `munmap_chunk` memory corruption observed in the **full** Rust suite run (the per-module targeted runs used by the todos are unaffected).
- `next lint` is incompatible with the repo's ESLint v9 — the lint command is `eslint .`, not `next lint`.
- `pwsh` / `powershell` are **not installed** on this host ⇒ every Windows / PowerShell surface (T3, T4, T32) is verified **statically only**; runtime validation is deferred to the Windows CI job.
- `shellcheck` is **not installed** ⇒ shell-script todos use `bash -n` only.
- The Linux `libappindicator3-dev` pkg-config entry may be missing on this host ⇒ a Linux desktop-build failure is an environment gap, not a product defect.

### Must NOT have (guardrails, anti-slop, scope boundaries)

- **No edits to upstream-origin logic.** The 10 deferred findings (listed in Todo 33) must not be touched on `fix/fork-review-fixes`. In particular, do not touch `frontend/src-tauri/src/database/repositories/setting.rs` at all.
- **No edits under `backend/`** — archived Python/FastAPI legacy, per `AGENTS.md`.
- **No dependency version bumps.** Keep `ort = "2.0.0-rc.10"` pinned (do not bump the RC), keep `llama-cpp-2 = "=0.1.146"` / `llama-cpp-sys-2 = "=0.1.146"` pinned (document-only if anything), keep the 5 `pnpm.overrides` prosemirror pins (model 1.25.3, state 1.4.3, view 1.41.3, transform 1.10.4, tables 1.8.1) — they are an accepted risk called out in `AGENTS.md`; add a comment only.
- **Do not remove the Windows `ort` `load-dynamic` feature** and do not re-enable ort default features (`copy-dylibs` conflicts with the sha-verified single `onnxruntime.dll`).
- **Do not remove the Windows shared/static `sherpa-onnx` split** (Windows `default-features=false, features=["shared"]` vs non-Windows static) — it is a legitimate /MD CRT conflict workaround (LNK2038 / LNK2005 / LNK1169).
- **Do not delete the DirectML execution provider** — gate it by `target_os` instead.
- **No new runtime dependencies** in any manifest. (T7 adds `eslint` as a **devDependency** — that is a build-time tool the `lint` script already assumes, not a runtime dependency.)
- **No work in the main checkout.** Every command runs with `workdir=/home/tango/repo/meetily/.worktrees/fork-review-fixes`. The main checkout at `/home/tango/repo/meetily` must remain byte-identical except for `.omo/` plan artifacts.
- **No git history surgery** — no `push`, no `rebase`, no `amend`, no `force-push`. Linear conventional commits on `fix/fork-review-fixes` only.
- **Do not touch the 1355 upstream-owned findings** from the original scan that were never in scope.
- **No behavior change beyond the finding fixes.** No refactors, renames, or drive-by cleanups.
- **Do not "fix" verified-false findings.** Specifically: `funasr_onnx/mod.rs` `AudioTooShort` already uses named fields (only the field names `min`/`actual` → `expected`/`got` + a doc comment are in scope); `audio/onnx_provider.rs` "`.build()` returns `Result`" is **false** for ort 2.0.0-rc.10 (do **not** add `.expect()` — document the pinned-version contract instead); `frontend/build-gpu.sh` comma-separated `--features` is valid (the finding is about the brittle `${ORT_FEATURES#--features }` *parser*, not the format); `audio/retranscription.rs` "mutex held across await" is false (the guard is dropped at line 658) — the in-scope defect is the fragile `unwrap_or_else(|e| e.into_inner())` poison recovery and the `OnceLock` TOCTOU.

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- **Test decision: TDD** (strict) + Rust `#[cfg(test)]` unit/integration tests via `cargo test -p meetily` and TypeScript `bun:test` via `bun test` from `frontend/`.
- **TDD loop, mandatory order inside every todo:** (1) write the failing test, (2) run it and confirm it fails **for the right reason** (the assertion must reference the defect, not a compile error or missing symbol), (3) implement the minimal fix, (4) re-run the test and confirm it passes, (5) run the broader suite to confirm no regression. A todo that ships without a recorded red→green transition is incomplete.
- **Per-todo evidence:** `.omo/evidence/task-<N>-fork-review-fixes.txt` (append-only, in the worktree's `.omo/evidence/`), containing: the exact commands run, the failing-test output (reason captured), the passing-test output, and the finding IDs addressed. T1 seeds the directory; the evidence directory is copied into the worktree by T1.
- **Baseline before change (T1):** run `cargo test -p meetily` and `bun test` and record the **pre-existing** failures so post-change comparisons are meaningful. Known pre-existing failures (see "Known pre-existing issues" below) must not be counted as regressions.
- **Full-suite re-verification:** each Rust todo re-runs `cargo test -p meetily`; each TS todo re-runs `bun test` from `frontend/`.
- **Static checks:** `cargo check -p meetily` after every Rust todo; `bash -n` (not shellcheck — not installed) after every shell-script todo; `eslint .` (not `next lint` — ESLint v9 incompatible) after every JS/TS todo. **Clippy and ESLint are NOT blocking for any todo, but their baselines ARE captured and ARE the reference for F2.2 and F2.5:** the clippy baseline is captured in T1 step 5 (`.omo/evidence/t1-clippy-baseline.txt`) and the ESLint baseline is captured in T7 (`.omo/evidence/t7-eslint-baseline.txt` — the `eslint` binary does not exist until T7 installs it). F2.2 and F2.5 diff against those two files; a new warning/error fails the **final wave**, not an individual todo.
- **Windows-only surfaces** (`.github/verify-windows-dlls.ps1`, `build/sherpa.rs` Windows target cfg, `build/onnxruntime.rs` Windows paths): `pwsh` is **not installed** on this host, so verification is static (parser-level review + `cargo check --target x86_64-pc-windows-msvc` is NOT attempted) and the actual runtime validation is deferred to the Windows CI job. The todo must state this explicitly in its evidence file.
- **Provenance re-check (T33):** at the final commit, re-run the whitespace-stripped substring test for all 161 findings to prove no upstream-origin code was modified on the fork branch.

## Execution strategy
### Parallel execution waves
> Target 5-8 todos per wave. Fewer than 3 (except the final) means you under-split.

- **Wave 0 — Setup (1 todo, 0 findings):** T1 worktree bootstrap, `.omo/` artifact copy, baseline test capture.
- **Wave 1 — Manifests, build scripts & CI config (7 todos, 28 findings):** T2 `frontend/src-tauri/Cargo.toml` (7) · T3 `frontend/src-tauri/build/sherpa.rs` (7) · T4 `frontend/src-tauri/build/onnxruntime.rs` (2) · T5 `frontend/build-gpu.sh` (3) · T6 `llama-helper/Cargo.toml` (2) · T7 `frontend/package.json` (3) · T8 `frontend/eslint.config.mjs` (4).
- **Wave 2 — FunASR module + ONNX EP (7 todos, 43 findings):** T9 `funasr_onnx/mod.rs` (6) · T10 `funasr_onnx/engine.rs` (5) · T11 `funasr_onnx/model.rs` (5) · T12 `funasr_onnx/decoder.rs` (6) · T13 `funasr_onnx/frontend.rs` (5) · T14 `funasr_onnx/commands.rs` (9) · T21 `audio/onnx_provider.rs` (7). **Wave 2 internal order:** T21 commits FIRST (it changes `get_onnx_providers`, which T11 calls at `model.rs:39` and `model.rs:180`), then T9 → T10 → T11, then T12/T13/T14 in parallel.
- **Wave 3 — Sherpa & transcription core (7 todos, 29 findings):** T15 `sherpa_onnx/mod.rs` (3) · T16 `sherpa_onnx/engine.rs` (4) · T17 `sherpa_onnx/commands.rs` (6) · T18 `audio/transcription/mod.rs` (4) · T19 `audio/transcription/engine.rs` (7) · T20 `whisper_engine/whisper_engine.rs` (3) · T22 `audio/mod.rs` (2). (T21 was moved OUT of Wave 3 into Wave 2 — T11 depends on it; see the dependency matrix.)
- **Wave 4 — Providers & app-level (7 todos, 32 findings):** T23 `audio/transcription/funasr_provider.rs` (4) · T24 `audio/transcription/sherpa_onnx_provider.rs` (5) · T25 `audio/import.rs` (5) · T26 `audio/retranscription.rs` (9) · T27 `audio/recording_preferences.rs` (2) · T28 `database/manager.rs` (4) · T29 `src-tauri/src/lib.rs` (3).
- **Wave 5 — Frontend TS/UI, PowerShell CI & upstream PR (5 todos, 29 findings):** T30 `frontend/src/lib/funasr.ts` (4) · T31 `frontend/src/hooks/useTranscriptionModels.ts` (4) · T32 `.github/verify-windows-dlls.ps1` (9) · T34 `frontend/src/components/FunasrModelManager.tsx` (2) · T33 upstream deferral PR (10 deferred findings).
- **Final wave — F1–F4** run in parallel after T1–T33 all land.

**Findings math:** 28 + 43 + 29 + 32 + 29 = 161 = 151 in-fork + 10 deferred. **Todo math:** 1 + 7 + 7 + 7 + 7 + 5 = 34. (Wave 2 = 43 findings because T21's 7 findings moved into it; Wave 3 = 29 for the same reason.)

**Wave sizing note:** Waves 1–5 hold 7/7/7/7/4 todos. Wave 5's smaller size is intentional and justified: T30/T31 are TypeScript and touch a different toolchain from the Rust waves, T32 is a Windows-only PowerShell script that no Rust todo can validate, and T33 must run **last** so it can re-run the provenance check against the final commit. The ulw-plan "fewer than 3 means you under-split" rule is satisfied in every wave.

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| T1 Bootstrap worktree + copy `.omo/` + baseline | — | T2–T33 | — (runs alone first) |
| T2 `frontend/src-tauri/Cargo.toml` | T1 | T3, T21 | T4, T5, T6, T7, T8 |
| T3 `frontend/src-tauri/build/sherpa.rs` | T1, T2 | — | T4, T5, T6, T7, T8, T21 |
| T4 `frontend/src-tauri/build/onnxruntime.rs` | T1 | — | T2, T3, T5, T6, T7, T8 |
| T5 `frontend/build-gpu.sh` | T1 | — | T2, T3, T4, T6, T7, T8 |
| T6 `llama-helper/Cargo.toml` | T1 | — | T2, T3, T4, T5, T7, T8 |
| T7 `frontend/package.json` | T1 | T8 | T2, T3, T4, T5, T6 |
| T8 `frontend/eslint.config.mjs` | T1, T7 | — | T2, T3, T4, T5, T6 |
| T9 `funasr_onnx/mod.rs` | T1 | T10, T11, T12, T13, T14, T23 | T15, T16, T17, T18, T19, T20 |
| T10 `funasr_onnx/engine.rs` | T1, T9 | T11, T14, T29 | T15, T16, T17, T18, T19, T20 |
| T11 `funasr_onnx/model.rs` | T1, T9, T10, T21 | — | T12, T13, T14, T15–T20 |
| T12 `funasr_onnx/decoder.rs` | T1, T9 | — | T10, T11, T13, T14, T15–T20 |
| T13 `funasr_onnx/frontend.rs` | T1, T9 | — | T10, T11, T12, T14, T15–T20 |
| T14 `funasr_onnx/commands.rs` | T1, T9, T10 | T30 | T11, T12, T13, T15–T20 |
| T15 `sherpa_onnx/mod.rs` | T1 | T16, T17 | T9–T14, T18–T22 |
| T16 `sherpa_onnx/engine.rs` | T1, T15 | T19, T24 | T9–T14, T17–T22 |
| T17 `sherpa_onnx/commands.rs` | T1, T15 | T29 | T9–T14, T16, T18–T22 |
| T18 `audio/transcription/mod.rs` | T1 | T19, T23, T24, T25, T26 | T9–T17, T20–T22 |
| T19 `audio/transcription/engine.rs` | T1, T16, T18 | — | T20, T21, T22, T23–T29 |
| T20 `whisper_engine/whisper_engine.rs` | T1 | — | T9–T19, T21–T29 |
| T21 `audio/onnx_provider.rs` | T1, T2 | T11 | — (lands FIRST in Wave 2: no same-wave parallel partner — it is file-independent of T3–T10, T12–T20 and T22–T29, but all of those land in later waves) |
| T22 `audio/mod.rs` | T1 | — | T9–T21, T23–T29 |
| T23 `audio/transcription/funasr_provider.rs` | T1, T9, T18 | — | T19–T22, T24–T29 |
| T24 `audio/transcription/sherpa_onnx_provider.rs` | T1, T16, T18 | — | T19–T23, T25–T29 |
| T25 `audio/import.rs` | T1, T18 | — | T19–T24, T26–T29 |
| T26 `audio/retranscription.rs` | T1, T18 | — | T19–T25, T27–T29 |
| T27 `audio/recording_preferences.rs` | T1 | — | T19–T26, T28–T29 |
| T28 `database/manager.rs` | T1 | — | T19–T27, T29 |
| T29 `src-tauri/src/lib.rs` | T1, T10, T17 | — | T19–T28, T30–T33 |
| T30 `frontend/src/lib/funasr.ts` | T1, T14 | T34 | T19–T29, T31–T33 |
| T31 `frontend/src/hooks/useTranscriptionModels.ts` | T1 | — | T19–T30, T32, T33 |
| T32 `.github/verify-windows-dlls.ps1` | T1 | — | T19–T31, T34, T33 |
| T34 `frontend/src/components/FunasrModelManager.tsx` | T1, T30 | — | T31, T32, T33 |
| T33 Upstream deferral PR (10 findings) | T1, T29 | — | — (runs last, alone) |

**Matrix column semantics:** `Depends on` is the authoritative blocker list — F1.5 checks that every todo's commit position is after every todo in this column, and the per-todo `Blocked by:` / `Blocks:` lines below are the same graph stated per todo (they must agree with this table). **This matrix is the single source of truth for the `Blocks` column** — where a todo's `Blocks:` line disagrees with this table, the table wins. `Blocks` mirrors `Depends on` in reverse; the one `Depends on` edge that is not mirrored in the `Blocks` column is T33's dependency on T29, because T33 is committed on the separate upstream branch and therefore blocks no fork-branch commit. `Can parallelize with` lists todos that are **file-independent** of this one (different source file, so two workers never touch the same file); where the plan states a stricter sequence — T21 → T9 → T10 → T11 → T12/T13/T14 in parallel — the stated sequence wins over the file-independence hint.

## Todos
> Implementation + Test = ONE todo. Never separate.
- [x] 1. Bootstrap the execution worktree, mirror `.omo/` artifacts, capture the baseline test state
  What to do / Must NOT do:
  1. The worktree **already exists** at `/home/tango/repo/meetily/.worktrees/fork-review-fixes` on branch `fix/fork-review-fixes` @ `5549e15`. **Do NOT create another one.** Verify it: `git -C /home/tango/repo/meetily/.worktrees/fork-review-fixes status --porcelain` must output **exactly one line, `?? .omo/`** (the plan-artifact directory is untracked and NOT gitignored — it is already present in the worktree) and nothing else, and `git -C ... rev-parse --abbrev-ref HEAD` must print `fix/fork-review-fixes`. Every command in this todo runs with **workdir = `/home/tango/repo/meetily/.worktrees/fork-review-fixes`** (never the main checkout) and uses absolute or worktree-relative paths only.
  2. `.omo/` is **not** gitignored, so the plan artifacts must be copied into the worktree. Copy **from the main checkout** (absolute source paths) **into the worktree** (worktree-relative destination), with workdir = the worktree: `mkdir -p .omo/evidence` then `cp -r /home/tango/repo/meetily/.omo/plans /home/tango/repo/meetily/.omo/drafts /home/tango/repo/meetily/.omo/evidence .omo/`. This copy makes `.omo/` show up as the single untracked entry `?? .omo/` in `git status --porcelain` (never as tracked files). **This step must run BEFORE the acceptance criteria below are evaluated.** The expected post-copy file list is exactly: `.omo/plans/fork-review-fixes.md` (this plan) and `.omo/evidence/fork-review-inscope.txt` (the in-scope evidence file) — confirm both with `test -f`.
  3. **Install the frontend dependencies FIRST** — neither the main checkout nor the worktree has `frontend/node_modules`, so `bun test` and `eslint .` cannot run before this: `cd /home/tango/repo/meetily/.worktrees/fork-review-fixes/frontend && pnpm install`. If `pnpm install` fails, **stop and report** — do not continue with a missing toolchain. **Every command in this todo runs with workdir = `/home/tango/repo/meetily/.worktrees/fork-review-fixes`** (never the main checkout).
  4. Capture the **test baseline** so later todos can distinguish regressions from pre-existing failures. Use **absolute** paths for every output file and run both commands with workdir = the worktree. Run `cargo test -p meetily 2>&1 | tee /home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/task-1-fork-review-fixes.txt`, then `bun test 2>&1 | tee -a /home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/task-1-fork-review-fixes.txt` with workdir = `/home/tango/repo/meetily/.worktrees/fork-review-fixes/frontend` — **`bun test` only runs after step 3's `pnpm install` has succeeded**, otherwise it reports spurious module-resolution failures (see item 6d).
  5. Capture the **clippy baseline** (the reference for F2.2): `cargo clippy -p meetily --all-targets 2>&1 | tee /home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/t1-clippy-baseline.txt`, and append the count to the main evidence file as `BASELINE_CLIPPY_WARNINGS=<n>`. The **ESLint** baseline is NOT captured here — the `eslint` binary does not exist until T7 installs it, so **T7** captures it as `.omo/evidence/t7-eslint-baseline.txt` (the reference for F2.5) using the **local** `frontend/node_modules/.bin/eslint` at the version T7 pins (a global ESLint such as v6.4.0 rejects this repo's flat config and exits 2, which would produce a bogus baseline).
  6. Enumerate the **known pre-existing failures** explicitly in the evidence file: (a) `test_calculate_buffer_timeout_bluetooth` float-precision failure; (b) ONNX-runtime global-state heap corruption `munmap_chunk` in the full Rust suite; (c) `next lint` vs ESLint v9 incompatibility (use `eslint .` directly, never `next lint`); (d) `bun test` reports 4 module-not-found errors for `react/jsx-dev-runtime` — a bun/JSX-runtime resolution gap that appears whenever `frontend/node_modules` is absent or incomplete, which is why step 3's `pnpm install` must succeed before the step-4 `bun test` baseline; if the 4 errors are still present after a successful install, record them verbatim as a known pre-existing failure.
  7. Record the toolchain versions in the evidence file: `cargo --version` (1.96.0), `bun --version` (1.3.14), `node --version` (v24.18.0), `pnpm --version`. Record that `pwsh` and `shellcheck` are **absent** — this is why T32 and the shell todos are statically verified.

  **Must NOT do:** do not edit any product code in this todo; do not run `git checkout`/`git reset`/`git stash`; do not switch or create a branch; do not modify the main checkout's product files; do not `cargo update` (that would change dependency versions and is forbidden).
  Parallelization: Wave 0 | Blocked by: — | Blocks: T2–T33
  References (executor has NO interview context - be exhaustive):
  - `/home/tango/repo/meetily/.worktrees/fork-review-fixes` — the pre-created execution worktree (branch `fix/fork-review-fixes`, HEAD `5549e15`).
  - `/home/tango/repo/meetily/.gitignore:81` — `.worktrees/` is ignored; `.omo/` is **not** ignored (hence the copy in step 2).
  - `/home/tango/repo/meetily/.omo/plans/fork-review-fixes.md` — this plan (source of truth for all 34 todos).
  - `/home/tango/repo/meetily/.omo/drafts/fork-review-fixes.md` — the draft record.
  - `/home/tango/repo/meetily/.omo/evidence/fork-review-inscope.txt` — the authoritative 225-line finding list (159 findings) this plan is derived from; T34's 2 additional findings are recorded in `.omo/evidence/w2-core-scan_v3.json` (`FunasrModelManager.tsx:129-146`, `TranscriptSettings.tsx:102-112`) and are the only findings not present in this file.
  - `/home/tango/repo/meetily/AGENTS.md` — project rules (pnpm not npm, `cargo test -p meetily`, `bun test` from `frontend/`, backend/ is archived, version source of truth is `tauri.conf.json`).
  Acceptance criteria (agent-executable):
  - `git -C /home/tango/repo/meetily/.worktrees/fork-review-fixes status --porcelain` → exactly one line, `?? .omo/` (the untracked plan-artifact directory), and nothing else.
  - `git -C /home/tango/repo/meetily/.worktrees/fork-review-fixes rev-parse --abbrev-ref HEAD` → `fix/fork-review-fixes`.
  - After step 2's copy, `test -f /home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/plans/fork-review-fixes.md` **and** `test -f /home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/fork-review-inscope.txt` → both exit 0 (the expected post-copy file list is exactly these two files).
  - `/home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/task-1-fork-review-fixes.txt` exists, is > 100 lines, and contains the strings `pnpm install`, `cargo test -p meetily`, `bun test`, `cargo clippy --all-targets -p meetily`, `BASELINE_CLIPPY_WARNINGS=`, `test_calculate_buffer_timeout_bluetooth`, `munmap_chunk`, `pwsh`, `shellcheck`.
  - `/home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/t1-clippy-baseline.txt` exists and is non-empty (F2.2 compares against it). The ESLint baseline is `/home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/t7-eslint-baseline.txt`, produced by T7 (F2.5 compares against it).
  - `git -C /home/tango/repo/meetily diff --stat` shows **no** product-file changes (only `.omo/` artifacts in the main checkout, which is expected and allowed).
  QA scenarios (name the exact tool + invocation):
  - Happy: baseline captured → `grep -c 'test result' .worktrees/fork-review-fixes/.omo/evidence/task-1-fork-review-fixes.txt` returns a non-zero count, and the 4 known failures are listed verbatim.
  - Failure: if `cargo test -p meetily` fails to **compile** at baseline, or if the worktree is dirty, **stop and report** — every downstream todo depends on a compiling, clean baseline. Record the exact compiler output in the evidence file. Do not attempt to fix it here.
  Evidence: `.omo/evidence/task-1-fork-review-fixes.txt`
  Commit: N | (no code change — setup only)

- [x] 2. `frontend/src-tauri/Cargo.toml` — wire the `ort-*` execution-provider features, unify the duplicate `ort` declaration, and document the pinned RC (7 findings)
  What to do / Must NOT do:
  1. **Feature wiring (verified — only `ort-cpu` is empty, and it must stay empty).** `Cargo.toml:56-60` already wires `ort-cuda = ["ort/cuda"]`, `ort-directml = ["ort/directml"]`, `ort-coreml = ["ort/coreml"]` and `ort-openvino = ["ort/openvino"]`. `ort` 2.0.0-rc.10 exposes no `cpu` feature (the CPU EP is ort's default provider), so `ort-cpu` is an intentional empty marker feature consumed by `frontend/build-gpu.sh:118,123` — do **not** change it to `["ort/cpu"]` (cargo errors: "feature `ort/cpu` does not exist"). In-scope action: add a one-line comment at `Cargo.toml:56` documenting that `ort-cpu` is an intentional empty marker and that the four EP features are already wired; change no feature mapping.
  2. **Duplicate `ort` declaration (low, lines 115-118).** `ort` is declared at global line 115 and again in the Windows target block at line 200 with `load-dynamic`. Cargo unions features, so the two declarations are merged. Consolidate to a single declaration with the union of the required features, keeping the Windows-only `load-dynamic` behaviour (see Must NOT do). **Do NOT collapse the Windows-target `ort` into the global entry** — the Windows `load-dynamic` feature must stay on its own target-specific declaration (it is in the Must-NOT-do list and is checked by F4.3/F4.4); make the global entry non-conflicting with it instead.
  3. **`platform-default = []` (medium, lines 38-42).** It is an empty feature that ships in `default`. Remove the platform-default feature: it is referenced only at `frontend/src-tauri/Cargo.toml:39` and `:42` — delete both lines (the default feature list becomes `["ort-cpu"]`). Verified by grep: `platform-default` occurs exactly twice repo-wide, both in `frontend/src-tauri/Cargo.toml` (lines 39 and 42); no script references it. (The CI workflows under the repo-root `.github/workflows/` are outside this plan's `frontend/` scope and do not reference it.)
  4. **RC pin documentation (medium, lines 115-118).** Keep `ort` pinned to the pre-release `2.0.0-rc.10`. Add a comment above the pin stating: the pre-release is intentional and pinned because the EP `.build()` API in the RC returns `ExecutionProviderDispatch` directly (not a `Result`); record the upstream crate version + date pinned.
  5. **Version drift notes (low ×2).** For `serde_json = "1.0"`, `serde_yaml = "0.9"`, `clap = "4.3"` — do **not** pin to `=x.y.z` (that would be a dependency change and is forbidden). Instead, add a comment noting these are caret-ranged and why exact pinning is not used. For the "outdated versions" finding (`uuid 1.0`, `log 0.4`, `env_logger 0.11`, `crossbeam 0.8.4`, `dashmap 6.1.0`, `dirs 5.0.1`, `regex 1.11.0`): run `cargo outdated` **only if the `cargo-outdated` plugin is installed** (`cargo outdated --version` succeeds) and write its output into the evidence file; if it is not installed, write `cargo-outdated not installed — outdated-version check skipped` into the evidence file and do **not** install it. **Do not upgrade anything.** (The `llama-cpp-2` / `llama-cpp-sys-2` `=0.1.146` pins live in the workspace-root `llama-helper/Cargo.toml` at lines 10 and 14 — **not** in `frontend/src-tauri/Cargo.toml`; T2 does not touch them, T6 owns that file.)
  6. **sherpa-onnx Windows split (low, lines 200-208).** Document (comment only) that the Windows `default-features = false, features = ["shared"]` vs the non-Windows default (static) split is an intentional /MD CRT conflict workaround (LNK2038 / LNK2005 / LNK1169), and state what to check if the split is ever changed. Verify the current default feature set is intentional by inspecting the sherpa-onnx crate's `Cargo.toml` in the cargo registry.

  7. **macOS target-section redundancy (hardening item, beyond the counted findings — low/maintainability, `Cargo.toml:176`, `:177`, `:180`, `:182`).** The `[target.'cfg(target_os="macos")'.dependencies]` block re-declares four crates the top level already declares: `tauri` (L176 — same `2.6.2` and the same `macos-private-api`/`protocol-asset`/`tray-icon` set as L162, only reordered), `once_cell` (L177 vs L77, both `1.17.1`), `anyhow` (L180 vs L76), and `reqwest` (L182 vs L64 — the macOS entry is a `multipart,json` subset of the top-level `blocking,multipart,json,stream`). Delete exactly those four lines so each crate is declared once. Keep `objc`, `time`, `core-graphics`, `cidre`, `dasp` and `futures-channel` in the target block — those are genuinely macOS-only. Cargo unions target-specific features with the global entry, so the effective macOS feature set must be unchanged; prove it agent-executably with `sed -n '/cfg(target_os="macos")/,/^\[target/p' frontend/src-tauri/Cargo.toml | grep -cE '^(tauri|once_cell|anyhow|reqwest) ='` returning `0`, and record in the evidence file that the retained global `tauri` entry still carries all three features (`grep -n 'macos-private-api' frontend/src-tauri/Cargo.toml` shows the L162 entry untouched).

  **Must NOT do:** do **not** bump `ort` off `2.0.0-rc.10`; do **not** remove the Windows `load-dynamic` feature; do **not** re-enable ort's default features (`copy-dylibs` conflicts with the sha-verified single `onnxruntime.dll`); do **not** remove the Windows shared/static `sherpa-onnx` split; do **not** change any dependency version number; do **not** run `cargo update`; do **not** add a new dependency.
  Parallelization: Wave 1 | Blocked by: T1 | Blocks: T3, T21
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/Cargo.toml:38-42` — `platform-default = []` empty feature inside `default`.
  - `frontend/src-tauri/Cargo.toml:56-60` — the five `ort-cpu|ort-cuda|ort-directml|ort-coreml|ort-openvino` features: four are already wired to the matching `ort/*` features; only `ort-cpu` is an intentional empty marker.
  - `frontend/src-tauri/Cargo.toml:115-118` — global `ort` dependency declaration (the `ort = { version = "2.0.0-rc.10", default-features = false, features = [ … ] }` entry; line 95 is `env_logger`, not `ort`).
  - `frontend/src-tauri/Cargo.toml:114-118` — the `ort` version pin (`2.0.0-rc.10`).
  - `frontend/src-tauri/Cargo.toml:200` — Windows target-specific `ort` declaration carrying `load-dynamic` (the Windows target block opens at line 198; line 149 is `symphonia`, not `ort`).
  - `frontend/src-tauri/Cargo.toml:200-208` — `sherpa-onnx` Windows vs non-Windows feature split.
  - `/home/tango/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ort-2.0.0-rc.10` — the pinned ort source; `execution_providers/mod.rs:331` returns the dispatch directly (`.build()` is **not** a `Result`).
  - `frontend/src-tauri/build/sherpa.rs` — the build script that reads the sherpa-onnx target/feature configuration (T3 depends on this file's expectations).
  - `AGENTS.md` — Rust feature list (`metal`, `coreml`, `cuda`, `vulkan`, `hipblas`, `openblas`, `openmp`); macOS defaults to Metal+CoreML, Windows/Linux default to CPU.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - `cargo check -p meetily` exits 0; `grep -n 'ort-cpu' frontend/src-tauri/Cargo.toml` shows exactly one line, still `ort-cpu = []`; `cargo tree -e features -p meetily --features ort-cuda` shows ort feature "cuda"; the four EP feature lines at Cargo.toml:57-60 are unchanged except for the added comment.
  - `grep -n '^ort = ' frontend/src-tauri/Cargo.toml` returns exactly **2** hits — the global entry at line 115 and the Windows-target entry at line 200 (the pattern must be `^ort = `, not `^ort`, because `^ort` also matches the `ort-cpu`/`ort-cuda`/`ort-dml`/`ort-directml`/`ort-coreml`/`ort-openvino` feature flags at lines 54-60). The Windows-target `ort` (with `load-dynamic`) must still be present.
  - `grep -rn 'platform-default' frontend/ --include=*.toml --include=*.js --include=*.yml --include=*.yaml --include=*.sh` returns **no** hits after removal (before removal it returns exactly 2 hits, both in `frontend/src-tauri/Cargo.toml` at lines 39 and 42).
  - `git diff frontend/src-tauri/Cargo.toml` contains no version-number changes other than added comments (verify: the diff shows no `-` line containing a version string).
  - `cargo outdated` output is captured in the evidence file **if `cargo-outdated` is installed**; otherwise the evidence file records `cargo-outdated not installed — check skipped` (a non-blocking, optional check — its absence is never a failure).
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo check -p meetily` exits 0 and `cargo tree -e features -p meetily --features ort-cuda` shows `ort` with the `cuda` feature enabled. This already holds at baseline, so it is a **regression guard**, not a red→green: the red→green for this finding is the added `Cargo.toml:56` comment, verified by `git diff frontend/src-tauri/Cargo.toml` showing that comment line and no version-string change.
  - Failure: if wiring the EP features pulls in a **new** transitive dependency or changes the resolved `ort` version, **revert the wiring** and fall back to documenting the no-op in a comment; record the resolution in the evidence file. Also verify the Windows build path is untouched: `git diff` shows the `load-dynamic` feature still present.
  Evidence: `.omo/evidence/task-2-fork-review-fixes.txt`
  Commit: Y | `build(cargo): wire ort execution-provider features and document pinned RC`

- [x] 3. `frontend/src-tauri/build/sherpa.rs` — make the prebuilt-DLL freshness check content-aware and stop swallowing build-script errors (7 findings)
  What to do / Must NOT do:
  1. **PROFILE read (medium/bug, lines 46-50).** `std::env::var("PROFILE").unwrap_or_default()` (line 46) yields `""` when the env var is missing; the subsequent `find(|p| p.file_name() == Some(OsStr::new(&profile)))` chain (lines 47-49) then matches nothing and returns `None`, so line 50's `.expect("could not resolve cargo profile dir from OUT_DIR")` **panics** — there is no silent wrong-directory path, the failure is a panic with a vague message. Replace the `unwrap_or_default()` with an `.expect("PROFILE env var not set by cargo; cannot resolve target profile dir")` that names PROFILE, and make line 50's message mention both `PROFILE` and `OUT_DIR`.
  2. **Ancestor search (high/bug, lines 55-63).** The `unwrap_or_else` fallback only tries `profile_dir.parent()`, which resolves to `<target>` in both cargo layouts (with `--target`: `<target>/<triple>/<profile>` → parent `<target>/<triple>`; without: `<target>/<profile>` → parent `<target>`) — it never resolves to `out`/`debug`, but it also does not walk far enough to find `<target>/sherpa-onnx-prebuilt/` in the `--target` layout. Walk **all** ancestors of the profile dir looking for a `sherpa-onnx-prebuilt` directory, and if none is found, panic with a clear message that names every ancestor that was searched.
  3. **Freshness check (high/bug, lines 73-80).** The current check compares only `src_meta.len() == dst_meta.len()`, so a same-size-but-different-content DLL is silently skipped and a **stale DLL gets shipped**. Compare modification times (`mtime`) and, better, a content hash. Implement a size + mtime comparison, and if a cheap hash is available for the file sizes involved, use it. The test must prove that a same-size/different-content file is detected as stale. **Testability:** move the pure freshness predicate into the new dependency-free lib file `frontend/src-tauri/src/build_env.rs` (see the shared testability note below) so it can be unit-tested without a build-script harness.
  4. **ReadDir errors (medium/maintainability, line 20).** `.ok()?.flatten()` swallows every `ReadDir` error. Propagate the error, or emit `cargo:warning=` and continue. Emitting a warning is acceptable for the "directory missing" case but not for a permission/IO failure.
  5. **Panic message accuracy (low ×2, lines 104-108 and 115-121).** "expected it in Cargo.toml target cfg for windows" is misleading: `shared` is a **feature**, not a target cfg. Correct both messages to say the `sherpa-onnx` `shared` feature must be enabled (or the corresponding target section present) and name the actual knob.
  6. **Actionable guidance (low, lines 104-108).** Panic messages must state what the user can do (e.g. "set the `SHERPA_ONNX_PREBUILT` env var to the directory containing the prebuilt DLLs" or "run the sherpa-onnx prebuilt download step").

  **Shared testability note (applies to T3 and T4):** build scripts cannot host `#[cfg(test)]` tests that `cargo test -p meetily` runs, so the testable pure logic lives in a NEW dependency-free lib file `frontend/src-tauri/src/build_env.rs`. The lib crate declares `pub mod build_env;` (with its own `#[cfg(test)] mod tests` inside that file), and each build script pulls the same source in with `include!("../src/build_env.rs");`. T3 puts the cargo profile-dir resolution helper there; T4 puts the download retry/backoff schedule there. The file must use only `std` (no new dependency).

  **Must NOT do:** do **not** change the `sherpa-onnx` feature split in `Cargo.toml` (T2 owns that); do **not** remove the prebuilt-DLL copy path; do **not** change the CRT workaround; do **not** add a runtime dependency — if you need a hashing crate you **must not add one**: implement the comparison with `std::fs::metadata` (`len()` + `modified()`) only.
  Parallelization: Wave 1 | Blocked by: T1, T2 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/build/sherpa.rs:20` — `.ok()?.flatten()` ReadDir error swallow.
  - `frontend/src-tauri/build/sherpa.rs:46-50` — `PROFILE` env read with `unwrap_or_default()` (line 46) and the vague `expect` message (line 50).
  - `frontend/src-tauri/build/sherpa.rs:55-63` — `unwrap_or_else` ancestor fallback that does not walk far enough to find `<target>/sherpa-onnx-prebuilt/` in the `--target` layout.
  - `frontend/src-tauri/build/sherpa.rs:73-80` — length-only freshness check.
  - `frontend/src-tauri/build/sherpa.rs:104-108` and `:115-121` — misleading panic texts mentioning a "target cfg".
  - `frontend/src-tauri/src/build_env.rs` — the NEW dependency-free lib file holding the testable profile-dir resolution + freshness helpers (declared `pub mod build_env;` in `frontend/src-tauri/src/lib.rs`; `include!`-ed by the build script).
  - `frontend/src-tauri/Cargo.toml:200-208` — the sherpa-onnx feature split this build script must stay consistent with.
  - `frontend/src-tauri/src/sherpa_onnx/model.rs` — the runtime model discovery that consumes the DLLs this script stages.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 (the build script compiles).
  - A new `#[cfg(test)] mod tests` inside the NEW lib file `frontend/src-tauri/src/build_env.rs` (declared as `pub mod build_env;` in `frontend/src-tauri/src/lib.rs`, and pulled into `frontend/src-tauri/build/sherpa.rs` via `include!("../src/build_env.rs");`) contains: (a) a test proving a same-size/different-content prebuilt file is treated as **stale** (freshness check must return "needs copy"), and (b) a test proving the ancestor search finds a `sherpa-onnx-prebuilt` directory when the profile dir is `<target>/debug` (i.e. the `out`/`debug` false-negative case).
  - `cargo test -p meetily` runs the new tests and they pass; the pre-existing failures list from T1 is unchanged (no new failures).
  - `grep -n 'unwrap_or_default' frontend/src-tauri/build/sherpa.rs` no longer matches the `PROFILE` read.
  - The panic strings no longer contain the phrase "target cfg for windows".
  QA scenarios (name the exact tool + invocation):
  - Happy: the new freshness test fails on the pre-fix code (record the red output in the evidence file), passes after the fix.
  - Failure: temporarily point the freshness check at two equal-length files with different bytes and assert the check reports "stale" — if it reports "fresh", the test is red and the fix is incomplete. Also assert that a missing `PROFILE` env var produces a **panic with a message naming PROFILE**, not a silent default.
  Evidence: `.omo/evidence/task-3-fork-review-fixes.txt`
  Commit: Y | `build(sherpa): detect stale prebuilt DLLs and fail loudly on build errors`

- [x] 4. `frontend/src-tauri/build/onnxruntime.rs` — add download retry with backoff and fix panic message casing (2 findings)
  What to do / Must NOT do:
  1. **Retry with backoff (medium/maintainability, lines 193-207).** `download_archive` (defined at line 193) performs a single GET with a 600s timeout (`.timeout(std::time::Duration::from_secs(600))`, line 195) and no retry, so a transient network blip fails the whole build. Wrap the request in a retry loop of **3 attempts** with exponential backoff of 1s → 2s → 4s, re-creating the client per attempt. Log each attempt with `cargo:warning=` (or `println!` with the `cargo:` prefix) so the build log shows the retry count. **Testability:** move the pure backoff-schedule helper into the new dependency-free lib file `frontend/src-tauri/src/build_env.rs` (see the shared testability note in T3) so it can be unit-tested without network access.
  2. **Panic message casing (low/style, all `panic!`/`expect` messages in this file).** The `panic!` messages in this file already start uppercase (e.g. `"ONNX Runtime is bundled only for …"`, `"Failed to stage bundled ONNX Runtime: …"`); the lowercase offenders are the `expect(...)` strings (e.g. `"TARGET environment variable not set"` at line 52, `"CARGO_MANIFEST_DIR environment variable not set"` at line 58, `"ONNX Runtime destination has no parent"` at line 119). Capitalize the first letter of every `panic!`/`expect`/`unwrap` message in the file. Do not change the message *content* beyond the first character. (There is no `"Downloading…"` message in this file — do not invent one.)

  **Must NOT do:** do **not** change the download URL, the expected archive layout, the checksum/sha-verification behaviour, the 600s timeout value, or the onnxruntime version; do **not** add a crate (no `reqwest-retry`); do **not** add a runtime dependency; implement the retry loop with `std::thread::sleep` inside the build script.
  Parallelization: Wave 1 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/build/onnxruntime.rs:193-207` — `download_archive` (single GET, 600s timeout at line 195).
  - `frontend/src-tauri/build/onnxruntime.rs` — every `panic!`/`expect`/`unwrap` message string in the file (the style finding is file-wide; the lowercase offenders are the `expect(...)` strings at lines 52, 58 and 119).
  - `frontend/src-tauri/src/build_env.rs` — the NEW dependency-free lib file holding the testable download retry/backoff schedule helper (declared `pub mod build_env;` in `frontend/src-tauri/src/lib.rs`; `include!`-ed by the build script).
  - `frontend/src-tauri/build/sherpa.rs:73-80` — the analogous freshness-check pattern in the sibling build script (for consistent style).
  - `.github/verify-windows-dlls.ps1` — the Windows CI script that consumes the onnxruntime DLL this build script downloads (T32).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - A new `#[cfg(test)] mod tests` inside the NEW lib file `frontend/src-tauri/src/build_env.rs` (declared as `pub mod build_env;` in `frontend/src-tauri/src/lib.rs`, and pulled into `frontend/src-tauri/build/onnxruntime.rs` via `include!("../src/build_env.rs");`) proves the backoff schedule is 1s/2s/4s over 3 attempts — the schedule is a `const`/fn so it is testable without network access.
  - `grep -nE '(panic!|expect|unwrap)\("[a-z]' frontend/src-tauri/build/onnxruntime.rs` returns **no** hits (all `panic!`/`expect`/`unwrap` messages capitalized). The pattern must include `expect`/`unwrap`, not just `panic!` — the lowercase offenders in this file are `expect(...)` strings at lines 52, 58 and 119.
  - `cargo test -p meetily` shows the new test passing and no new failures beyond the T1 baseline.
  QA scenarios (name the exact tool + invocation):
  - Happy: unit test asserts the retry schedule `[1, 2, 4]` seconds and that the attempt count is 3.
  - Failure: simulate a transport error for the first 2 attempts and assert the download **succeeds** on the 3rd (the test must inject the failure — factor the request into a closure/parameter so this is testable without network). If the test cannot be made hermetic, assert instead that the attempt counter reaches 3 and the error message names the attempt count.
  Evidence: `.omo/evidence/task-4-fork-review-fixes.txt`
  Commit: Y | `build(onnxruntime): retry onnxruntime download with backoff`

- [x] 5. `frontend/build-gpu.sh` — make binary path resolution cwd-independent and harden the ORT feature parsing (3 findings)
  What to do / Must NOT do:
  1. **Path resolution (high/bug, file-level).** `FRONTEND_DIR` is set to `"."` (line 41) or `"frontend"` (line 47) — never absolute — so `WORKSPACE_ROOT=$FRONTEND_DIR/..` is cwd-relative, and the `target/release/$BASE_BINARY` fallback at line 195 assumes the current working directory. Resolve paths **absolutely and canonically** at the top of the script: derive `SCRIPT_DIR` from `${BASH_SOURCE[0]}` (resolved with `cd "$(dirname "${BASH_SOURCE[0]}")" && pwd`), derive `FRONTEND_DIR` and `WORKSPACE_ROOT` from it with `realpath`/`cd ... && pwd`, and use **absolute** paths for every `target/...` reference so no `cd` inside the script can change their meaning.
  2. **The `openblas|*)` catch-all (low/maintainability, lines 117-120).** The catch-all silently swallows typos in the GPU string. Replace the catch-all with an explicit `*)` arm that **prints the recognised values and exits non-zero** on an unknown value, and fix the misleading message that calls OpenBLAS a "CPU fallback" in a GPU-detection context — OpenBLAS is CPU linear algebra, not a GPU. Say so plainly.
  3. **ORT feature parsing (medium/maintainability, lines 218-225).** `${ORT_FEATURES#--features }` is brittle: it breaks when the value is empty, unset, contains multiple features, or differs in whitespace. Parse it robustly (strip an optional leading `--features` prefix with a whitespace-tolerant expression, then split on `,` and trim each element). Note the actual contract: `frontend/scripts/auto-detect-gpu.js` emits a **single** GPU token (not a comma-joined list), and `frontend/scripts/tauri-auto.js` never splits on commas — so the comma-join contract is **not** shared with the whisper detection step or the `ort` feature selection. Add a comment block documenting the parser's own contract and naming the real call sites (`frontend/build-gpu.sh` itself and the `ort-*` features in `frontend/src-tauri/Cargo.toml`).

  4. **`set -e` turns the llama-helper failure branch into dead code (hardening item, beyond the counted findings — medium/bug, lines 146-162).** The script does `HELPER_FEATURES="--features $LLAMA_FEATURE"`, then `(cd "$HELPER_DIR" && cargo build --release $HELPER_FEATURES)`, then `if [ $? -ne 0 ]; then echo "❌ Failed to build llama-helper"; exit 1; fi`. Because `set -e` is active, a failing subshell aborts the script before the `if` is ever evaluated, so the friendly message can never print. Rewrite it as one conditional that owns the failure: `if ! (cd "$HELPER_DIR" && cargo build --release $HELPER_FEATURES); then echo "❌ Failed to build llama-helper"; exit 1; fi`. Keep the unquoted `$HELPER_FEATURES` expansion — it deliberately word-splits into two argv entries (`--features` and its value) and quoting it would pass one argument and break the build. This is the same verified-false trap recorded in `### Must NOT have` for this file: do **not** "fix" it by quoting (a bash array is the only safe hardening and is optional).
  5. **GPU auto-detection failure is silently swallowed (hardening item, beyond the counted findings — medium/bug, lines 69-84).** `TAURI_GPU_FEATURE=$(node scripts/auto-detect-gpu.js)` runs with no status check, so a Node crash (missing `node_modules`, syntax error, non-zero exit) yields an empty string and the script prints the misleading `⚠️ No specific GPU feature detected or forced` before silently building CPU-only. Capture the status and fail loudly: `if ! TAURI_GPU_FEATURE=$(node scripts/auto-detect-gpu.js); then echo "❌ GPU auto-detection failed: node scripts/auto-detect-gpu.js exited non-zero"; exit 1; fi`, and when the script is run in forced mode keep the existing `TAURI_GPU_FEATURE` override intact. Then validate any non-empty detected value against the same known-GPU-string list used by item 2 and exit non-zero on an unknown token instead of falling through to `openblas`.

  **Must NOT do:** do **not** change the set of supported GPU strings or the ORT feature names; do **not** change the comma-joined `--features` format (it is valid — only the *parser* is brittle); do **not** change the `tauri-auto.js` contract; do **not** add a dependency; do **not** change the script's exit codes for the success path.
  Parallelization: Wave 1 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/build-gpu.sh` (whole file) — the `WORKSPACE_ROOT`/`FRONTEND_DIR` derivation, the `target/release/$BASE_BINARY` fallback, and every `cd` that makes relative paths ambiguous.
  - `frontend/build-gpu.sh:117-120` — the `openblas|*)` match arm and the "CPU fallback" message.
  - `frontend/build-gpu.sh:218-225` — the `${ORT_FEATURES#--features }` expansion.
  - `frontend/scripts/tauri-auto.js` — the JS consumer of the GPU feature token (it does **not** split on commas; `frontend/scripts/auto-detect-gpu.js` emits a single token).
  - `frontend/src-tauri/Cargo.toml:55-60` — the `ort-*` features this script selects (T2 owns the manifest side of the same contract).
  - `frontend/build-gpu.sh` — the script itself (there is **no** `build-gpu.sh` entry in `frontend/package.json`; the script is invoked directly).
  Acceptance criteria (agent-executable):
  - `bash -n frontend/build-gpu.sh` exits 0 (`shellcheck` is **not installed** — `bash -n` is the available static check).
  - A new test script `frontend/tests/scripts/build-gpu-path-resolution.test.sh` (run with `bash`) proves: (a) running the script from `/tmp` (an unrelated cwd) still resolves `WORKSPACE_ROOT` to the repo root; (b) an unknown GPU string causes a non-zero exit with a message listing the valid values; (c) `--features a,b` and `--features a, b` both parse to the same feature list, and an empty value yields an empty list rather than the literal `--features `.
  - `grep -n 'WORKSPACE_ROOT=' frontend/build-gpu.sh` shows an absolute, canonical derivation from `BASH_SOURCE`.
  - The evidence file records that `shellcheck` is unavailable on this host.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cd /tmp && bash /home/tango/repo/meetily/.worktrees/fork-review-fixes/frontend/tests/scripts/build-gpu-path-resolution.test.sh` exits 0.
  - Failure: `TAURI_GPU_FEATURE=bogus bash frontend/build-gpu.sh` must **fail loudly** with the list of valid values; assert a non-zero exit code and that the message does not say "CPU fallback". (The script reads no CLI arguments, so the catch-all is reached only via the `TAURI_GPU_FEATURE` env var — a `--gpu=bogus` argument would be ignored, not parsed.)
  Evidence: `.omo/evidence/task-5-fork-review-fixes.txt`
  Commit: Y | `build(script): resolve gpu build paths absolutely and harden feature parsing`

- [x] 6. `llama-helper/Cargo.toml` — make the llama.cpp backend optional and document the exact pin (2 findings)
  What to do / Must NOT do:
  1. **Optional backend (medium/performance, lines 10-14).** `llama-cpp-2` and `llama-cpp-sys-2` are hard dependencies, so the full llama.cpp is always compiled. Introduce a `backend` feature (default-on) that gates them: `llama-cpp-2 = { version = "=0.1.146", optional = true }` plus `backend = ["dep:llama-cpp-2", "dep:llama-cpp-sys-2"]` and `default = ["backend"]`. Then add a minimal no-backend code path in the crate so `cargo check -p llama-helper --no-default-features` still compiles (return a clear "backend not compiled in" error at runtime). **If** the crate cannot compile without the backend without a large refactor, do **not** force it: instead document the finding as accepted with a comment explaining the coupling, and record the reason in the evidence file.
  2. **Exact pin documentation (medium/maintainability, lines 10-14).** `=0.1.146` freezes the crate. Do **not** change the pin to `^0.1.146` (that is a dependency-version change and is forbidden). Instead add a comment recording why the exact pin is used (reproducible builds of the sidecar against a fixed llama.cpp ABI) and what CI would be needed to relax it.

  **Must NOT do:** do **not** change any version number (`=0.1.146` stays); do **not** run `cargo update`; do **not** add a dependency; do **not** change the `llama-helper` binary name or the Tauri `externalBin` sidecar wiring; do **not** change the `llama-cpp-sys-2` build behaviour.
  Parallelization: Wave 1 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `llama-helper/Cargo.toml:10-14` — the `llama-cpp-2` / `llama-cpp-sys-2` exact pins and the `[features]` table (if present).
  - `llama-helper/src/main.rs` (or the crate's entrypoint) — where the llama backend is constructed; the no-backend path must be wired here.
  - `frontend/src-tauri/tauri.conf.json` — the `externalBin` sidecar declaration for `llama-helper` (the binary name must not change).
  - `frontend/src-tauri/src/llama_helpers` / the Rust code that spawns the sidecar (if it lives outside the `llama-helper` crate).
  Acceptance criteria (agent-executable):
  - `cargo check -p llama-helper` exits 0.
  - `cargo check -p llama-helper --no-default-features` exits 0 **if** the optional-backend path was implemented; if it was not implementable, the evidence file contains an explicit `ACCEPTED` note with the technical reason and a code comment at `llama-helper/Cargo.toml:10-14`.
  - `grep -n '0.1.146' llama-helper/Cargo.toml` shows the pin unchanged and now accompanied by an explanatory comment.
  - `git diff llama-helper/Cargo.toml` contains no changed version strings (only added comments / feature plumbing).
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo check -p llama-helper` and `cargo check -p llama-helper --no-default-features` both exit 0; `cargo tree -p llama-helper --no-default-features | grep llama-cpp` returns **no** hits (the backend is genuinely not compiled).
  - Failure: `cargo check -p llama-helper --no-default-features` fails to compile ⇒ the optional path is incomplete; either fix it or revert to the documented-accepted state and record the reason. Do not leave a half-wired feature.
  Evidence: `.omo/evidence/task-6-fork-review-fixes.txt`
  Commit: Y | `build(llama-helper): make llama.cpp backend optional behind a feature`

- [x] 7. `frontend/package.json` — de-duplicate `@tauri-apps/plugin-fs`, add the missing `eslint` / `eslint-config-next` devDependencies, and document the prosemirror overrides (3 findings)
  What to do / Must NOT do:
  1. **Duplicate `@tauri-apps/plugin-fs` (medium/maintainability, file-level).** It appears in `dependencies` as `^2.4.0` and in `devDependencies` as `~2.4.0`. **The stated justification is FALSE** — `@tauri-apps/plugin-fs` is imported **nowhere** in `frontend/src` (verified: `grep -rn 'plugin-fs' frontend/src` returns no hits), so "keep the runtime one in `dependencies`, since it is imported by application code" does not apply and the old decision rule yields no answer. Re-decide on the ACTUAL import graph and record the reason in the evidence file: if the package is only needed by the Tauri build toolchain, keep exactly ONE copy in `devDependencies` (`~2.4.0`) and remove the `dependencies` copy; if a Rust-side runtime need (the `tauri-plugin-fs` capability or a future `frontend/src` import) justifies `dependencies`, state that concrete reason — do **not** restate the false "imported by application code" claim. Either way the key MUST appear exactly once.
  2. **Missing `eslint` (high/other, file-level).** The `lint` script runs `eslint .` but `eslint` is not in `devDependencies`, so a clean install cannot lint. Add BOTH `eslint` and `eslint-config-next` to `devDependencies` as **explicit literal versions** — `"eslint": "9.17.0"` (the ESLint 9 line that matches this repo's flat config in `frontend/eslint.config.mjs`) and `"eslint-config-next": "14.2.25"` (the release that matches the repo's `"next": "^14.2.25"`). **Do NOT run `pnpm why eslint` and do NOT copy a version out of the lockfile**: the fresh worktree has no `node_modules` (so `pnpm why eslint` is unexecutable without a network install), `eslint` appears in `frontend/pnpm-lock.yaml` **zero** times, and `eslint-config-next` is required by `frontend/eslint.config.mjs:10` (`require("eslint-config-next/core-web-vitals")`) so the flat config cannot even load without it. If `pnpm install` reports a peer-range warning for `eslint-config-next@14.2.25` (its published peer range may predate ESLint 9), record the warning verbatim in the evidence file and bump ONLY `eslint-config-next` to the newest 14.2.x that declares ESLint 9 support — do **not** change `next`. After `pnpm install` succeeds, capture the **ESLint baseline** that F2.5 diffs against: `cd frontend && npx eslint . 2>&1 | tee /home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/t7-eslint-baseline.txt`, and record `BASELINE_ESLINT_ERRORS=<n>` in this todo's evidence file. **The baseline is captured HERE, in T7, not in T1** — the `eslint` binary only exists after this todo's `pnpm install`. Run the **local** binary (`frontend/node_modules/.bin/eslint`, i.e. `npx eslint` resolved from `frontend/node_modules`) and record the exact resolved ESLint version in the evidence file: a **global** ESLint (e.g. v6.4.0) rejects this repo's flat config and exits 2, which would produce a bogus baseline.
  3. **Prosemirror overrides (medium/performance, file-level).** The `pnpm.overrides` block pins 5 prosemirror packages (`model 1.25.3`, `state 1.4.3`, `view 1.41.3`, `transform 1.10.4`, `tables 1.8.1`) shared by `@tiptap`/`@remirror`. This is a **deliberate accepted risk** per `AGENTS.md` (ProseMirror version-pinned via pnpm overrides; do NOT bump them independently). Add only an explanatory comment stating the pin, the reason (BlockNote/TipTap singleton compatibility — the webpack aliases in `next.config.js` keep them on one singleton), and the instruction never to bump them independently.

  **Must NOT do:** do **not** remove or change any `pnpm.overrides` version; do **not** remove the webpack aliases in `next.config.js`; do **not** change any other dependency version; do **not** run `pnpm update`; do **not** change the `lint` script's command (`next lint` is ESLint-v9-incompatible — the script must stay `eslint .`); do **not** touch the `tauri:*` scripts.
  Parallelization: Wave 1 | Blocked by: T1 | Blocks: T8
  References (executor has NO interview context - be exhaustive):
  - `frontend/package.json` — `dependencies` and `devDependencies` blocks (the duplicated `@tauri-apps/plugin-fs`), the `lint` script, and the `pnpm.overrides` block.
  - `frontend/eslint.config.mjs` — the flat config that consumes the `eslint` binary and `eslint-config-next` (T8).
  - `frontend/next.config.js` — the webpack aliases that keep BlockNote/TipTap on one ProseMirror singleton (the reason the overrides exist).
  - `AGENTS.md` constraints #2 (ProseMirror version-pinned via pnpm overrides — do NOT bump independently) and the pnpm v8 requirement.
  - `frontend/pnpm-lock.yaml` — contains **no** `eslint` / `eslint-config-next` entries (verified), so there is no resolved version to copy; the versions are pinned as literals in `frontend/package.json` and the lockfile is regenerated by `pnpm install` (commit the updated lockfile).
  Acceptance criteria (agent-executable):
  - `node -e "const p=require('./frontend/package.json'); const d={...p.dependencies,...p.devDependencies}; console.log(Object.keys(d).filter(k=>k==='@tauri-apps/plugin-fs'||k==='eslint'))"` prints each of the two keys **exactly once** (no key present in both blocks).
  - `cd frontend && pnpm install --frozen-lockfile=false` succeeds and `node -e "require.resolve('eslint/package.json')"` resolves from `frontend/node_modules`.
  - `frontend/package.json` `devDependencies` contains BOTH explicit literals — verify with `node -e "const d=require('./frontend/package.json').devDependencies; console.log(d.eslint, d['eslint-config-next'])"`, which must print `9.17.0 14.2.25`; and `node -e "require.resolve('eslint-config-next/package.json', {paths:['frontend']})"` resolves (without it `frontend/eslint.config.mjs:10` throws at lint time).
  - `cd frontend && npx eslint .` exits 0 (or exits with only pre-existing findings, which must be recorded in the evidence file as the baseline) **and the flat config actually loaded** — `cd frontend && npx eslint --print-config src/app/page.tsx` exits 0 and prints a non-empty config (a file matched by no config object is silently ignored, which would make this check vacuous).
  - `/home/tango/repo/meetily/.worktrees/fork-review-fixes/.omo/evidence/t7-eslint-baseline.txt` exists and is non-empty, and this todo's evidence file records `BASELINE_ESLINT_ERRORS=<n>` — this is the file F2.5 diffs against.
  - `git diff frontend/package.json` shows the 5 prosemirror override versions **unchanged** (verify with `git diff frontend/package.json | grep -E '^-.*(prosemirror|1\.25\.3|1\.4\.3|1\.41\.3|1\.10\.4|1\.8\.1)'` returning no removed version lines).
  QA scenarios (name the exact tool + invocation):
  - Happy: `cd frontend && pnpm install && npx eslint .` runs (the `eslint` binary resolves), and `pnpm run lint` no longer fails with "eslint: command not found".
  - Failure: `cd frontend && rm -rf node_modules && pnpm install` must still make `pnpm run lint` runnable — if `eslint` is still missing, the devDependency was added to the wrong block or with an unpinned version; fix and re-verify.
  Evidence: `.omo/evidence/task-7-fork-review-fixes.txt`
  Commit: Y | `chore(deps): de-duplicate plugin-fs and add missing eslint/eslint-config-next devDependencies`

- [x] 8. `frontend/eslint.config.mjs` — extend the glob and comment the two workarounds (3 findings)
  What to do / Must NOT do:
  1. **Glob excludes this very file (medium/maintainability, line 18).** The pattern `**/*.{js,jsx,ts,tsx}` excludes `.mjs`/`.cjs` — so `frontend/eslint.config.mjs` is not linted by its own config. Extend it to `**/*.{js,jsx,ts,tsx,mjs,cjs}`.
  2. **`createRequire` rationale (low/documentation, line 8).** Add a comment explaining: `eslint-config-next` is CommonJS, this project uses ESM flat config, and `createRequire(import.meta.url)` is the standard bridge; name the minimum Node version required.
  3. **`Array.isArray(nextCoreWebVitals)` guard rationale (low/documentation, line 16).** Add a comment/TODO naming the `eslint-config-next` version in which `next/core-web-vitals` becomes a stable, non-array export, so the defensive guard can be removed later.

  **Must NOT do:** do **not** change the ESLint rules themselves (no rule additions/removals/severity changes); do **not** change the `eslint-config-next` import; do **not** remove the defensive `Array.isArray` guard (only document it); do **not** change the `ignores` list semantics; do **not** add a dependency.
  Parallelization: Wave 1 | Blocked by: T1, T7 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/eslint.config.mjs:5-6` — `const __filename = fileURLToPath(import.meta.url);` (L5) and `const __dirname = path.dirname(__filename);` (L6) — `__filename` IS consumed by L6, so **do not delete it** (the "dead `__filename`" finding is not reproducible and is dropped from this todo).
  - `frontend/eslint.config.mjs:8` — the `createRequire` import/usage.
  - `frontend/eslint.config.mjs:16` — the `Array.isArray(nextCoreWebVitals)` guard.
  - `frontend/eslint.config.mjs:18` — the `files` glob `**/*.{js,jsx,ts,tsx}`.
  - `frontend/package.json` — the `lint` script (`"lint": "eslint ."`, L28) and, after T7, the `devDependencies` entries `"eslint": "9.17.0"` and `"eslint-config-next": "14.2.25"` (L98-115 before T7 — neither package is present in the repo today; T7 adds both).
  - `AGENTS.md` — "pnpm run lint → Next.js lint (ESLint, no config file in repo)"; note the script is now `eslint .` because `next lint` is ESLint-v9-incompatible.
  Acceptance criteria (agent-executable):
  - `cd frontend && npx eslint .` exits 0 **and the flat config actually loaded** — `cd frontend && npx eslint --print-config src/app/page.tsx` exits 0 and prints a non-empty config (under ESLint v9 a file matching no config object's `files` pattern is silently ignored, which would make this criterion vacuous).
  - `npx eslint --print-config eslint.config.mjs >/dev/null` exits 0 (the config is valid flat config).
  - `grep -n 'files:' -A2 frontend/eslint.config.mjs` shows the glob contains `mjs` and `cjs`.
  - `cd frontend && npx eslint --no-ignore --print-config eslint.config.mjs` exits 0 and prints a **non-empty** config (proving the config file is now matched by the extended glob instead of being silently ignored).
  - A file that IS matched by the config's `files` globs is really linted and clean: `cd frontend && npx eslint src/app/page.tsx` exits 0 and reports no errors in it (this is the non-vacuous "reports no errors" check — a file matching no pattern exits 0 without linting anything).
  - `git diff frontend/eslint.config.mjs` shows only added comments + the extended glob; **no removal of `__filename`** (L5-6 stays intact) and no rule changes.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cd frontend && npx eslint .` exits 0 and the config file itself is included in the linted set (`npx eslint --debug eslint.config.mjs 2>&1 | grep -q 'eslint.config.mjs'`).
  - Failure: introduce a temporary unused-variable error in a `.mjs` file under `frontend/` and confirm `npx eslint .` **reports** it (proving the glob extension took effect); then remove the temporary file.
  Evidence: `.omo/evidence/task-8-fork-review-fixes.txt`
  Commit: Y | `chore(lint): extend eslint glob and document flat-config workarounds`

- [x] 9. `frontend/src-tauri/src/funasr_onnx/mod.rs` — make `FunasrError` `Send + Sync`, preserve error sources, and stop leaking internals (6 findings)
  What to do / Must NOT do:
  1. **`Send + Sync` (high/bug, lines 4-5).** `FunasrError` is used across `await` points and moved into `tokio::task::spawn_blocking` (T23/T14). Add `#[derive(Debug, Send, Sync)]` on top of the existing `thiserror::Error` derive (which supplies `Display` / `std::error::Error`). **`Clone`, `PartialEq` and `Eq` must ALL be dropped, not just `Clone`**: `FunasrError::Ort(#[from] ort::Error)` (mod.rs:7) is the blocker — `ort::Error` is declared `#[derive(Debug)]` only (registry `ort-2.0.0-rc.10/src/error.rs:27-28`), so it is **neither `Clone` nor `PartialEq`/`Eq`**, and the other `#[from]` variants (`std::io::Error`, `serde_json::Error`) have the same limitation. If a test needs equality, implement it by hand with a `matches!`-based helper that compares the variant discriminant and the `String`/`PathBuf` payloads instead of deriving `PartialEq`.
  2. **`#[source]` (medium/maintainability, lines 14-15).** `DecodeError`, `InvalidTokenFile`, `InvalidCmvnFile` currently stringify their inner error, losing the chain. Restructure them to carry the source type (`ort::Error`, `std::io::Error`) and annotate with `#[source]`, or box the source as `Box<dyn std::error::Error + Send + Sync + 'static>` with `#[source]`. This requires the inner errors to be `Send + Sync` — verify that for `ort::Error`.
  3. **Typed payload paths (low/maintainability, lines 8-15).** `ModelNotFound(String)`, `InvalidTokenFile(String)`, `InvalidCmvnFile(String)` should carry `PathBuf` (the offending path) rather than a `String`. Update every construction site and every pattern match (grep for these variant names across the crate and the rest of the workspace) — T10/T11/T12/T13/T14 and T23/T25/T26 construct/match them.
  4. **`AudioTooShort` (low/maintainability, lines 16-17).** The finding's claim that the fields are positional is **stale** — the code already uses named fields `min`/`actual`. The actionable part: rename the fields to `expected`/`got`, add a doc comment, and keep the named-struct syntax.
  5. **Wildcard re-exports (medium/maintainability, lines 32-39).** `pub use decoder::*;` (line 33) and `pub use commands::*;` (line 39) leak internals. Replace them with **explicit** re-export lists (the pattern the other submodules in this file already use), keeping only the items the rest of the crate actually uses.
  6. **Module docs (implicit).** While editing, ensure the crate root's module doc states the public surface of the FunASR module (engine lifecycle, error contract).

  **Must NOT do:** do **not** change the error *semantics* (which condition produces which variant); do **not** remove an error variant; do **not** change the crate's public function signatures beyond the error-type changes listed; do **not** add a dependency (`thiserror` may already be present — check `Cargo.toml` before using it; if it is absent, write the `Display`/`Error` impls by hand).
  Parallelization: Wave 2 | Blocked by: T1 | Blocks: T10, T11, T12, T13, T14, T23
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:4-5` — the `FunasrError` enum declaration (missing `Send + Sync`).
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:8-9` — `ModelNotFound(String)`.
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:10-21` — `InvalidTokenFile` (L10-11), `DecodeError` (L14-15) and `InvalidCmvnFile` (L20-21) — the range must reach L20-21, because `InvalidCmvnFile` sits outside `10-15`.
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:16-17` — `AudioTooShort` (already named fields `min`/`actual`).
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:32-39` — the `pub use decoder::*;` and `pub use commands::*;` wildcard re-exports.
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:13` — the **fully-qualified** `crate::funasr_onnx::FunasrError` path in the `new` signature (`pub fn new(model_dir: &Path) -> Result<Self, crate::funasr_onnx::FunasrError>`) — NOT an import; the file's only `use` of the module is `use crate::funasr_onnx::FunasrEngine;` (L6) (T23 depends on this).
  - `frontend/src-tauri/src/audio/import.rs:18` and `frontend/src-tauri/src/audio/retranscription.rs:19` — import `crate::funasr_onnx::FunasrEngine` (T25/T26 depend on this).
  - `frontend/src-tauri/src/funasr_onnx/{decoder.rs,engine.rs,model.rs,frontend.rs,commands.rs}` — the submodules whose items are re-exported here (T10–T14).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - A new `#[cfg(test)]` in `frontend/src-tauri/src/funasr_onnx/mod.rs` contains a **compile-time** assertion that `FunasrError: Send + Sync` (e.g. `fn assert_send_sync<T: Send + Sync>() {}` + `assert_send_sync::<FunasrError>();`) and a runtime assertion that the `#[source]` chain is preserved (`std::error::Error::source(&err).is_some()` for a constructed `DecodeError`).
  - `grep -n 'pub use .*::\*' frontend/src-tauri/src/funasr_onnx/mod.rs` returns **no** hits.
  - `cargo test -p meetily` passes with no new failures vs the T1 baseline.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily funasr_onnx` passes; the new test proves `FunasrError` is `Send + Sync` and that the source chain is intact.
  - Failure: remove the `Send` bound temporarily and confirm the compile-time assertion test **fails to compile** (proving the assertion is real); restore it. Also assert that `std::error::Error::source()` returns `None` for a variant that has no source and `Some` for one that does.
  Evidence: `.omo/evidence/task-9-fork-review-fixes.txt`
  Commit: Y | `refactor(funasr): make FunasrError Send+Sync and preserve error sources`

- [x] 10. `frontend/src-tauri/src/funasr_onnx/engine.rs` — remove the dead `MODELS_DIR`, return errors instead of swallowing them, and fix the check-then-act race (5 findings)
  What to do / Must NOT do:
  1. **Dead `MODELS_DIR` (high/maintainability, lines 9-10).** `MODELS_DIR` is written by `initialize_models_directory` but never read. Either remove it and derive the model dir from the global, or keep it as the single source of truth and make the reader use it. Prefer: make it the single source of truth and have every consumer derive from it.
  2. **`Mutex<Option<PathBuf>>` → `OnceLock<PathBuf>`/`LazyLock` (low/maintainability, line 10).** Replace the mutex-guarded `Option` with a `OnceLock<PathBuf>` (or `LazyLock`) — it is initialized once and never mutated.
  3. **Errors swallowed (high/other, lines 14-26).** `initialize_models_directory` currently `return`s on the error path, discarding the error and leaving a partially-initialized state. Change the signature to return `Result<(), FunasrError>` and propagate.
  4. **`expect()` on `app_data_dir()` (high/other, lines 15-19).** `expect` crashes the app in sandbox/permission cases. Change the signature to propagate the error AND to take the base directory as an explicit input, so the failure path is testable without a Tauri app handle: `pub fn initialize_models_directory_at<R: Runtime>(app: &tauri::AppHandle<R>, base_dir: &Path) -> Result<PathBuf, FunasrError>` — `app` is used only to resolve the default app-data root, `base_dir` overrides it (production passes the app-data root). Add a thin wrapper `pub fn initialize_models_directory<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, FunasrError>` that resolves `app.path().app_data_dir()?` (as a `Result`, never `expect`) and delegates to `initialize_models_directory_at`. The explicit `base_dir` parameter is REQUIRED — without it the QA scenario below (a path whose parent is a regular file) is not writable, because the current signature derives the path internally from `app.path().app_data_dir()` (engine.rs:14-19).
  5. **Check-then-act race (high/bug, lines 21-26).** `exists()` followed by `create_dir_all()` is a TOCTOU race. Call `create_dir_all()` directly and treat `ErrorKind::AlreadyExists` as benign.
  6. **Caller update — RECORD ONLY, do not apply it here.** `frontend/src-tauri/src/lib.rs:616` calls `crate::funasr_onnx::engine::initialize_models_directory(&_app.handle())`; the new signature returns `Result<…>`, so that call site must handle the error. **This todo changes ONLY `funasr_onnx/engine.rs`.** Write the exact required lib.rs edit (the new call expression and how the `Err` arm should be handled — e.g. `if let Err(e) = … { error!(…) }` or propagating into the existing setup function's error path) into the evidence file under a heading `REQUIRED lib.rs CHANGE (applied by T29)`. **Todo 29 owns `frontend/src-tauri/src/lib.rs` and applies it** — T29's dependency list already includes T10 for exactly this reason.

  **Must NOT do:** do **not** edit `frontend/src-tauri/src/lib.rs` in this todo (T29 owns that file — a concurrent edit there would collide with T29's commit); do **not** change the on-disk model directory layout or naming; do **not** change the `FunasrError` variants added in T9; do **not** change the initialization order relative to the sherpa-onnx initialization in `lib.rs`; do **not** swallow any error with `unwrap()`/`expect()` in the new code.
  Parallelization: Wave 2 | Blocked by: T1, T9 | Blocks: T11, T14, T29
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/funasr_onnx/engine.rs:9-10` — `MODELS_DIR` and the `Mutex<Option<PathBuf>>`.
  - `frontend/src-tauri/src/funasr_onnx/engine.rs:14-26` — `initialize_models_directory` (error-swallowing `return`s, `expect()` on `app_data_dir()`, `exists()`/`create_dir_all()` race).
  - `frontend/src-tauri/src/lib.rs:616` — the only call site of `initialize_models_directory(&_app.handle())`.
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:8-21` — the `FunasrError` variants this todo must use (`ModelNotFound` L8-9, `InvalidTokenFile` L10-11, `Io` L12-13, `DecodeError` L14-15, `AudioTooShort` L16-17, `InvalidCmvnFile` L20-21) — the range must reach L20-21, because `InvalidCmvnFile` sits outside `8-17`.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:20-25` — the ACTUAL divergent path derivation: `models_dir()` joins `data_dir/models/funasr/paraformer-large`, i.e. it does **not** match the `data_dir/models` root that `MODELS_DIR` (engine.rs:10, written at :30-31) holds — this is the divergence to reconcile (`model.rs` never reads `MODELS_DIR`; it takes the directory as a parameter, `pub fn new(model_dir: &Path)`, model.rs:30).
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:74-88` — the sibling `initialize_models_directory` used at `lib.rs:619` (T17). It returns `()` and **swallows** errors (`if let Err(e) = std::fs::create_dir_all(...) { warn!(...) }`) — the very anti-pattern this todo removes, so there is **no error-handling style to mirror**. Do **not** copy it: T17 must change the sherpa sibling to return `Result` (record the required T17 change in this todo's evidence file, exactly as item 6 does for lib.rs). T29 owns `lib.rs` call-site integration if the signature changes.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `frontend/src-tauri/src/funasr_onnx/engine.rs` prove: (a) `initialize_models_directory_at(app, base_dir)` returns `Err` (not `Ok`/`()`) when the base directory cannot be created — simulate with a path under a non-creatable parent (e.g. a path whose parent is an existing *file*); (b) calling it twice on an existing directory returns `Ok` and does **not** error (the `AlreadyExists` case); (c) the directory path is retrievable from the `OnceLock` and is stable across calls. Add a path parameter (e.g. `initialize_models_directory_at(app, base_dir: &Path)`) to the T10 signature so the non-creatable-parent scenario is testable.
  - `grep -n 'MODELS_DIR' frontend/src-tauri/src/funasr_onnx/engine.rs` shows the constant is either removed or actually read.
  - `grep -n 'expect(' frontend/src-tauri/src/funasr_onnx/engine.rs` returns no hits on `app_data_dir()`.
  - `cargo test -p meetily funasr_onnx` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: the double-initialization test returns `Ok` twice; the error test returns `Err` with a `FunasrError` that has a `source`.
  - Failure: call `initialize_models_directory_at` with a `base_dir` whose parent is a regular file and assert it returns `Err` rather than panicking — if it panics, the `expect()` was not removed. (This scenario REQUIRES the explicit `base_dir` parameter from item 4; if the function still derives its path internally from `app.path().app_data_dir()`, the scenario is not executable — that is the failure signal.)
  Evidence: `.omo/evidence/task-10-fork-review-fixes.txt`
  Commit: Y | `fix(funasr): propagate model directory init errors and remove dead state`

- [x] 11. `frontend/src-tauri/src/funasr_onnx/model.rs` — reuse the already-cached hardware profile, avoid buffer copies, and stop panicking on inference errors (5 findings)
  What to do / Must NOT do:
  1. **GPU probing per construction (low/performance, lines 38-39) — PREMISE NOT REPRODUCIBLE; do NOT add a second cache.** `HardwareProfile::detect()` is already cached process-wide: `frontend/src-tauri/src/audio/hardware_detector.rs:49` declares `static HARDWARE_PROFILE: OnceLock<HardwareProfile> = OnceLock::new();` and `:53` `pub fn detect() -> &'static HardwareProfile { HARDWARE_PROFILE.get_or_init(...) }` — the probe already runs once per process, and `model.rs:38` only reads `.gpu_type` off the returned `&'static` reference. **Do NOT introduce a new `OnceLock`/`LazyLock` cache in `model.rs`** (it would duplicate the existing one and be a no-op). Re-scope this item to: verify by reading `hardware_detector.rs:49,53` that the existing cache covers the probe, then record "NOT REPRODUCIBLE — already OnceLock-cached in audio/hardware_detector.rs:49,53" in the evidence file and change nothing in `model.rs` for this item.
  2. **Input buffer copy (medium/performance, lines 103-109).** `features.to_vec()` copies the input buffer before the ndarray conversion. Avoid the copy — construct the ndarray view/array directly from the existing buffer (use `ndarray::ArrayView`/`ArrayViewMut` over the borrowed slice, or `Array::from_shape_vec` with the owned buffer instead of `to_vec()`).
  3. **Error context lost (low/other, line 104).** `.to_string()` on the ndarray/ort error drops the source. Preserve it: wrap in a `FunasrError` variant that carries the source (the `DecodeError`/`#[source]` pattern established in T9) and include the `Debug` representation.
  4. **Mutex held across inference (high/other, lines 112-114).** `self.session.lock().unwrap()` is held for the whole `session_guard.run(...)` call, serializing all concurrent transcriptions. Restructure so the lock is taken only to **acquire** the session reference, and the inference runs without holding the guard (e.g. `Arc<Session>` cloned out from under the lock, or `parking_lot`/an `Arc<Mutex<..>>` pattern that releases before `run`).
  5. **Panic risk on missing output keys (medium/other, lines 113-130).** `outputs["logits"]` / `outputs["token_num"]` use `[]` indexing, which panics when a key is missing, and `.unwrap()` panics on a poisoned mutex. Use `.get(...)` and return a typed `FunasrError` instead.
  6. **Poison handling.** Replace `lock().unwrap()` with poison-tolerant handling (recover the inner value or return an error) — the same rule T19 applies to the transcription-engine mutex.

  **Must NOT do:** do **not** change the ONNX model inputs/outputs contract (the tensor names `logits` / `token_num` are fixed by the model); do **not** change the session-creation options; do **not** change the `FunasrModel` public API shape beyond the error changes; do **not** add a dependency; do **not** change the `get_onnx_providers` call sites (T21 owns that file).
  Parallelization: Wave 2 | Blocked by: T1, T9, T10, T21 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/funasr_onnx/model.rs:38-39` — `HardwareProfile::detect()` has a **single** call site at `model.rs:38`; `model.rs:39` and `model.rs:180` are calls to `crate::audio::onnx_provider::get_onnx_providers(...)`, not to `detect()`.
  - `frontend/src-tauri/src/funasr_onnx/model.rs:103-109` — `features.to_vec()` copy + `.to_string()` error context loss.
  - `frontend/src-tauri/src/funasr_onnx/model.rs:112-114` — `self.session.lock().unwrap()` held across `session_guard.run(...)`.
  - `frontend/src-tauri/src/funasr_onnx/model.rs:113-130` — `outputs["logits"]` / `outputs["token_num"]` indexing.
  - `frontend/src-tauri/src/audio/onnx_provider.rs` — `get_onnx_providers(gpu_type)`, called from `model.rs:39` and `model.rs:180` **and** from its own unit tests at `audio/onnx_provider.rs:121, 132, 150, 176` (T21 changes this file; T11 must call the resulting API correctly).
  - `frontend/src-tauri/src/funasr_onnx/mod.rs:4-21` — the `FunasrError` variants T11 must use (the range must reach L20-21, because `InvalidCmvnFile` sits outside `4-17`).
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs` — produces the `features` buffer T11 feeds into the session (T13).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `frontend/src-tauri/src/funasr_onnx/model.rs` prove: (a) `HardwareProfile::detect()` is invoked at most once across two `FunasrModel::new()` calls (assert against the existing `HARDWARE_PROFILE` `OnceLock` in `audio/hardware_detector.rs:49` — no new cache, see item 1); (b) a missing output key produces an `Err` (not a panic) — assert via `std::panic::catch_unwind` that the call does **not** panic and returns `Err`; (c) the inference path does not hold the session lock during `run` — this MUST be an **automated, deterministic test that fails before the fix and passes after it** (red→green, per F1.3): add a test-only hook/counter inside `model.rs` that is incremented at the moment `run` is entered, then run two concurrent inferences and assert that the second one entered `run` while the first was still inside it. **Do NOT satisfy (c) by reading the code, by a code-level/grep check, or with a timing-sensitive flaky test** — the red state must be demonstrated explicitly: record in the evidence file that the new test FAILS against the pre-fix code (guard held across `run`) and PASSES after the fix.
  - `grep -n 'outputs\[' frontend/src-tauri/src/funasr_onnx/model.rs` returns no hits (all lookups use `.get`).
  - `cargo test -p meetily funasr_onnx` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: the concurrency test runs two transcriptions and both complete; the missing-key test returns `Err(FunasrError::...)`.
  - Failure: deliberately pass an output map without `token_num` and assert the function returns `Err`; if it panics, the `.get()` conversion is incomplete. Assert `catch_unwind` returns `Ok(Err(_))`.
  Evidence: `.omo/evidence/task-11-fork-review-fixes.txt`
  Commit: Y | `fix(funasr): avoid buffer copies and panics in model inference`

- [x] 12. `frontend/src-tauri/src/funasr_onnx/decoder.rs` — remove the hot-path allocation, name the magic offset, and handle NaN argmax (6 findings)
  What to do / Must NOT do:
  1. **Hot-path allocation (low/performance, lines 29-33).** `text.split_whitespace().collect::<Vec<_>>().join(" ")` allocates a temporary `Vec` in the hot path. Replace with an iterator/filter that joins directly (e.g. `split_whitespace().collect::<Vec<_>>()` → `itertools`-free manual join via `fold`, or write into a reused `String` buffer).
  2. **Magic `token_num - 1` (medium/maintainability, lines 80-82).** Introduce a named constant (e.g. `const PRED_BIAS: i32 = 1;`) or document the API invariant, and use it at the offset site.
  3. **Missing capacity (low/performance, lines 87-88).** Replace `String::new()` + `push_str` with `String::with_capacity(num_frames * 4)`.
  4. **Duplicate `@@` stripping (low/maintainability, lines 87-101).** `@@` is stripped in two places — the token loop's `strip_suffix` and `postprocess`'s `replace`. Pick **one** location. Prefer keeping the strip in `postprocess` (single choke point) and removing the in-loop `strip_suffix`, OR vice versa — whichever requires the smaller, safer change. Then fix the `postprocess` doc comment accordingly.
  5. **`postprocess` doc (low/documentation, lines 8-12).** The doc claims `postprocess` handles `@@` but callers already strip them. After step 4 the doc must describe the actual behaviour.
  6. **NaN argmax (medium/bug, lines 112-118).** `argmax` uses `unwrap_or(Ordering::Equal)` on a `partial_cmp` that returns `None` for NaN ⇒ silently wrong tokens. Propagate the error (return `Result`) or guard NaN explicitly and fail loudly.

  **Must NOT do:** do **not** change the decoding output format (the token→text contract consumed by `funasr_onnx/frontend.rs` and the transcription providers); do **not** change the `@@` marker semantics (it is the FunASR word-boundary marker — only the *location* of the strip changes); do **not** change the `PRED_BIAS` value; do **not** add a dependency.
  Parallelization: Wave 2 | Blocked by: T1, T9 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:8-12` — the `postprocess` doc comment claiming `@@` handling.
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:29-33` — the `split_whitespace().collect::<Vec<_>>().join(" ")` hot path.
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:80-82` — the magic `token_num - 1` offset.
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:87-88` — `String::new()` without capacity.
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:87-101` — the in-loop `strip_suffix("@@")` plus `postprocess`'s `replace`.
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:112-118` — `argmax` with `unwrap_or(Ordering::Equal)`.
  - `frontend/src-tauri/src/funasr_onnx/decoder.rs:122` — the existing `#[cfg(test)] mod tests` (extend it; do not create a second test module).
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs` — consumes the decoder output (T13).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New/extended `#[cfg(test)]` tests in `frontend/src-tauri/src/funasr_onnx/decoder.rs` prove: (a) `argmax` on a slice containing `f32::NAN` returns an `Err` (or panics-with-message) instead of silently picking an arbitrary index; (b) `postprocess` output is identical before and after the de-duplication of the `@@` strip (golden-string test); (c) a multi-space input is normalized to single spaces.
  - `grep -c 'strip_suffix' frontend/src-tauri/src/funasr_onnx/decoder.rs` + `grep -c 'replace("@@"' frontend/src-tauri/src/funasr_onnx/decoder.rs` — exactly **one** of the two `@@`-stripping mechanisms remains.
  - `grep -n 'token_num - 1' frontend/src-tauri/src/funasr_onnx/decoder.rs` returns no hits (replaced by the named constant).
  - `cargo test -p meetily funasr_onnx` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily funasr_onnx::decoder` passes; the golden-string test confirms byte-identical `postprocess` output.
  - Failure: feed `argmax` a slice `[0.0, f32::NAN, 1.0]` and assert an error is returned (not index 0 chosen silently). If the test passes without an error, the NaN guard is missing.
  Evidence: `.omo/evidence/task-12-fork-review-fixes.txt`
  Commit: Y | `perf(funasr): remove decoder hot-path allocation and handle NaN argmax`

- [x] 13. `frontend/src-tauri/src/funasr_onnx/frontend.rs` — cache the FFT planner, validate CMVN lengths, and reject non-finite audio (5 findings)
  What to do / Must NOT do:
  1. **FFT planner rebuilt per call (high/performance, lines 111-117).** `RealFftPlanner::new()` + `plan_fft_forward()` are rebuilt on **every** `compute_fbank()` call. Cache the planner (and the scratch buffers) at module level in a `OnceLock`/`LazyLock`, exactly like the existing `hamming_window()` / `mel_filterbank()` helpers. Preserve thread-safety: the cached planner must be usable from multiple threads (use `OnceLock` + interior mutability only if the planner requires `&mut`; otherwise compute the plan once into an owned structure).
  2. **Trailing frames dropped (medium/maintainability, lines 189-194).** The `break` silently discards trailing frames when fewer than `LFR_M` (7) remain. Log a warning naming the number of dropped frames, **or** right-pad, **or** return a `Result` signalling dropped frames. Choose the smallest change that does not alter the feature contract — a warning plus a documented invariant is acceptable; a `Result` change requires updating all callers (T23/T25/T26).
  3. **CMVN length mismatch (low/maintainability, lines 291-294).** `apply_cmvn` silently truncates when `means`/`inv_stds` lengths differ from the feature frame count. Validate the lengths and return an error (or panic with a precise message) instead of truncating.
  4. **Non-finite audio (medium/bug, lines 346-347).** `process()` scales the audio without checking for NaN/Inf ⇒ non-finite features reach the ONNX session. Validate finiteness and fail with a clear error.
  5. **Incorrect test comment (low/maintainability, line 460).** The comment says "T_lfr = 2 (ceil(14/6)=2)" but `ceil(14/6) = 3`; the real cause of the observed value is the `break` at `start + LFR_M > padded.len()` (i = 2, start = 12, 12 + 7 > 17). Fix the comment (or the assertion) so it states the true invariant. **Do not change the algorithm to make the comment true** — the comment is wrong, not the code.

  **Must NOT do:** do **not** change the fbank feature dimensions (the model input shape is fixed); do **not** change the LFR_M value (7); do **not** change the CMVN math; do **not** change the FFT size; do **not** add a dependency.
  Parallelization: Wave 2 | Blocked by: T1, T9 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs:111-117` — `RealFftPlanner::new()` + `plan_fft_forward()` inside `compute_fbank()`.
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs:189-194` — the `break` that drops trailing frames when `< LFR_M` remain.
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs:291-294` — `apply_cmvn` (silent truncation on length mismatch).
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs:346-347` — `process()` audio scaling without a finiteness check.
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs:372` — the existing `#[cfg(test)] mod tests` (extend it).
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs:460` — the incorrect "T_lfr = 2 (ceil(14/6)=2)" comment.
  - `frontend/src-tauri/src/funasr_onnx/frontend.rs` — the `hamming_window()` / `mel_filterbank()` module-level caches to mirror.
  - `frontend/src-tauri/src/funasr_onnx/model.rs:103-109` — the consumer of the fbank output (T11).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New/extended `#[cfg(test)]` tests in `frontend/src-tauri/src/funasr_onnx/frontend.rs` prove: (a) `apply_cmvn` with mismatched `means`/`inv_stds` lengths returns an error (or panics with a precise message) instead of truncating; (b) `process()` on an audio buffer containing `f32::NAN` or `f32::INFINITY` returns an error; (c) the FFT planner is constructed once — expose a test-only construction counter and assert it stays at 1 across two `compute_fbank()` calls; (d) the corrected comment states the real cause (the `break` at `start + LFR_M > padded.len()`), verified by `grep -n 'ceil(14/6)=2' frontend/src-tauri/src/funasr_onnx/frontend.rs` returning no hits.
  - `cargo test -p meetily funasr_onnx` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: two consecutive `compute_fbank()` calls produce identical output and the planner counter remains 1.
  - Failure: pass a `means` vector that is one element shorter than the feature frame and assert an error is returned; if it silently truncates, the validation is missing. Also pass `f32::NAN` audio and assert an error, not a NaN feature matrix.
  Evidence: `.omo/evidence/task-13-fork-review-fixes.txt`
  Commit: Y | `perf(funasr): cache FFT planner and validate feature inputs`

- [x] 14. `frontend/src-tauri/src/funasr_onnx/commands.rs` — add request timeouts, per-download cancellation, atomic model writes, and progress guards (9 findings)
  What to do / Must NOT do:
  1. **Unnecessary derive (low/style, lines 11-12).** Remove `#[derive(Clone)]` from `DownloadProgress` if it is never cloned (grep for `DownloadProgress` uses). If it is cloned somewhere, keep the derive and record that in the evidence file.
  2. **reqwest timeouts (high/bug, lines 37-40).** The client has no timeouts ⇒ an indefinite hang. Add `.connect_timeout(Duration::from_secs(30))` and `.timeout(Duration::from_secs(120))` to the client builder. Do **not** add a new dependency — `reqwest` is already a dependency; the timeout builder methods come with it.
  3. **Windows removal failure (medium/security, lines 70-73).** On `CANCEL_DOWNLOAD`, the blocking `fs::remove_file` runs while the tokio `File` handle is still open ⇒ removal fails on Windows. Explicitly `drop(file)` before removal and use `tokio::fs` (not the blocking `std::fs`).
  4. **NaN progress (medium/none, lines 89-93).** `content_length()` returning `None` ⇒ `total = 0` ⇒ `0.0/0.0 = NaN`; `.min(100.0)` does **not** clamp NaN, so NaN propagates into the progress events. Guard the division (treat an unknown total as indeterminate) and clamp the result to `0.0..=100.0` with an explicit `if value.is_finite()` check.
  5. **Static cancel flag (medium/bug, lines 142-144).** The global `CANCEL_DOWNLOAD: AtomicBool` is reset at download start (losing a previous cancel) and a second concurrent download cancels the first. Replace with a **per-download** `Arc<AtomicBool>` handle held in a registry (e.g. `static DOWNLOADS: Mutex<HashMap<String, Arc<AtomicBool>>>`). **The exact command contract T30 consumes — fix it here, do not improvise:**
     - `funasr_download_model` **returns the download id** (a `String`; e.g. `format!("funasr-{}", Utc::now().timestamp_millis())` or an incrementing counter) and registers the handle under that id before streaming starts.
     - `funasr_cancel_download` takes an **OPTIONAL** id — Rust signature `pub async fn funasr_cancel_download(download_id: Option<String>) -> Result<(), String>`: `Some(id)` cancels **that** download and removes it from the registry; `None` cancels the **most recent still-active** download (this preserves the existing single-download UI contract, because `FunasrModelManager.tsx` — OUT OF SCOPE — calls the cancel command with no argument). Removing an unknown id returns `Ok(())` (idempotent), not an error.
     - Tauri v2 maps JS camelCase arguments to Rust snake_case parameters automatically, so the frontend sends `{ downloadId: … }` for the Rust parameter `download_id`. **The command NAMES and their registration in `lib.rs` do not change** — only the parameter list of `funasr_cancel_download` and the return type of `funasr_download_model` change, and Tauri v2's `generate_handler!` needs no registration edit for that.
     - Record the final signatures verbatim in the evidence file under `COMMAND CONTRACT (consumed by T30)`.
  6. **Partial model on error (medium/maintainability, lines 153-155).** An error during config.yaml / token extraction leaves a partial `model.onnx` / `am.mvn`, and the next run's "already downloaded" check passes with a corrupt model. Download to a temp path and **atomically rename** on success, and/or clean up partials on the error path; add an integrity check.
  7. **Hardcoded sizes/weights (low/maintainability, lines 174-183).** `700_000_000` / `50_000` / `60_000` and the weights `0.95` / `0.025` / `0.02` must be derived from `content_length()` or turned into documented named constants. Prefer named constants with a comment stating the provenance of each value.
  8. **config.yaml single point of failure (medium/maintainability, lines 215-233).** `config.yaml` is deleted right after the token extraction and orphaned files are left if interrupted. Make the sequence robust: extract tokens first, then delete `config.yaml`, and clean up leftovers on failure.
  9. **Non-string YAML tokens (high/bug, lines 221-226).** `.map(|v| v.as_str().unwrap_or(""))` silently converts non-string YAML entries (e.g. floats) to `""`. Validate and return an error naming the offending entry.

  **Must NOT do:** do **not** change the Tauri command names (`funasr_download_model`, `funasr_cancel_download` — registered at `lib.rs:917-918`); do **not** change the download URLs; do **not** change the model file names/paths (`model.onnx`, `am.mvn`, the token list, `config.yaml`); do **not** add a dependency; do **not** change the progress event names or payload shape beyond what the frontend contract requires — if the per-download handle changes the command signature, T30 (`frontend/src/lib/funasr.ts`) must be updated consistently (T30 and T14 are in different waves: land T14's signature change and record it in the evidence file; T30 consumes it).
  Parallelization: Wave 2 | Blocked by: T1, T9, T10 | Blocks: T30
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:11-12` — the `DownloadProgress` struct and its `#[derive(Clone)]`.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:37-40` — the `reqwest` client builder (no timeouts).
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:70-73` — the cancel path (`fs::remove_file` + open `File` handle).
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:89-93` — the progress calculation that can produce NaN.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:142-144` — a **use** of the global `CANCEL_DOWNLOAD` flag (the `CANCEL_DOWNLOAD.store(...)` reset at download start), NOT its declaration; the `static CANCEL_DOWNLOAD: AtomicBool` declaration lives elsewhere in the file — locate it with `grep -n 'CANCEL_DOWNLOAD' frontend/src-tauri/src/funasr_onnx/commands.rs` and treat the declaration plus every use as this one finding.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:251-255` — the `funasr_cancel_download` command (`251: #[tauri::command]`, `252: pub async fn funasr_cancel_download() -> Result<(), String> {`, `253: CANCEL_DOWNLOAD.store(true, Ordering::Relaxed);`) — the cancel path that item 5 converts to a per-download handle.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:153-155` — the **"already downloaded" early-return branch** (it skips the download when the model already exists); it is NOT the partial-model-on-error path. The partial-model path is the code that writes `model.onnx` / `am.mvn` and extracts the token list from `config.yaml` — locate it by those quoted constructs and treat the early-return branch as a second reference for this same finding.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:174-183` — the hardcoded sizes and progress weights.
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:215-233` — the `config.yaml` lifecycle and the token-list extraction.
  - `frontend/src-tauri/src/lib.rs:917-918` — the Tauri command registration for `funasr_download_model` / `funasr_cancel_download` (verified: L917 `funasr_onnx::commands::funasr_download_model,` L918 `funasr_onnx::commands::funasr_cancel_download,`).
  - `frontend/src/lib/funasr.ts:12-18` and `:16-18` — the frontend caller that must be kept in sync (T30).
  - `frontend/src-tauri/src/funasr_onnx/engine.rs` — the models-directory root the download targets (T10).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `frontend/src-tauri/src/funasr_onnx/commands.rs` prove: (a) the progress calculation with `content_length() == None` yields a **finite** value in `0.0..=100.0` (extract the calculation into a pure `fn progress_percent(downloaded: u64, total: Option<u64>) -> f64` so it is testable without network); (b) a per-download cancel handle cancels only its own download and does not affect a second concurrent download (two handles, cancel one, assert the other is unaffected); (c) token extraction over a YAML fixture containing a non-string entry returns `Err` naming the entry; (d) the token extraction over a valid YAML fixture returns the expected token list.
  - `grep -n 'connect_timeout\|\.timeout(' frontend/src-tauri/src/funasr_onnx/commands.rs` shows both timeouts present.
  - `grep -n 'CANCEL_DOWNLOAD' frontend/src-tauri/src/funasr_onnx/commands.rs` returns no hits (the global flag is gone).
  - `grep -n 'fn funasr_download_model' frontend/src-tauri/src/funasr_onnx/commands.rs` shows a `String` (or `Result<String, …>`) return type and `grep -n 'fn funasr_cancel_download' frontend/src-tauri/src/funasr_onnx/commands.rs` shows an `Option<String>` id parameter — the exact contract T30 consumes.
  - `cargo test -p meetily funasr_onnx` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: the pure progress function returns `50.0` for `(50, Some(100))` and a finite value for `(50, None)`; the cancel test shows isolated cancellation.
  - Failure: a YAML fixture `token_list: [1.5, "a"]` must produce `Err`; if it produces a token list containing `""`, the validation is missing.
  Evidence: `.omo/evidence/task-14-fork-review-fixes.txt`
  Commit: Y | `fix(funasr): add download timeouts, per-download cancel, and atomic model writes`

- [x] 15. `frontend/src-tauri/src/sherpa_onnx/mod.rs` — structured error variants, preserved `io::ErrorKind`, modern test formatting (3 findings)
  What to do / Must NOT do:
  1. **Raw-String error variants (low/maintainability, lines 10-21).** `SherpaOnnxError` carries `SherpaOnnx(String)`, `ModelNotFound(String)`, `Io(String)`, `InvalidModelDir(String)`, `EmptyAudio` — every payload is an opaque string, so callers cannot match on the failure kind. Replace the string payloads with structured fields: `SherpaOnnx { message: String }`, `ModelNotFound { path: PathBuf }`, `Io { path: Option<PathBuf>, kind: std::io::ErrorKind, message: String }`, `InvalidModelDir { path: PathBuf, reason: String }`, `EmptyAudio`. Keep the variant NAMES identical so every existing `match` arm and `matches!` call still compiles; only the payload types change. Update every construction site (this file, `sherpa_onnx/engine.rs`, `sherpa_onnx/model.rs`, `sherpa_onnx/commands.rs`, `audio/transcription/sherpa_onnx_provider.rs`) and every `Display` arm to render the new fields. Add `use std::path::PathBuf;` if not present.
  2. **`From<std::io::Error>` loses the `ErrorKind` (low/maintainability, lines 37-41).** The current impl does `Io(e.to_string())`, discarding `e.kind()`. Keep the `ErrorKind` in the new `Io` variant and render it in `Display` (e.g. `io error (NotFound): ...`). Because the `From` impl cannot know the path, `path` is `None` there; the explicit constructors at the call sites fill it in.
  3. **Tests use `format!("{}", err)` (low/style, lines 55, 62, 69, 76, 84).** Replace every `format!("{}", err)` in this file's `#[cfg(test)] mod tests` with `err.to_string()` (or `format!("{err}")`). This is a pure idiom change — the asserted substrings must stay identical, otherwise the test loses meaning.
  **Must NOT do:** do **not** rename or remove any error variant; do **not** change the `Display` wording of existing variants beyond what the new structured fields require (the existing tests assert on substrings — keep those substrings present); do **not** add a dependency (use `std::io::ErrorKind`); do **not** change `EmptyAudio` (it carries no payload); do **not** touch `sherpa_onnx/model.rs` beyond adapting the construction sites you must adapt (T-file ownership: this todo owns `sherpa_onnx/mod.rs`; edits to other files are only the mechanical call-site updates caused by this change and MUST be listed in the evidence file). If adapting a call site would change another file's behaviour, stop and record it in the evidence file instead.
  Parallelization: Wave 3 | Blocked by: T1 | Blocks: T16, T17
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:1-4` — the module doc (GPU claim is stale; T16 fixes the doc there, not here).
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:6` — `use std::fmt;`.
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:9-21` — `#[derive(Debug, Clone)] pub enum SherpaOnnxError { SherpaOnnx(String), ModelNotFound(String), Io(String), InvalidModelDir(String), EmptyAudio }`.
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:23-32` — the hand-written `Display` impl that must render the new fields.
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:35` — `impl std::error::Error for SherpaOnnxError {}`.
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:37-41` — `From<std::io::Error>` → `Io(e.to_string())`.
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:44-46` — `pub mod engine; pub mod model; pub mod commands;`.
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:48-93` — the `#[cfg(test)] mod tests` block: `test_error_empty_audio` (L53, `format!` at L55), `test_error_model_not_found` (L59, L62), `test_error_sherpa_onnx` (L66, L69), `test_error_invalid_model_dir` (L73, L76), `test_error_io_conversion` (L80, L84), `test_error_is_std_error` (L88).
  - Construction sites that must be adapted: `frontend/src-tauri/src/sherpa_onnx/engine.rs:33-36` (`ModelNotFound`/`InvalidModelDir`), `frontend/src-tauri/src/sherpa_onnx/engine.rs:52-58` (`SherpaOnnx`), `frontend/src-tauri/src/sherpa_onnx/engine.rs:72-74` (`EmptyAudio`), `frontend/src-tauri/src/sherpa_onnx/engine.rs:80-82` (`SherpaOnnx`), `frontend/src-tauri/src/sherpa_onnx/model.rs` (all `SherpaOnnxError` constructions), `frontend/src-tauri/src/sherpa_onnx/commands.rs:61-70` (validation errors), `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:13-17` (error propagation only).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with no new warnings.
  - New/updated `#[cfg(test)]` tests in `frontend/src-tauri/src/sherpa_onnx/mod.rs` prove: (a) `SherpaOnnxError::Io` constructed from a `std::io::Error` with `ErrorKind::NotFound` exposes that kind (e.g. a `kind()` accessor or a `Display` string containing `NotFound`); (b) `ModelNotFound { path }` displays the path; (c) the pre-existing asserted substrings (e.g. `tokens.txt`, `.onnx`, `empty audio`) are STILL present in the rendered `Display` output; (d) no `format!("{}",` remains in this file (`grep -n 'format!("{}",' frontend/src-tauri/src/sherpa_onnx/mod.rs` returns no hits).
  - `cargo test -p meetily sherpa_onnx` passes with no new failures.
  - `grep -rn 'SherpaOnnxError::' frontend/src-tauri/src` shows every construction site updated to the new payload shapes.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily sherpa_onnx::tests` — all six existing error tests pass with the same asserted substrings, plus the new `ErrorKind` test.
  - Failure: an `io::Error` of kind `PermissionDenied` must NOT display as `NotFound` — if the kind is lost, the structured-variant work did not happen.
  Evidence: `.omo/evidence/task-15-fork-review-fixes.txt`
  Commit: Y | `refactor(sherpa): structure error variants and preserve io error kinds`

- [x] 16. `frontend/src-tauri/src/sherpa_onnx/engine.rs` — explicit recognizer-create error, correct GPU doc, reusable stream, checked sample-rate cast (4 findings)
  What to do / Must NOT do:
  1. **Ambiguous `create()` failure (medium/maintainability, lines 52-58).** `OfflineRecognizer::create(&config)` returns an `Option`; the `ok_or_else` collapses every failure into one generic message, so the user cannot tell a corrupt ONNX file from a missing token file from an unsupported provider. Replace the `Option` with the crate's error-returning API if the installed `sherpa-onnx` version exposes one (check the vendored source under the cargo registry / `~/.cargo/registry/src/**/sherpa-onnx-*/src/lib.rs`); if it only exposes `Option`, keep `Option` but enrich the message with the concrete inputs used: the resolved model path, the `tokens.txt` path, the configured provider (`cpu`), and the `num_threads`, e.g. `SherpaOnnxError::SherpaOnnx { message: format!("failed to create OfflineRecognizer (model={model_path}, tokens={tokens_path}, provider=cpu): recognizer creation returned None — model files may be invalid or incompatible") }`. The message MUST name `model_path` and `tokens_path`. Use the post-T15 structured `SherpaOnnx { message }` form.
  2. **Stale GPU documentation (high/documentation, lines 1-4 and 49).** The module doc claims "GPU acceleration is auto-detected at runtime by sherpa-onnx", but line 49 hard-forces `provider = Some("cpu".into())`. Fix the DOC, not the provider: rewrite the module doc to state that this build runs the SenseVoice recognizer on the **CPU provider** and that GPU execution is not enabled here. Do **not** switch the provider to a GPU one (that would be a behaviour change and is out of scope). Add the same correction to `frontend/src-tauri/src/sherpa_onnx/mod.rs:1-4` if that doc repeats the claim — if you touch that file, T15 owns it, so instead record the required wording in the evidence file and let T15's file be corrected only if T15 has not already run; if T15 has already run, apply the same one-line doc fix here and note it. Use `frontend/src-tauri/src/lib.rs:917-918` everywhere as the reference for the exit-cleanup pattern if needed.
  3. **New `OfflineStream` per call (medium/performance, lines 76-82).** `transcribe_audio` creates a fresh stream on every call, so per-call recognizer state is lost and the allocation cost repeats. Introduce a reusable stream wrapper: hold the stream inside the engine (MANDATORY: `std::sync::Mutex<OfflineStream>` — a `RefCell` is NOT `Sync` and would make T24's `Send + Sync` assertion impossible, so `RefCell` is FORBIDDEN here) and `reset()`/flush it between calls so consecutive transcriptions do not bleed into each other. If the `sherpa-onnx` API exposes `stream.reset()`, call it before every use; if it only exposes `input_finished()` + `get_result()`, flush the stream after each call so the next call starts clean. Keep `transcribe_audio`'s public signature unchanged. For a Tauri async app, use `std::sync::Mutex` / `parking_lot::Mutex` (not `RefCell`) to satisfy `Send + Sync`.
  4. **Unchecked `as i32` sample-rate cast (medium/security, lines 76-77).** `stream.accept_waveform(sample_rate as i32, samples)` truncates a `u32` above `i32::MAX` into a negative rate. Guard it with `i32::try_from(sample_rate)` and return a dedicated error (add `SherpaOnnxError::InvalidSampleRate { sample_rate: u32 }` — T15's variant set already exists; extend it in the same commit as T15's shape, or reuse `SherpaOnnx { message }` if T15 has not landed) on failure. Also reject `sample_rate == 0` explicitly. Use occurrence count (`grep -o ... | wc -l` == 2) where relevant.
  5. **Missing `current_model()` accessor (blocks T24).** `SherpaOnnxEngine` has NO way to report which model is loaded, so `audio/transcription/sherpa_onnx_provider.rs` hardcodes `Some("sensevoice".to_string())` and `is_model_loaded()` returns the literal `true`. **This todo adds `pub fn current_model(&self) -> Option<String>` to `SherpaOnnxEngine`**, returning the RESOLVED model directory name (the directory name passed to `new`, e.g. `model_dir.file_name().and_then(|n| n.to_str()).map(str::to_string)`), stored in a new field on the engine so it survives the `Debug` impl update at lines 20-24. T24 then consumes it (`self.engine.current_model().is_some()` for `is_model_loaded()`, `self.engine.current_model()` for `get_current_model()`). T24 is BLOCKED on this todo — if `current_model()` is missing, T24 must report BLOCKED, never re-add the hardcoded literal.
  **Must NOT do:** do **not** change the provider from `cpu` to a GPU provider; do **not** change `num_threads = 4`; do **not** change the public signature of `transcribe_audio`; do **not** change the model-file names or the `tokens.txt` lookup; do **not** add a dependency; do **not** delete the `#[ignore = "requires real ONNX model files …"]` test at lines 137-159 — it must keep existing (and keep its `#[ignore]` reason) because sherpa-onnx may abort on garbage input; do **not** edit `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs` (T24's file).
  Parallelization: Wave 3 | Blocked by: T1, T15 | Blocks: T19, T24
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:1-4` — the stale module doc.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:8-10` — `use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig};`.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:16-18` — `pub struct SherpaOnnxEngine { recognizer: OfflineRecognizer }` (the reusable-stream holder must live here).
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:20-24` — the manual `Debug` impl (`finish_non_exhaustive`) — update it if a new field is added.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:31-36` — `pub fn new(model_dir: &Path) -> Result<Self, SherpaOnnxError>`, `model::validate_model_dir(model_dir)?`, `model::find_onnx_model_file(model_dir)?`.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:39-49` — the config build; **line 49 `provider = Some("cpu".into())`** is the hard-forced CPU provider.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:52-58` — `OfflineRecognizer::create(&config).ok_or_else(...)`.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:67-85` — `transcribe_audio`: L72-74 empty→`EmptyAudio`; **L76 `let stream = self.recognizer.create_stream();`**; **L77 `stream.accept_waveform(sample_rate as i32, samples);`**; L78 `self.recognizer.decode(&stream);`; L80-82 `stream.get_result().ok_or_else(...)`.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:88-171` — the test module: L93 `test_new_with_nonexistent_dir`, L103 `test_new_with_invalid_dir_missing_tokens`, L120 `test_new_with_invalid_dir_missing_onnx`, L137-159 `#[ignore]` `test_new_with_valid_looking_dir_but_fake_model`, L161 `test_transcribe_empty_audio_returns_error`.
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:44-51` — the only caller of `transcribe_audio` (it must keep compiling).
  - The `sherpa-onnx` crate API surface: read `~/.cargo/registry/src/*/sherpa-onnx-*/src/lib.rs` (or `frontend/src-tauri/Cargo.lock`'s pinned version) to confirm whether `OfflineRecognizer::create` has a `Result` variant and whether `OfflineStream` exposes `reset()`.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `frontend/src-tauri/src/sherpa_onnx/engine.rs` prove: (a) `transcribe_audio(&[], 16000)` still returns `EmptyAudio`; (b) a sample rate of `u32::MAX` returns the invalid-sample-rate error instead of proceeding (assert on the error kind/message, never on a panic); (c) sample rate `0` is rejected; (d) the recognizer-create failure message (or error) contains the model path and the tokens path.
  - `grep -n 'provider = Some("cpu")' frontend/src-tauri/src/sherpa_onnx/engine.rs` still matches (the provider is intentionally unchanged) AND the module doc no longer claims runtime GPU auto-detection (`grep -n -i 'auto-detect' frontend/src-tauri/src/sherpa_onnx/engine.rs` returns no hits).
  - `grep -n 'as i32' frontend/src-tauri/src/sherpa_onnx/engine.rs` returns no hits (the unchecked cast is gone).
  - `grep -n 'pub fn current_model' frontend/src-tauri/src/sherpa_onnx/engine.rs` returns a hit (the accessor T24 depends on exists), and a `#[cfg(test)]` test proves `current_model()` returns `Some(<resolved dir name>)` for an engine built from a temp model dir and `None` when no model name can be resolved.
  - `cargo test -p meetily sherpa_onnx` passes with no new failures (the `#[ignore]`d test stays `#[ignore]`d).
  QA scenarios (name the exact tool + invocation):
  - Happy: the reusable stream is created ONCE per engine (assert the stream is reused across two consecutive `transcribe_audio` calls — e.g. by asserting the engine holds exactly one stream, or by counting `create_stream()` calls behind a test-only counter) and each call still returns a result.
  - Failure: `transcribe_audio(&[0.0f32; 100], u32::MAX)` must return an `InvalidSampleRate` error; if it panics or silently wraps to a negative rate, the guard is missing.
  Evidence: `.omo/evidence/task-16-fork-review-fixes.txt`
  Commit: Y | `fix(sherpa): enrich recognizer errors, reuse the stream, and guard the sample rate`

- [x] 17. `frontend/src-tauri/src/sherpa_onnx/commands.rs` — non-blocking scan, path-traversal guard, real validation errors, honest test (6 findings)
  What to do / Must NOT do:
  1. **Blocking filesystem walk in an async command (high/performance, line 40).** `sherpa_onnx_scan_models` is `async` but calls the synchronous `model::discover_models(&sherpa_dir)` (a recursive directory walk) with no `await` before it, so the walk runs on the async runtime thread. Move the entire scan (discovery + size computation) into `tokio::task::spawn_blocking`, e.g. `let sherpa_dir = ...; let models = spawn_blocking(move || scan_models_blocking(&sherpa_dir)).await.map_err(|e| e.to_string())?;`. Also move the `crate::directory_size_mb(&m.path)` call (line 49) into that same blocking closure.
  2. **Double allocation on the path string (low/performance, line 47).** `m.path.to_string_lossy().to_string()` allocates twice; use `m.path.to_string_lossy().into_owned()`. Use `152-169` for the directory_size_mb reference.
  3. **Unhandled size computation (high/other, line 49).** The review claims `crate::directory_size_mb` "panics if file deleted". Verified: `frontend/src-tauri/src/lib.rs:152-169` does NOT panic — it swallows the `read_dir` error and returns `0.0`, so the real defect is (a) a blocking walk on the async thread (fixed by item 1) and (b) a vanished path is reported as `size_mb: 0.0` with no signal. Fix by computing the size inside the `spawn_blocking` closure, checking `path.exists()` before the walk, and returning `0.0` **with an explicit `warn!`** when the path has vanished. Record in the evidence file that the "panics" premise was verified false against `lib.rs:152-169`.
 4. **Path traversal via the RPC surface (medium/security, lines 58-60).** `sherpa_onnx_validate_model(model_path: String)` accepts ANY path from the frontend and validates it. Confine it: resolve the models root (`app_data_dir/models/sherpa-onnx`), canonicalize the requested path, and reject anything that is not inside the canonicalized models root (compare with `Path::starts_with` after canonicalization; also reject `..` components before canonicalization). Return `Err` with a clear message for a traversal attempt. The command needs the `AppHandle` to compute the root — add it to the signature; the command is registered in `frontend/src-tauri/src/lib.rs`, so verify the registration still resolves (Tauri injects `AppHandle` automatically for `AppHandle<R>` parameters — keep the existing generic form). T29 owns `lib.rs` call-site integration if the signature changes.
  5. **Validation errors discarded (medium/other, lines 61-70).** The `Err(e)` arm logs a warning and returns `Ok(false)`, so the caller cannot distinguish "not a valid model" from "I could not check". Change the command to return `Result<bool, String>` semantics that preserve the error: on a validation failure return `Err(<message including the underlying error>)`; keep `Ok(false)` only for a legitimate "not a model" verdict if the underlying validator distinguishes that case — otherwise return `Err`.
  6. **Test that never calls the function under test (low/other, lines 108-124).** `test_initialize_models_directory_creates` does its own `std::fs::create_dir_all` and never calls `initialize_models_directory` (an `AppHandle` cannot be constructed in a unit test). Fix by extracting the pure filesystem logic of `initialize_models_directory` into a testable function that takes a `&Path` (e.g. `pub(crate) fn ensure_models_dir(root: &Path) -> std::io::Result<()>`), have `initialize_models_directory` delegate to it, and rewrite the test to call the extracted function with a temp dir. If you prefer, mark the test `#[ignore = "requires a Tauri AppHandle — covered by the extracted path-based helper"]`; the extracted-helper route is strongly preferred.
  **Must NOT do:** do **not** change the command NAMES (`sherpa_onnx_scan_models`, `sherpa_onnx_validate_model`) or the `SherpaOnnxModelEntry` field names/shape — the frontend depends on them (see `frontend/src/hooks/useTranscriptionModels.ts`); do **not** change the models directory layout (`app_data_dir/models/sherpa-onnx`); do **not** add a dependency; do **not** edit `frontend/src-tauri/src/lib.rs` in this todo — adding `AppHandle<R>` to a `#[tauri::command]` needs NO registration change because Tauri v2's `generate_handler!` resolves `AppHandle` automatically, so the required lib.rs edit is **zero**; if you are convinced a registration change is unavoidable, write it into the evidence file under `REQUIRED lib.rs CHANGE (applied by T29)` and leave the edit to T29; do **not** weaken the path-traversal guard to a simple string prefix check (use canonicalized path comparison).
  Parallelization: Wave 3 | Blocked by: T1, T15 | Blocks: T29
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:6-10` — imports (`crate::sherpa_onnx::model`, `log::{info,warn}`, `serde`, `std::path::Path`, `tauri::{AppHandle, Manager, Runtime}`).
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:13-19` — `pub struct SherpaOnnxModelEntry { name, path, valid, size_mb }` (Serialize + Deserialize).
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:22-55` — `#[tauri::command] pub async fn sherpa_onnx_scan_models<R: Runtime>(app: AppHandle<R>) -> Result<Vec<SherpaOnnxModelEntry>, String>`; L26-29 `app.path().app_data_dir()`; L30 `sherpa_dir = app_data_dir.join("models").join("sherpa-onnx")`; L32-38 missing dir → `Ok(Vec::new())`; **L40 `let models = model::discover_models(&sherpa_dir);`**; L44 `valid = matches!(m.status, model::ModelStatus::Ready)`; **L47 `path: m.path.to_string_lossy().to_string()`**; **L49 `size_mb: crate::directory_size_mb(&m.path)`**.
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:58-71` — `#[tauri::command] pub async fn sherpa_onnx_validate_model(model_path: String) -> Result<bool, String>`; **L59-60 `let path = Path::new(&model_path);` with no containment check**; L61-70 `Ok(()) => Ok(true)` / `Err(e) => { warn!(...); Ok(false) }`.
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:73-88` — `pub fn initialize_models_directory(app: &AppHandle)`; L75 `if let Ok(app_data_dir) = app.path().app_data_dir()`; L77 `if !models_dir.exists()`; L78 `create_dir_all` with the error swallowed by `warn!`.
  - `frontend/src-tauri/src/sherpa_onnx/commands.rs:90-125` — the test module: L95 `test_model_entry_serialization`; **L108-124 `test_initialize_models_directory_creates` (never calls the function under test)**.
  - `frontend/src-tauri/src/lib.rs:152-169` — `pub(crate) fn directory_size_mb(path: &std::path::Path) -> f64` (verified: does not panic; returns `0.0` on an unreadable path).
  - `frontend/src-tauri/src/lib.rs` command registration for `sherpa_onnx_scan_models` / `sherpa_onnx_validate_model` — grep `sherpa_onnx_` in `frontend/src-tauri/src/lib.rs`.
  - `frontend/src/hooks/useTranscriptionModels.ts` — the frontend consumer of the scan/validate commands (T31).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `frontend/src-tauri/src/sherpa_onnx/commands.rs` prove: (a) a path-traversal attempt (`app_data/models/sherpa-onnx/../../etc`) is REJECTED with an `Err` (or `Ok(false)`) naming the traversal, and never validated; (b) a path inside the models root is validated normally; (c) the extracted `ensure_models_dir(&Path)` helper creates the directory on a fresh temp path and returns `Ok` when it already exists; (d) `SherpaOnnxModelEntry` still serializes to the same JSON field names (`name`, `path`, `valid`, `size_mb`).
  - `grep -n 'spawn_blocking' frontend/src-tauri/src/sherpa_onnx/commands.rs` shows the scan moved off the async thread.
  - `grep -n 'to_string_lossy().to_string()' frontend/src-tauri/src/sherpa_onnx/commands.rs` returns no hits.
  - `cargo test -p meetily sherpa_onnx` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `sherpa_onnx_validate_model` on a path inside the models root returns `Ok(true)` for a valid model dir and the scan returns entries with correct `size_mb` values.
  - Failure: `sherpa_onnx_validate_model("../../etc/passwd")` (and an absolute path outside the root) must NOT return `Ok(true)`; if it does, the containment check is missing.
  Evidence: `.omo/evidence/task-17-fork-review-fixes.txt`
  Commit: Y | `fix(sherpa): move the model scan off the async thread and confine validation to the models dir`

- [x] 18. `frontend/src-tauri/src/audio/transcription/mod.rs` — narrow the public surface and document the module (4 findings)
  What to do / Must NOT do:
  1. **Concrete provider types re-exported (medium/maintainability, lines 18, 19, 21, 22).** `pub use funasr_provider::FunasrProvider; pub use parakeet_provider::ParakeetProvider; pub use sherpa_onnx_provider::SherpaOnnxProvider; pub use whisper_provider::WhisperProvider;` widen the crate's public API to concrete implementations. Remove those four re-exports so consumers depend on the `TranscriptionProvider` trait (already re-exported at line 20). Verify every consumer still compiles: `TranscriptionEngine::Provider(Arc<dyn TranscriptionProvider>)` in `audio/transcription/engine.rs:15-19` needs no concrete type.
  2. **Module documentation is minimal (low/documentation, lines 1-3).** Expand the module doc to describe: the module's purpose (provider abstraction + engine lifecycle + transcription worker), the lifecycle (`model load → engine init → worker poll → transcript emit`), the thread-safety model (providers are used behind `Arc<dyn TranscriptionProvider>`; the engine init functions take `&AppHandle` and are async), and the lazy-initialization/concurrency semantics (engine caches are process-global; the init functions are idempotent and must be awaited by the caller before use).
  3. **All submodules are `pub` (low/maintainability, lines 5-11).** `pub mod engine; pub mod funasr_provider; pub mod parakeet_provider; pub mod provider; pub mod sherpa_onnx_provider; pub mod whisper_provider; pub mod worker;` — make each `pub(crate) mod` (or private where nothing outside the module tree needs it). Every external user of these paths must be switched to the re-exports at lines 13-23. Grep the crate for `transcription::engine::`, `transcription::provider::`, `transcription::worker::`, `transcription::funasr_provider::`, `transcription::sherpa_onnx_provider::`, `transcription::parakeet_provider::`, `transcription::whisper_provider::` and route each through the module's own re-exports (adding a `pub(crate) use` line if an item is needed by a sibling module).
  4. **Engine functions re-exported publicly (low/maintainability, lines 14-17).** `pub use engine::{get_or_init_transcription_engine, get_or_init_whisper, validate_transcription_model_ready, TranscriptionEngine};` — narrow the three free functions to `pub(crate) use` and keep `TranscriptionEngine` as `pub(crate) use` as well unless a genuinely external crate needs it (the crate is a lib crate `app_lib`, so `pub(crate)` is the safe default).
  **Must NOT do:** do **not** change any function SIGNATURE; do **not** rename the re-exported items; do **not** delete a re-export that is actually used elsewhere in the crate (grep first — `get_or_init_transcription_engine` and `get_or_init_whisper` are used by `audio/transcription/worker.rs`, `parakeet_engine/commands.rs`, `whisper_engine/commands.rs`, `audio/recording_commands.rs`); do **not** change `provider::TranscriptResult` / `provider::TranscriptionError` / `provider::TranscriptionProvider` visibility in a way that breaks the other providers; do **not** restructure the module tree.
  Parallelization: Wave 3 | Blocked by: T1 | Blocks: T19, T23, T24, T25, T26
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/transcription/mod.rs:1-3` — the current one-line doc.
  - `frontend/src-tauri/src/audio/transcription/mod.rs:5-11` — the seven `pub mod` declarations.
  - `frontend/src-tauri/src/audio/transcription/mod.rs:13-23` — the re-export block: L14-17 engine functions + `TranscriptionEngine`, L18 `FunasrProvider`, L19 `ParakeetProvider`, L20 `provider::{TranscriptResult, TranscriptionError, TranscriptionProvider}`, L21 `SherpaOnnxProvider`, L22 `WhisperProvider`, L23 `worker::{reset_speech_detected_flag, start_transcription_task, TranscriptUpdate}`.
  - Known consumers of the module path: `frontend/src-tauri/src/audio/transcription/engine.rs`, `.../worker.rs`, `frontend/src-tauri/src/parakeet_engine/commands.rs`, `frontend/src-tauri/src/whisper_engine/commands.rs`, `frontend/src-tauri/src/audio/recording_commands.rs`, `frontend/src-tauri/src/audio/retranscription.rs`, `frontend/src-tauri/src/audio/import.rs`.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with no new warnings.
  - `grep -n '^pub mod\|^pub use' frontend/src-tauri/src/audio/transcription/mod.rs` shows NO `pub mod` lines and NO `pub use` of a concrete provider type; every visibility is `pub(crate)`.
  - `cargo test -p meetily` passes with no new failures.
  - A new doc-comment assertion is not needed; instead the module doc must mention all four required topics — verify with `grep -n -i 'lifecycle\|thread-safe\|lazy' frontend/src-tauri/src/audio/transcription/mod.rs`.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo check -p meetily` proves every consumer of the narrowed API still compiles — in particular the four engine-init consumers named above.
  - Failure: `grep -rn 'transcription::funasr_provider::\|transcription::sherpa_onnx_provider::\|transcription::parakeet_provider::\|transcription::whisper_provider::' frontend/src-tauri/src` must return NO hits after the change (each consumer now goes through the trait re-export); any hit is a broken narrowing.
  Evidence: `.omo/evidence/task-18-fork-review-fixes.txt`
  Commit: Y | `refactor(transcription): narrow the module surface to the provider trait`

- [x] 19. `frontend/src-tauri/src/audio/transcription/engine.rs` — shared config helper, poison-safe locks, real model validation (7 findings)
  What to do / Must NOT do:
  1. **Poisoned-lock panics (medium/maintainability, lines 256-260, 330-335, 414-422).** `PARAKEET_ENGINE.lock().unwrap()` and `WHISPER_ENGINE.lock().unwrap()` propagate a `PoisonError` panic when the guard was poisoned by a panic in another thread. Replace every `.lock().unwrap()` on these two statics with poison-tolerant handling: `match guard { Ok(g) => g, Err(poisoned) => poisoned.into_inner() }` (the inner value is still valid; only the "poison" flag is set) and `log::warn!` once about the recovered poison. Use a small local helper (e.g. `fn lock_unpoison<T>(m: &Mutex<T>) -> MutexGuard<'_, T>`) so the three sites stay consistent.
  2. **Config fetch-and-fallback duplicated 3× (low/maintainability, lines 59-89, 217-247, and the two fetches at 346-372 and 425-459).** The same "call `api_get_transcript_config`; on `Ok(Some(cfg))` use it; on `Ok(None)` fall back to provider `parakeet` + `DEFAULT_PARAKEET_MODEL`; on `Err` warn and use the same default" block is copy-pasted. Extract ONE helper (e.g. `async fn resolve_transcription_config<R: Runtime>(app: &AppHandle<R>) -> TranscriptConfig`) that performs the fetch and applies the fallback exactly once, and route `validate_transcription_model_ready`, `get_or_init_transcription_engine`, and `get_or_init_whisper` through it. The fallback semantics must stay byte-identical to today's behaviour.
  3. **FunASR validation only checks file existence (high/bug, lines 150-169).** The `funasr` branch verifies `model_dir.join("model.onnx").exists()` and returns `Ok(())` — a corrupt or truncated model passes validation, unlike Whisper/Parakeet which load the engine. Make the branch actually load the model: construct `FunasrProvider::new(&model_dir)` (which calls `FunasrEngine::new` and therefore loads the ONNX session) and map any error into `Err(<message>)`. Keep the "model directory missing" error message for the missing-directory case. Because loading is expensive, do it on a blocking thread (`spawn_blocking`) if the constructor is CPU-bound; it is a synchronous constructor, so call it inside `tokio::task::spawn_blocking`.
  4. **Sherpa-ONNX validation is structure-only (high/bug, lines 170-198).** The `sherpa-onnx` branch checks `model_dir.exists()` and calls `validate_model_dir(&model_dir)` (structure only). Make it load the model the same way: `SherpaOnnxProvider::new(&model_dir)` inside `spawn_blocking`, mapping errors to `Err`. Keep the missing-directory error message.
  5. **Dead `model_name` parameter (low/other, lines 156-160) — VERIFY FIRST.** The review claims a `model_name` parameter is immediately reassigned at line 135, making it dead. Verified against this commit: lines 156 and 176 are LOCAL `let model_name = if config.model.is_empty() { … } else { &config.model };` bindings that are immediately USED to build `model_dir`; there is NO shadowed parameter. **Action:** grep the file for `fn .*model_name` and for any `model_name` parameter; if no such parameter exists, record the finding as **NOT REPRODUCIBLE at this commit** in the evidence file and change NO code for it. If a shadowed parameter DOES exist (for example in a helper this branch introduced), remove it and its call-site argument.
  6. **Second dead `model_name` shadowing (low/other, lines 176-184)** — same verification and same rule as item 5.
  7. **Repeated config fetch-and-fallback in `get_or_init_transcription_engine` (low/maintainability, lines 217-247).** Covered by item 2 — after the helper exists, this block must call the helper and must not contain a second copy of the fetch-and-fallback logic.
  **Must NOT do:** do **not** change the provider name strings (`localWhisper`, `parakeet`, `funasr`, `sherpa-onnx`) — they are the persisted config vocabulary; do **not** change the default provider (`parakeet`) or `DEFAULT_PARAKEET_MODEL`; do **not** change the `TranscriptionEngine` enum shape; do **not** change the public signatures of `validate_transcription_model_ready`, `get_or_init_transcription_engine`, `get_or_init_whisper`; do **not** change `FunasrProvider::new` / `SherpaOnnxProvider::new` signatures (T23/T24 own those files and must keep them stable — if you believe they must change, record it in the evidence file instead of changing them); do **not** swallow validation errors back into `Ok(())`.
  Parallelization: Wave 3 | Blocked by: T1, T16, T18 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/transcription/engine.rs:15-19` — `pub enum TranscriptionEngine { Whisper(Arc<WhisperEngine>), Parakeet(Arc<ParakeetEngine>), Provider(Arc<dyn TranscriptionProvider>) }`.
  - `frontend/src-tauri/src/audio/transcription/engine.rs:21-48` — the `impl TranscriptionEngine` (`is_model_loaded`, `get_current_model`, `provider_name`).
  - `frontend/src-tauri/src/audio/transcription/engine.rs:55-210` — `pub async fn validate_transcription_model_ready<R: Runtime>(app: &AppHandle<R>) -> Result<(), String>`; L59-89 the config fetch-and-fallback block; L92-209 the provider match: "localWhisper" L93-120, "parakeet" L121-149, **"funasr" L150-169 (L156 the `model_name` local, L157-160 the model dir, L161-166 the `model.onnx` existence check)**, **"sherpa-onnx" L170-198 (L176-180 the `model_name` local, L181-184 the model dir, L185-190 the dir check, L191-192 `validate_model_dir`)**, L199-208 the unsupported-provider error.
  - `frontend/src-tauri/src/audio/transcription/engine.rs:213-322` — `pub async fn get_or_init_transcription_engine<R: Runtime>(app: &AppHandle<R>) -> Result<TranscriptionEngine, String>`; L217-247 the duplicated config block; L250-321 the provider match: "parakeet" L251-281 (**L256-260 `PARAKEET_ENGINE.lock().unwrap()`**), "funasr" L282-296 (L288 the `model_name` local, L293 `FunasrProvider::new(&model_dir)`), "sherpa-onnx" L297-315 (L303-307 the `model_name` local, L312 `SherpaOnnxProvider::new(&model_dir)`), "localWhisper" | _ L316-320.
  - `frontend/src-tauri/src/audio/transcription/engine.rs:326-547` — `pub async fn get_or_init_whisper<R: Runtime>(app: &AppHandle<R>) -> Result<Arc<WhisperEngine>, String>`; **L330-335 `WHISPER_ENGINE.lock().unwrap()`**; L337-403 the reuse/reload logic with the config fetch at L346-372; L409 `whisper_init()`; **L414-422 `WHISPER_ENGINE.lock().unwrap()` again**; L425-459 a further config fetch; L464-467 `engine.discover_models()`; L480-488 the `model_info` lookup; L490-544 the status match.
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:13-17` — `FunasrProvider::new(model_dir: &Path) -> Result<Self, crate::funasr_onnx::FunasrError>` (loads the engine — this is the load-time validation entry point).
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:13-17` — `SherpaOnnxProvider::new(model_dir: &Path) -> Result<Self, crate::sherpa_onnx::SherpaOnnxError>`.
  - `frontend/src-tauri/src/audio/transcription/mod.rs:14-17` — the re-exports T18 narrows (keep the three functions reachable).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with no new warnings.
  - New `#[cfg(test)]` tests in `frontend/src-tauri/src/audio/transcription/engine.rs` prove: (a) the shared config helper returns the `parakeet` / `DEFAULT_PARAKEET_MODEL` default when the config fetch yields `None` and when it errors — implemented with a pure, injectable seam (e.g. the helper takes the fetched `Result<Option<TranscriptConfig>, _>` as an argument, or a trait/closure) so the test needs no `AppHandle`; (b) the helper is the ONLY place that contains the fallback — verified by `grep -n 'DEFAULT_PARAKEET_MODEL' frontend/src-tauri/src/audio/transcription/engine.rs` returning exactly 2 hits — the `const DEFAULT_PARAKEET_MODEL` definition line and ONE usage inside the shared helper (any third hit means a second copy of the fallback exists; `grep -c` counts LINES, so the old "returning 1" was unsatisfiable) — and by `grep -n 'api_get_transcript_config' frontend/src-tauri/src/audio/transcription/engine.rs` showing exactly one call site inside the helper; (c) a poison-safety unit test: lock the mutex, poison it by panicking inside the lock in a test thread, then call the `lock_unpoison` helper and assert it returns the value instead of panicking.
  - `grep -n 'lock().unwrap()' frontend/src-tauri/src/audio/transcription/engine.rs` returns NO hits.
  - `grep -n 'fn .*model_name' frontend/src-tauri/src/audio/transcription/engine.rs` output is recorded in the evidence file together with the NOT-REPRODUCIBLE verdict for findings 156-160 / 176-184 (unless a real shadowed parameter is found and removed).
  - The FunASR/Sherpa validation branches must call the provider constructor — verified by `grep -n 'FunasrProvider::new\|SherpaOnnxProvider::new' frontend/src-tauri/src/audio/transcription/engine.rs` showing hits inside `validate_transcription_model_ready`.
  - `cargo test -p meetily` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: with the fallback seam fed `Ok(None)`, the helper yields provider `parakeet` + `DEFAULT_PARAKEET_MODEL` and the validation function reports the parakeet model as the intended one.
  - Failure: with the fallback seam fed `Err(...)`, the helper must still return the default AND emit exactly one `warn!` — not two different defaults from two different code paths (the duplicated-block bug). With a poisoned `WHISPER_ENGINE` mutex, `get_or_init_whisper` must NOT panic.
  Evidence: `.omo/evidence/task-19-fork-review-fixes.txt`
  Commit: Y | `fix(transcription): share the config fallback, survive lock poisoning, and validate models by loading them`

- [x] 20. `frontend/src-tauri/src/whisper_engine/whisper_engine.rs` — consistent lock snapshot, remove no-op shadowing and dead locals (3 findings)
  What to do / Must NOT do:
  1. **RwLock read across multiple async checks (low/other, the `should_log_transcription` / `should_log_short_warning` block).** The code reads `self.last_transcription_was_short` more than once across an async boundary, so the value can change between the reads and the derived flags can be mutually inconsistent. Take ONE snapshot (`let last_was_short = *self.last_transcription_was_short.read().await;`) and derive `should_log_transcription` / `should_log_short_warning` from that single value; keep the later write (`write().await = true/false`) where it is.
  2. **No-op shadowing (low/maintainability, line 630; reported at 616-630).** `let num_segments = num_segments;` does nothing — remove the line (or the redundant binding) so the value flows directly from its source.
  3. **Unused `_start_time` / `_end_time` (low/maintainability, lines 793-794).** These locals ARE used at line 802 inside the `info!`/`debug!` format args (`_start_time as f64 / 100.0`). If the format string no longer references them, delete the bindings; if it does reference them, keep them and record in the evidence file that the finding is not reproducible (the review's premise was that they are unused). Verify by compiling — an unused variable is a warning, so `cargo check` after the change must be warning-free for this file.
  **Must NOT do:** do **not** change the logging semantics (which lines are logged, the counters, the `transcription_count % 10` / `% 20` cadence); do **not** change the transcript text that is logged; do **not** change the Whisper engine's public API; do **not** change the duration/segment thresholds.
  Parallelization: Wave 3 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:74` — `last_transcription_was_short: Arc<RwLock<bool>>` (the field behind the snapshot).
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:190` — the field's initialisation `Arc::new(RwLock::new(false))`.
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:630` — `let num_segments = num_segments;` (the no-op shadowing; the review cites the range 616-630).
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:730-731` — `let mut should_log_transcription = true; let mut should_log_short_warning = false;`.
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:734` — the first read `let last_was_short = *self.last_transcription_was_short.read().await;`.
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:738` and `:743` — the branches that set `should_log_short_warning` / `should_log_transcription`.
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:745` and `:755` — the writes `*self.last_transcription_was_short.write().await = true/false;`.
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:747` — the SECOND read `let last_was_short = *self.last_transcription_was_short.read().await;` (the inconsistent double read).
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:758, 770, 782, 823` — the consumers of `should_log_short_warning` / `should_log_transcription`.
  - `frontend/src-tauri/src/whisper_engine/whisper_engine.rs:793-794` — `let _start_time = …; let _end_time = …;` and `:802` — the format-args site that uses them.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with no new warnings (an unused local would warn — that is the check for item 3).
  - `grep -n 'last_transcription_was_short.read().await' frontend/src-tauri/src/whisper_engine/whisper_engine.rs` shows EXACTLY ONE read in the logging block.
  - `grep -n 'let num_segments = num_segments;' frontend/src-tauri/src/whisper_engine/whisper_engine.rs` returns no hits.
  - `cargo test -p meetily whisper` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: the logging block still logs the transcript every 10th transcription and after >10 s, and the short-warning branch still fires — assert this with a **deterministic, automated unit test that fails before the fix and passes after it** (red→green, per F1.3): extract the flag logic into the same pure helper the Failure scenario uses (`fn should_log_flags(last_was_short: bool) -> (bool, bool)`, extended with the transcription count if needed) and assert the expected flag pairs for representative inputs. **Do NOT assert by reading the code path** and **do NOT add a flaky timing test** — the cadence is unit-testable once the flags are a pure function of the snapshot. Record in the evidence file that the new test FAILS against the pre-fix code (the inconsistent double read) and PASSES after the fix.
  - Failure: a new unit test that snapshots the flag twice in a row (e.g. a helper `fn should_log_flags(last_was_short: bool) -> (bool, bool)` extracted from the block) must return the SAME pair for the same input; if the two flags can disagree for one input, the single-snapshot fix is wrong.
  Evidence: `.omo/evidence/task-20-fork-review-fixes.txt`
  Commit: Y | `refactor(whisper): snapshot the short-transcription flag once`

- [x] 21. `frontend/src-tauri/src/audio/onnx_provider.rs` — gate the DirectML/OpenVINO providers by OS, assert the CPU fallback, extend the GpuType coverage (7 findings)
  What to do / Must NOT do:
  1. **DirectML EP leaks into the CUDA path on every OS (high/bug, lines 83-87).** The CUDA branch pushes the DirectML provider under `#[cfg(feature = "ort-directml")]` only, so a Linux/macOS CUDA build would also register DirectML. Change it to `#[cfg(all(target_os = "windows", feature = "ort-directml"))]`, exactly like the Vulkan branch at line 93. This is a `cfg`-only change — no logic moves.
  2. **OpenVINO EP is enabled on every non-Windows OS (high/bug, lines 99-104).** `#[cfg(all(not(target_os="windows"), feature="ort-openvino"))]` compiles the OpenVINO provider on macOS and FreeBSD, where it cannot work. Change it to `#[cfg(all(target_os = "linux", feature = "ort-openvino"))]` and fix the misleading comment at line 99 (it currently says "On Linux (and other non-Windows platforms), use OpenVINO." → the correct statement is "On Linux, use OpenVINO."). Keep the DirectML branch above it for Windows.
  3. **The "`.build()` returns a `Result`" premise is FALSE — do NOT "fix" it (medium/maintainability, lines 56-64).** Verified against `ort = "2.0.0-rc.10"`: the registry at `~/.cargo/registry/src/*/ort-2.0.0-rc.10/src/execution_providers/mod.rs:331` returns the `ExecutionProviderDispatch` DIRECTLY, so `CUDAExecutionProvider::default().build()` is not a `Result`. Adding `.expect(...)`/`.unwrap()` would NOT compile. Treat this finding as future-proofing only: do **not** add any `expect`/`unwrap` around a `.build()` call. Record the verification (file + line) in the evidence file so the finding is closed as "premise verified false for ort 2.0.0-rc.10".
  4. **`test_cpu_last_in_chain` never inspects the CPU provider (high/test, lines 166-182).** The test loops over a manual `GpuType` array and asserts only `!providers.is_empty()`. Extend it to assert that the LAST element is the CPU provider, e.g. `assert!(matches!(providers.last(), Some(p) if p.name() == "CPUExecutionProvider"))` — confirm the exact accessor name against the `ort` crate (`ExecutionProviderDispatch::name()` returns `&str`), and if the API differs, assert on the `Debug`/`Display` representation of the last element instead and record the accessor used.
  5. **Manual `GpuType` list in the test (medium/test, lines 166-182).** The array `&[GpuType::None, GpuType::Metal, GpuType::Cuda, GpuType::Vulkan, GpuType::OpenCL]` silently omits any future variant. `GpuType` is declared at `frontend/src-tauri/src/audio/hardware_detector.rs:15-22` and ALREADY derives `Debug, Clone, Copy, PartialEq, Eq` — adding `EnumIter` would require editing `hardware_detector.rs`, which is **OUT OF SCOPE** for this plan. Therefore implement the fix entirely inside `audio/onnx_provider.rs`: replace the manual array with an exhaustive `match` over `GpuType` that has a wildcard arm `other => panic!("uncovered GpuType variant: {other:?}")` annotated `#[allow(unreachable_patterns)]`. This gives the same "new variant breaks the build" guarantee without touching an out-of-scope file.
  6. **Casing in a test message (low/style, line 152).** `"With ort-cuda, Cuda should have >=2 providers"` → `"With ort-cuda, CUDA should have >=2 providers"`.
  7. **Document the DirectML-as-Windows-fallback behaviour (medium/maintainability, lines 71-88).** The module doc's EP table (lines 1-19) lists `| Cuda | CUDAExecutionProvider → (DirectMLExecutionProvider) → CPU |` and `| Vulkan | DirectMLExecutionProvider (Windows) / OpenVINO (Linux) → CPU |`. After the `cfg` fixes, update the table and add a short note that DirectML is the Windows cross-platform fallback when the CUDA provider is unavailable, and that OpenVINO is Linux-only.
  **Must NOT do:** do **not** change the `ort` dependency version (keep `ort = "2.0.0-rc.10"); do **not** add `.expect()`/`.unwrap()` to any `.build()` call; do **not** edit `frontend/src-tauri/src/audio/hardware_detector.rs`; do **not** change the ORDER of the provider chain (CPU must remain LAST); do **not** remove the CPU fallback (line 109); do **not** change the `CoreML` branch's `CoreMLComputeUnits::CPUAndNeuralEngine` setting; do **not** change `get_onnx_providers`'s signature (`pub fn get_onnx_providers(gpu_type: GpuType) -> Vec<ExecutionProviderDispatch>`) — it is called from `funasr_onnx/model.rs:39` and `:180` plus the in-file unit tests at `audio/onnx_provider.rs:121, 132, 150, 176`.
  Parallelization: Wave 2 (lands FIRST in Wave 2 — T11 calls the API this todo changes) | Blocked by: T1, T2 | Blocks: T11
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/onnx_provider.rs:1-19` — the module doc and the EP-selection table (L13 the Cuda row, L14 the Vulkan row).
  - `frontend/src-tauri/src/audio/onnx_provider.rs:21-39` — the imports and the cfg-gated provider imports.
  - `frontend/src-tauri/src/audio/onnx_provider.rs:45` — `pub fn get_onnx_providers(gpu_type: GpuType) -> Vec<ExecutionProviderDispatch>`.
  - `frontend/src-tauri/src/audio/onnx_provider.rs:48-106` — the `match gpu_type`: L50 `None | OpenCL`; L55-69 the Metal branch (L56-64 the CoreML EP with `CoreMLComputeUnits::CPUAndNeuralEngine`); L72-88 the Cuda branch (**L76 `CUDAExecutionProvider::default().build()`**, L78-81 the `not(feature="ort-cuda")` warn, **L83-87 the unconditional `#[cfg(feature = "ort-directml")]` DirectML push**); L91-105 the Vulkan branch (L93-97 `#[cfg(all(target_os = "windows", feature = "ort-directml"))]`, **L99 the misleading comment, L100-104 `#[cfg(all(not(target_os="windows"), feature="ort-openvino"))]`**); L109 `providers.push(CPUExecutionProvider::default().build());` (CPU always last).
  - `frontend/src-tauri/src/audio/onnx_provider.rs:114-183` — the tests: L119 `test_cpu_only_fallback` (asserts `len == 1`); L130 `test_metal_includes_coreml` (asserts `len >= 2`); L148 `test_cuda_includes_cuda` (**L152 the "Cuda" casing**); L166-182 `test_cpu_last_in_chain` (**L169-175 the manual `&[...]` array, L177-180 the `!providers.is_empty()` assertion**).
  - `frontend/src-tauri/src/audio/hardware_detector.rs:15-22` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum GpuType { None, Metal, Cuda, Vulkan, OpenCL }` (Debug already derived; file is OUT OF SCOPE).
  - `~/.cargo/registry/src/*/ort-2.0.0-rc.10/src/execution_providers/mod.rs:331` — the registry proving `.build()` returns the dispatch directly.
  - `frontend/src-tauri/src/funasr_onnx/model.rs:39` and `:180` — the callers of `get_onnx_providers` **outside** `audio/onnx_provider.rs`; the four in-file unit-test call sites are `frontend/src-tauri/src/audio/onnx_provider.rs:121, 132, 150, 176` (so the complete inventory is six call sites, not two).
  - `frontend/src-tauri/Cargo.toml` — the `ort` features (`ort-cuda`, `ort-coreml`, `ort-directml`, `ort-openvino`) and the `target_os` gating of the `ort-directml` feature.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with no new warnings.
  - `cargo check -p meetily --features ort-directml` (Linux) exits 0 and `grep -n 'all(target_os = "windows", feature = "ort-directml")' frontend/src-tauri/src/audio/onnx_provider.rs` shows BOTH DirectML branches (CUDA path and Vulkan path) gated by `target_os = "windows"`.
  - `grep -n 'not(target_os="windows")\|not(target_os = "windows")' frontend/src-tauri/src/audio/onnx_provider.rs` returns NO hits (the OpenVINO gate is now Linux-only).
  - `grep -n 'all(target_os = "linux", feature = "ort-openvino")' frontend/src-tauri/src/audio/onnx_provider.rs` matches.
  - The extended `test_cpu_last_in_chain` asserts the LAST provider is the CPU one, and the GpuType coverage is exhaustive via a `#[allow(unreachable_patterns)]` wildcard arm that panics on an uncovered variant (verify by reading the final test source — a deliberate `panic!` in the wildcard arm, and NO `&[GpuType::…]` array remains: `grep -n 'GpuType::None, GpuType::Metal' frontend/src-tauri/src/audio/onnx_provider.rs` returns no hits).
  - `cargo test -p meetily onnx_provider` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily onnx_provider::tests::test_cpu_last_in_chain` passes and the CPU provider is genuinely last in the returned vector.
  - Failure: temporarily add a new `GpuType` variant in a scratch copy of the enum and confirm the wildcard arm would panic — if the arm is missing, the coverage guarantee is absent. (Do NOT commit the scratch change; record the result in the evidence file.)
  Evidence: `.omo/evidence/task-21-fork-review-fixes.txt`
  Commit: Y | `fix(onnx): gate DirectML and OpenVINO providers by operating system`

- [x] 22. `frontend/src-tauri/src/audio/mod.rs` — make internal-only modules and re-exports crate-private (2 findings)
  What to do / Must NOT do:
  1. **Modules that are never re-exported (low/maintainability, lines 16, 26, 28, 33, 34).** `pub mod incremental_saver;` (L26), `pub mod simple_level_monitor;` (L28), `pub mod async_logger;` (L33), and `pub mod batch_processor;` (L34) are declared `pub` but never re-exported from this module, so their public visibility only widens the crate API (`pub mod ffmpeg_mixer;` at L16 IS re-exported at L113 — `pub use ffmpeg_mixer::{FFmpegAudioMixer, BufferStats, RNNOISE_APPLY_ENABLED};` — so for ffmpeg_mixer only the visibility, not the re-export status, is at issue). Change each to `pub(crate) mod`. Before doing so, grep the crate for every use of these paths (`ffmpeg_mixer::`, `incremental_saver::`, `simple_level_monitor::`, `async_logger::`, `batch_processor::`) — every use must be inside the crate; if a use lives in another crate (the `llama-helper` crate cannot use them — it is a separate binary), no change is needed.
  2. **Internal-only re-exports widen the API (low/maintainability, lines 103 and 113).** `pub use device_monitor::{AudioDeviceMonitor, DeviceEvent, DeviceMonitorType};` (L103) and `pub use ffmpeg_mixer::{FFmpegAudioMixer, BufferStats, RNNOISE_APPLY_ENABLED};` (L113) are internal-only (the file contains 23 `pub use` statements in total, L55-121 — narrow ONLY the two groups the finding names; do NOT sweep the rest of the file). Change them to `pub(crate) use`. Grep the crate for each of the seven names first and confirm every use is crate-internal; if a name is used from a `pub` API surface, keep that one name `pub` and record the reason in the evidence file.
  **Must NOT do:** do **not** touch the other `pub use` groups in this file (devices, capture, system_detector, system_audio_commands, recording_state, pipeline, stream, recording_manager, recording_commands, recording_preferences, recording_saver, level_monitor, buffer_pool, post_processor, hardware_detector, encode, device_detection, diagnostics, vad, decoder, constants) — they are consumed by other modules and are not in scope; do **not** change `pub mod transcription;` (T18 handles the transcription tree) or `pub mod common;`; do **not** remove a re-export that is used outside the crate; do **not** reorder or reformat the file.
  Parallelization: Wave 3 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/mod.rs:2-53` — the module declarations: L16 `pub mod ffmpeg_mixer;`, L26 `pub mod incremental_saver;`, L28 `pub mod simple_level_monitor;`, L33 `pub mod async_logger;`, L34 `pub mod batch_processor;`, L41 `pub mod transcription;`, L44 `pub(crate) mod common;`.
  - `frontend/src-tauri/src/audio/mod.rs:103` — `pub use device_monitor::{AudioDeviceMonitor, DeviceEvent, DeviceMonitorType};`.
  - `frontend/src-tauri/src/audio/mod.rs:113` — `pub use ffmpeg_mixer::{FFmpegAudioMixer, BufferStats, RNNOISE_APPLY_ENABLED};`.
  - `frontend/src-tauri/src/audio/mod.rs:55-121` — the other `pub use` groups that are explicitly OUT of scope.
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with no new warnings (an unused `pub(crate)` re-export produces a dead-code warning — if one appears, either the item really is unused, in which case remove the re-export line and record it, or it is used elsewhere, in which case revert that single item to `pub` and record why).
  - `grep -n 'pub mod ffmpeg_mixer\|pub mod incremental_saver\|pub mod simple_level_monitor\|pub mod async_logger\|pub mod batch_processor' frontend/src-tauri/src/audio/mod.rs` returns NO hits; the corresponding `pub(crate) mod` lines are present.
  - `grep -n 'pub use device_monitor::\|pub use ffmpeg_mixer::' frontend/src-tauri/src/audio/mod.rs` returns NO hits.
  - `cargo test -p meetily` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo check -p meetily` succeeds with every in-crate consumer of the seven narrowed names still compiling.
  - Failure: `grep -rn 'use .*::audio::\(AudioDeviceMonitor\|DeviceEvent\|DeviceMonitorType\|FFmpegAudioMixer\|BufferStats\|RNNOISE_APPLY_ENABLED\)' frontend/src-tauri/src` must return NO hits after the change (the pattern matches ANY path form, not just `crate::`-prefixed ones — the old `use crate::audio::…` pattern could never observe an outside consumer and was already 0 today, so it had no red state).
  Evidence: `.omo/evidence/task-22-fork-review-fixes.txt`
  Commit: Y | `refactor(audio): make internal-only modules and re-exports crate-private`

- [x] 23. `frontend/src-tauri/src/audio/transcription/funasr_provider.rs` — move ONNX inference off the async runtime, derive `is_model_loaded`, and preserve structured errors (5 findings)
  What to do / Must NOT do:
  1. **Dead `is_model_loaded` stub (medium/bug, lines 25-27).** `async fn is_model_loaded(&self) -> bool { true }` is a literal, so it reports "loaded" for an engine that may not be usable. Replace it with a real derivation: store the resolved model name on the provider (derived from the model directory passed to `new`, e.g. `model_dir.file_name()` → `String`) in a new field, and implement `is_model_loaded()` as `!self.model_name.is_empty()` (i.e. derived from real state, not a literal). Implement `get_current_model()` (lines 29-31) to return that stored name instead of delegating to `FunasrEngine::current_model()`, which returns the hardcoded `"paraformer-large"` (`funasr_onnx/engine.rs:62-64` — deliberately OUT OF SCOPE for this plan, see the Scope "deliberate non-changes" note). Keep the public signature of `new()` unchanged.
  2. **`Vec<f32>` by value (low/performance, lines 33-37).** The `Vec<f32>` parameter is dictated by the `TranscriptionProvider` trait, which lives in `audio/transcription/provider.rs` — an OUT-OF-SCOPE file. The trait signature therefore CANNOT change. Resolve the finding in-file: (a) add a comment at the impl site stating that the by-value `Vec<f32>` is a trait constraint (owned by `provider.rs`) and must not be changed here; (b) ensure the impl never makes an ADDITIONAL copy of the buffer — pass `&audio` down and MOVE the `Vec<f32>` into the blocking task (no `audio.clone()`).
  3. **Sync CPU-heavy inference on the async runtime (high/performance, lines 41-48).** `self.engine.transcribe_audio(&audio)` runs synchronous ONNX inference directly on the async executor thread, blocking the runtime. Change the provider's field to `engine: Arc<FunasrEngine>` (public API of `FunasrProvider::new` unchanged — it still returns `Result<Self, FunasrError>`), then run the inference as `tokio::task::spawn_blocking(move || engine.transcribe_audio(&samples))` with the `Vec<f32>` moved in. `tokio` already has the `full` feature (`frontend/src-tauri/Cargo.toml:122`) — no new dependency. `Arc<FunasrEngine>` requires `FunasrEngine: Send + Sync`; add a compile-time static assertion `fn _assert_send_sync<T: Send + Sync>() {} fn _assert() { _assert_send_sync::<FunasrEngine>(); }` in this file's `#[cfg(test)]` module. If that assertion FAILS to compile, do NOT edit `funasr_onnx/engine.rs` (T10's file, scope fixed at its 5 flagged findings): record the failure, the exact compiler error, and the required upstream change in the evidence file, and report it as a BLOCKER for this todo instead of silently working around it.
  4. **Flattened error type (medium/maintainability, lines 42-48).** `TranscriptionError` (in the OUT-OF-SCOPE `provider.rs`) has no variant able to carry `FunasrError`, so "add a variant / `From<FunasrError>`" is IMPOSSIBLE in scope. In-scope resolution: preserve the structured error through the `Debug` representation instead of `e.to_string()` — extract a pure helper `fn map_engine_error(e: &FunasrError) -> TranscriptionError` that formats with `format!("{e:?}")` (T9 gives `FunasrError` structured variants and `#[source]` chaining, so the `Debug` output contains the variant name AND the source chain), and log the full chain with `log::error!("FunASR transcription failed: {e:?}")`. The helper must be unit-testable without a real model.
  5. **Noisy per-transcription log (low/maintainability, line 49).** `info!("FunASR provider transcription completed: '{}'", text)` logs at info level for EVERY transcription, echoing the full transcript into the log. Downgrade it to `debug!` (keep the message text unchanged) so normal transcription does not spam the log and transcripts do not leak into info-level logs.

  **Must NOT do:** do **not** modify `audio/transcription/provider.rs` (the trait and `TranscriptionError` are out of scope); do **not** change the `TranscriptionProvider` trait signature or add a `TranscriptionError` variant; do **not** change `FunasrProvider::new`'s signature or its error type; do **not** add a dependency; do **not** change the `TranscriptResult` payload shape (`text`, `confidence: None`, `is_partial: false`); do **not** touch `funasr_onnx/engine.rs` (T10's file) even though `current_model()` is hardcoded there — record it as a deliberate non-change instead.
  Parallelization: Wave 4 | Blocked by: T1, T9, T18 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:8-10` — `pub struct FunasrProvider { engine: FunasrEngine }` (field to become `Arc<FunasrEngine>`).
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:12-17` — `pub fn new(model_dir: &Path) -> Result<Self, FunasrError>`.
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:21-23` — `provider_name()` delegating to the engine.
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:25-27` — the `is_model_loaded` stub.
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:29-31` — `get_current_model()`.
  - `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:33-55` — `transcribe()`, including the language-hint warning (L38-40), the sample-count log (L41), the `map_err` (L42-48), the verbatim transcript log (L49) and the `TranscriptResult` construction (L50-54).
  - `frontend/src-tauri/src/audio/transcription/provider.rs:1-73` — OUT OF SCOPE: the trait and the `TranscriptionError` enum that constrain this todo.
  - `frontend/src-tauri/src/funasr_onnx/engine.rs:48-56` — `FunasrEngine::transcribe_audio(&self, audio: &[f32])` (synchronous, CPU-heavy).
  - `frontend/src-tauri/src/funasr_onnx/engine.rs:62-64` — `current_model()` hardcoded to `"paraformer-large"` (out of scope; deliberate non-change).
  - `frontend/src-tauri/Cargo.toml:122` — `tokio` with the `full` feature (`spawn_blocking` available).
  - `frontend/src-tauri/src/audio/transcription/engine.rs:282-296` — the call site `FunasrProvider::new(&model_dir)` (must keep compiling unchanged).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - A `#[cfg(test)]` module in `funasr_provider.rs` proves, WITHOUT a real ONNX model: (a) `map_engine_error(&FunasrError::…)` returns `TranscriptionError::EngineFailed` whose message contains the STRUCTURED variant name (e.g. `ModelNotFound`) — a test asserting the message contains a variant name FAILS if the implementation uses a flattened `to_string()` that loses it; (b) the derived `is_model_loaded` is false for an empty stored model name and true otherwise, via a pure helper that the test can call; (c) the static assertion `_assert_send_sync::<FunasrEngine>()` compiles; (d) a structural guard test using `include_str!("funasr_provider.rs")` asserting the source contains `spawn_blocking` and contains NO direct `self.engine.transcribe_audio(` call outside the blocking closure.
  - `grep -n 'spawn_blocking' frontend/src-tauri/src/audio/transcription/funasr_provider.rs` returns a hit; `grep -n 'audio.clone()\|samples.clone()' frontend/src-tauri/src/audio/transcription/funasr_provider.rs` returns no hits.
  - `grep -n 'info!("FunASR provider transcription completed' frontend/src-tauri/src/audio/transcription/funasr_provider.rs` returns NO hits (remove/downgrade the `info!` at `frontend/src-tauri/src/audio/transcription/funasr_provider.rs:49`).
  - `cargo test -p meetily funasr_provider` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily funasr_provider` — the pure-helper tests pass; `map_engine_error` message contains `ModelNotFound` (structured) not an opaque string.
  - Failure: temporarily revert `map_engine_error` to `e.to_string()` and re-run the test — it MUST fail (red→green evidence). If it still passes, the test is not asserting the structured form and must be tightened before the fix is accepted.
  - Failure: if the `_assert_send_sync::<FunasrEngine>()` assertion does not compile, capture the exact `E0277`/`E0275` error text into the evidence file and stop this todo as BLOCKED (do not work around it by editing `funasr_onnx/engine.rs`).
  Evidence: `.omo/evidence/task-23-fork-review-fixes.txt`
  Commit: Y | `fix(transcription): run funasr inference off the async runtime and derive model state`

- [x] 24. `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs` — move ONNX inference off the async runtime, derive model state, and stop logging transcripts (5 findings)
  What to do / Must NOT do:
  1. **`is_model_loaded()` always true + hardcoded `get_current_model()` (low/maintainability, lines 25-31).** `is_model_loaded()` returns the literal `true`; `get_current_model()` returns `Some("sensevoice".to_string())` regardless of the loaded model. Fix: `SherpaOnnxEngine` must expose `pub fn current_model(&self) -> Option<String>` returning the RESOLVED model directory name (T16 adds it — T16 is blocked-by this dependency direction: T16 blocks T24). The provider then implements `is_model_loaded()` as `self.engine.current_model().is_some()` and `get_current_model()` as `self.engine.current_model()` — no literal model name anywhere in this file.
  2. **Sync inference on the async executor (high/performance, lines 45-51).** `self.engine.transcribe_audio(&audio, 16000)` is synchronous CPU-heavy work executed on the async runtime thread. Change the field to `engine: Arc<SherpaOnnxEngine>` (the public API of `SherpaOnnxProvider::new` is unchanged — it still returns `Result<Self, SherpaOnnxError>`) and run it via `tokio::task::spawn_blocking(move || engine.transcribe_audio(&samples, 16000))`, MOVING the `Vec<f32>` into the task (no clone). `tokio` `full` feature is already enabled (`frontend/src-tauri/Cargo.toml:122`) — no new dependency. Add a compile-time static assertion that `SherpaOnnxEngine: Send + Sync` (a `fn _assert_send_sync<T: Send + Sync>() {}` + a call in the `#[cfg(test)]` module). If it does not compile, do NOT edit `sherpa_onnx/engine.rs` (T16's file): record the exact compiler error and report it as a BLOCKER.
  3. **`Vec<f32>` by value + clone into `spawn_blocking` (low/performance, lines 45-51).** The by-value parameter is a TRAIT CONSTRAINT (`TranscriptionProvider::transcribe(&self, audio: Vec<f32>, …)` in the OUT-OF-SCOPE `audio/transcription/provider.rs`) and cannot be changed. The "clone into spawn_blocking" half is NOT REPRODUCIBLE at this commit — the current code borrows (`&audio`) and never clones. Resolve in-file: add a comment at the impl site stating that the by-value `Vec<f32>` is a trait constraint owned by `provider.rs` and must not change here, and MOVE the vector into the blocking task (zero copies). Record the "clone" half as NOT REPRODUCIBLE in the evidence file.
  4. **Flattened error type (medium/maintainability, lines 48-51).** `TranscriptionError` (OUT OF SCOPE) cannot carry `SherpaOnnxError`, so "add a variant" is IMPOSSIBLE in scope. Resolve by preserving the structured error through `Debug`: extract a pure helper `fn map_engine_error(e: &SherpaOnnxError) -> TranscriptionError` using `format!("{e:?}")` (T15 gives `SherpaOnnxError` structured variants + `#[source]`), and log the full chain with `log::error!("Sherpa-ONNX transcription failed: {e:?}")`.
  5. **PII in logs (medium/security, lines 52-55).** `info!("Sherpa-ONNX provider transcription completed: '{}'", text)` writes the full transcript to the application log. Remove the verbatim transcript from the log line; replace it with a non-reversible summary — e.g. `info!("Sherpa-ONNX provider transcription completed ({} chars)", text.chars().count())` — or a `log::debug!`-level truncated preview if a preview is genuinely needed. Never log the transcript text at info/warn/error level.

  **Must NOT do:** do **not** modify `audio/transcription/provider.rs`; do **not** add a `TranscriptionError` variant or change the trait signature; do **not** change `SherpaOnnxProvider::new`'s signature; do **not** edit `sherpa_onnx/engine.rs` (T16 owns it — if the `current_model()` accessor or the `Send + Sync` assertion is missing, report it as BLOCKED and let T16 land it); do **not** log transcript text at info level or above; do **not** change the sample rate literal `16000` (it is the model's native rate); do **not** add a dependency.
  Parallelization: Wave 4 | Blocked by: T1, T15, T16, T18 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:8-10` — `pub struct SherpaOnnxProvider { engine: SherpaOnnxEngine }` (field to become `Arc<SherpaOnnxEngine>`).
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:12-17` — `new(model_dir)`.
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:21-23` — `provider_name()` → `"sherpa-onnx (SenseVoice)"`.
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:25-27` — the `is_model_loaded()` literal.
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:29-31` — the hardcoded `get_current_model()`.
  - `frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs:33-61` — `transcribe()`: the empty-audio guard (L38-43), the sample-count log (L44), the `map_err` block (L45-51), the verbatim transcript log (L52-55), the `TranscriptResult` construction (L56-60).
  - `frontend/src-tauri/src/audio/transcription/provider.rs:1-73` — OUT OF SCOPE: trait + `TranscriptionError`.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:31` — `SherpaOnnxEngine::new(&Path)`.
  - `frontend/src-tauri/src/sherpa_onnx/engine.rs:67-85` — `transcribe_audio(&self, samples: &[f32], sample_rate: u32)` (synchronous; T16 adds `current_model()` here).
  - `frontend/src-tauri/src/sherpa_onnx/mod.rs:9-21` — `SherpaOnnxError` variants (T15 restructures them with `#[source]`).
  - `frontend/src-tauri/Cargo.toml:122` — `tokio` `full` feature.
  - `frontend/src-tauri/src/audio/transcription/engine.rs:297-315` — the call site `SherpaOnnxProvider::new(&model_dir)` (must keep compiling unchanged).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - A `#[cfg(test)]` module in `sherpa_onnx_provider.rs` proves WITHOUT a real model: (a) `map_engine_error(&SherpaOnnxError::ModelNotFound { path: std::path::PathBuf::from("/tmp/x") })` returns `TranscriptionError::EngineFailed` whose message contains the structured variant name `ModelNotFound` (written against T15's post-T15 struct form `ModelNotFound { path }`, NOT the pre-T15 tuple form `ModelNotFound(String)` — T15 is a blocker of this todo); (b) the static assertion `_assert_send_sync::<SherpaOnnxEngine>()` compiles; (c) a structural guard test using `include_str!("sherpa_onnx_provider.rs")` asserting the source contains `spawn_blocking` and contains NO direct `self.engine.transcribe_audio(` call outside the blocking closure; (d) a source-level guard asserting the file contains NO `info!("… {}", text)`-style transcript echo (e.g. assert the source does not contain `transcription completed: '{}'`).
  - `grep -n 'spawn_blocking' frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs` returns a hit; `grep -n 'Some("sensevoice"' frontend/src-tauri/src/audio/transcription/sherpa_onnx_provider.rs` returns NO hits.
  - `cargo test -p meetily sherpa_onnx_provider` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily sherpa_onnx_provider` — the structured-error and structural-guard tests pass.
  - Failure: revert `map_engine_error` to `e.to_string()` and re-run — the test MUST fail; if it still passes, tighten the assertion (the message must name the variant, not merely be non-empty).
  - Failure: if T16 has not landed `current_model()`, `cargo check` fails with "no method named `current_model`" — capture that error and report BLOCKED (do not re-add the hardcoded literal).
  Evidence: `.omo/evidence/task-24-fork-review-fixes.txt`
  Commit: Y | `fix(transcription): run sherpa inference off the async runtime and stop logging transcripts`

- [x] 25. `frontend/src-tauri/src/audio/import.rs` — validate the provider, stop double-initialising Sherpa-ONNX, reuse the engine per segment, and add cache invalidation (5 findings)
  What to do / Must NOT do:
  1. **Never-invalidated global engine caches (low/maintainability, lines 34-38).** `static IMPORT_FUNASR_ENGINE: OnceLock<Arc<FunasrEngine>>` (L35) and `static SHERPA_ONNX_ENGINE: OnceLock<Arc<SherpaOnnxProvider>>` (L38) are set once and never cleared, so a user who deletes or replaces a model keeps the stale engine. Add a `pub(crate) fn invalidate_engine_cache()` (or `refresh_engines()`) in this file that clears BOTH `OnceLock`s, and call it from the point where the import unloads engines (the L281-283 area, which currently unloads only when neither funasr nor sherpa is used) and after any model (re)load. `OnceLock` cannot be reset in place — use the `std::sync::OnceLock` + `Arc` pattern already used elsewhere, or replace the two statics with `RwLock<Option<Arc<…>>>`/`Mutex<Option<Arc<…>>>` so they can be cleared. Choose ONE approach and apply it consistently to both statics; document the choice in a comment.
  2. **Unvalidated provider string (medium/maintainability, lines 260-277).** `provider: Option<String>` is compared against the magic strings `"parakeet"`, `"funasr"`, `"sherpa-onnx"` (L274-276) and ANY other value silently falls through to Whisper. Introduce a typed enum in THIS file (e.g. `enum ImportProvider { Whisper, Parakeet, Funasr, SherpaOnnx }`) with `impl TryFrom<&str> for ImportProvider` that returns a descriptive `Err` for an unknown provider, and map `Option<String>` → `ImportProvider` at the command boundary (`start_import`, L260-267): `None` → `Whisper` (the current default), `Some(s)` → `ImportProvider::try_from(s.as_str())?`. Replace the three `use_*` booleans with `match`/`matches!` on the enum. Unknown providers must produce an error, not a silent Whisper fallback.
 3. **Sherpa-ONNX engine initialised twice (low/maintainability, lines 542-547).** The engine is created once at L542-547 (binding `_sherpa_onnx_engine`) and AGAIN inside the loop at L630-631 via `get_or_init_sherpa_onnx(...)` for every segment. Remove the per-segment initialisation: create the engine ONCE before the segment loop (reuse the binding from L542-547), pass it by reference/cloned handle to each segment, and delete the per-segment call. The binding `_sherpa_onnx_engine` at L542 becomes the reused engine (rename it to something meaningful).
  4. **Dead shadowed binding (low/bug, lines 542-547).** `_sherpa_onnx_engine` is assigned and never read. Remove the dead binding and keep exactly ONE engine value for the whole import: a single `let sherpa_onnx_engine = if matches!(provider, ImportProvider::SherpaOnnx) && total_segments > 0 { Some(get_or_init_sherpa_onnx(&app, model.as_deref()).await?) } else { None };` bound BEFORE the segment loop, named WITHOUT the underscore prefix.
  5. **Per-segment engine re-initialisation (low/performance, lines 629-632).** The Sherpa segment branch calls `get_or_init_sherpa_onnx(&app, model.as_deref()).await` for EVERY segment. Change it to clone the outer `sherpa_onnx_engine` handle (an `Arc` clone) once per segment — or hoist the `Arc` clone out of the loop — and use that clone. The per-segment call must disappear: `grep -n 'get_or_init_sherpa_onnx' frontend/src-tauri/src/audio/import.rs` must show the call exactly 2 times total (one call + the definition) and the per-segment call is gone.

  **Must NOT do:** do **not** change the `start_import` Tauri command's parameter list (the frontend passes `provider: Option<String>`; the typed enum is an INTERNAL boundary conversion — see the frontend contract note in the Scope section); do **not** change the import file formats, the segmenting logic, or the progress events; do **not** edit `audio/retranscription.rs` (it has its own separate `SHERPA_ONNX_ENGINE` static — T26 owns that file); do **not** edit `funasr_onnx/` or `sherpa_onnx/`; do **not** add a dependency.
  Parallelization: Wave 4 | Blocked by: T1, T18 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/import.rs:29` — `static IMPORT_IN_PROGRESS: AtomicBool`.
  - `frontend/src-tauri/src/audio/import.rs:32` — `static IMPORT_CANCELLED: AtomicBool`.
  - `frontend/src-tauri/src/audio/import.rs:35` — `static IMPORT_FUNASR_ENGINE: OnceLock<Arc<FunasrEngine>>`.
  - `frontend/src-tauri/src/audio/import.rs:38` — `static SHERPA_ONNX_ENGINE: OnceLock<Arc<SherpaOnnxProvider>>`.
  - `frontend/src-tauri/src/audio/import.rs:40-46` — `struct ImportGuard` (L42) + `impl ImportGuard {` (L44) + `fn acquire() -> Result<Self, String>` (L46).
  - `frontend/src-tauri/src/audio/import.rs:260-267` — `pub async fn start_import<R: Runtime>(app, source_path, title, language, model, provider: Option<String>)` (the command boundary).
  - `frontend/src-tauri/src/audio/import.rs:274-276` — the three magic-string provider comparisons.
  - `frontend/src-tauri/src/audio/import.rs:277` — the `run_import(...)` call.
  - `frontend/src-tauri/src/audio/import.rs:281-283` — the engine-unload branch (the hook point for invalidation).
  - `frontend/src-tauri/src/audio/import.rs:527-531` / `:532-536` / `:537-541` — the whisper / parakeet / funasr engine initialisations.
  - `frontend/src-tauri/src/audio/import.rs:542-547` — the dead `_sherpa_onnx_engine` binding (the ONLY initialisation: binding at L542, `get_or_init_sherpa_onnx(...)` call at L544).
  - `frontend/src-tauri/src/audio/import.rs:612-621` / `:622-628` / `:629-635` / `:636-643` — the four segment branches (funasr / parakeet / sherpa / whisper).
  - `frontend/src-tauri/src/audio/import.rs:911` / `:928` — `IMPORT_FUNASR_ENGINE.get()/set`.
  - `frontend/src-tauri/src/audio/import.rs:940` / `:958` — `SHERPA_ONNX_ENGINE.get()/set`.
  - `frontend/src-tauri/src/audio/retranscription.rs:32` — a DIFFERENT module-level `SHERPA_ONNX_ENGINE` (not the same static; T26's file).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - A `#[cfg(test)]` module in `import.rs` proves, WITHOUT a real model: (a) `ImportProvider::try_from("parakeet"|"funasr"|"sherpa-onnx"|"localWhisper"|"whisper")` maps to the expected variants and `try_from("nonsense")` returns `Err` (no silent Whisper fallback); (b) `None` maps to `ImportProvider::Whisper`; (c) the cache-invalidation helper actually clears the engine caches — assert that a value stored in the cache is gone after the helper runs (make the helper testable by parameterising it over the statics or by extracting a small, generic `clear_once_lock` helper).
  - `grep -n 'get_or_init_sherpa_onnx' frontend/src-tauri/src/audio/import.rs` shows exactly 2 hits total — ONE call (the pre-loop initialisation) + the `async fn get_or_init_sherpa_onnx` definition — and the per-segment call inside the segment loop is gone (at 5549e15 the string occurs at L544 call, L630 per-segment call, L936 definition; after the fix only the call + definition remain).
  - `grep -n '_sherpa_onnx_engine' frontend/src-tauri/src/audio/import.rs` returns NO hits.
  - `cargo test -p meetily audio::import` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily audio::import` — the provider-mapping and cache-invalidation tests pass.
  - Failure: `ImportProvider::try_from("nonsense")` must return `Err`; if it returns `Ok(Whisper)` the validation is missing → the test must fail (red→green evidence).
  Evidence: `.omo/evidence/task-25-fork-review-fixes.txt`
  Commit: Y | `fix(import): validate the provider, reuse the sherpa engine, and invalidate caches`

- [x] 26. `frontend/src-tauri/src/audio/retranscription.rs` — remove per-segment engine churn, fix the FunASR init race, and surface lock poisoning (9 findings)
  What to do / Must NOT do:
  1. **`language.clone()` per segment (low/performance, line 475 / segment loop).** The language string is cloned for every segment. Clone it ONCE before the loop into a local `let language = language.clone();` (or pass `&language`) and reuse the local inside the loop.
  2. **Dead `_sherpa_onnx_engine` binding (medium/maintainability, lines 376-381).** The binding is assigned at L376-381 and never used.
  3. **Duplicate `get_or_init_sherpa_onnx` call per segment (medium/maintainability, lines 376-381 / L466).** The Sherpa branch calls `get_or_init_sherpa_onnx(&app, model.as_deref()).await` again for every segment at L466.
  4. **Dead binding removal + engine reuse (the 3rd of the three L376-381 entries).** Fix 2 and 3 together: bind the engine ONCE before the segment loop as `let sherpa_onnx_engine = if use_sherpa_onnx { Some(get_or_init_sherpa_onnx(&app, model.as_deref()).await?) } else { None };` (no underscore prefix), and inside the segment branch clone that `Arc` handle instead of re-initialising.
  5. **`funasr_engine.as_ref().unwrap().clone()` per segment (low/performance, lines 447-450).** The `Arc` is cloned for every segment, churning the refcount.
  6. **Same clone churn as a maintainability defect (low/maintainability, lines 447-450).** Clone the `Arc` ONCE before the loop (`let funasr_engine = funasr_engine.as_ref().cloned();`) and clone only the cheap `Arc` handle per segment, not per-segment `Arc` construction from `unwrap()`.
  7. **`parakeet_engine.as_ref().unwrap()` borrow held across an `.await` (lines 458-464).** The borrow of `parakeet_engine` is alive across the `.await` at L462. Take an owned `Arc` clone BEFORE the await (`let engine = parakeet_engine.as_ref().cloned();`) and use the owned handle inside the async block, so no borrow of the outer binding crosses an await point.
  8. **Silent lock-poison recovery (lines 651-654).** `WHISPER_ENGINE.lock().unwrap_or_else(|e| e.into_inner())` silently recovers from poisoning. Keep the recovery (it is the intended behaviour) but LOG it: `log::warn!("WHISPER_ENGINE lock poisoned; recovering the inner value")` before `into_inner()`. Note: the review's "mutex held across await" claim is FALSE here — the guard is dropped at L654 — so the ONLY change is the added warning (record the "held across await" claim as NOT REPRODUCIBLE in the evidence file).
  9. **TOCTOU race on `FUNASR_ENGINE.get()` → `FUNASR_ENGINE.set(...)` (high/bug, lines 876-894, three entries).** Two concurrent callers both observe `None`, both perform the expensive `FunasrEngine::new(&model_dir)`, and the second `set` fails with `"FunASR engine already initialized"`. Fix with `OnceLock::get_or_init()` (or a `tokio::sync::Mutex`/`OnceCell`) so the initialisation happens exactly once and concurrent callers await the same result. Apply the same fix to the analogous `SHERPA_ONNX_ENGINE` get/set pair (L905 / L923) — it has the identical race.

  **Must NOT do:** do **not** change the retranscription command signatures, the DB write ordering, or the metadata/JSON writing (the `572-591` ordering defect and the `751-758`/`751-759` query defects are DEFERRED to the upstream PR — T33; do NOT fix them here); do **not** edit `audio/import.rs` (it has its own separate statics — T25 owns that file); do **not** change `MAX_SEGMENT_SAMPLES` (`retranscription.rs:386`); do **not** add a dependency; do **not** change the transcript text handling beyond the two `clone` reductions.
  Parallelization: Wave 4 | Blocked by: T1, T18 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/retranscription.rs:29` — `static FUNASR_ENGINE: OnceLock<Arc<FunasrEngine>>`.
  - `frontend/src-tauri/src/audio/retranscription.rs:32` — `static SHERPA_ONNX_ENGINE: OnceLock<Arc<SherpaOnnxProvider>>`.
  - `frontend/src-tauri/src/audio/retranscription.rs:371-375` — the funasr engine initialisation before the loop.
  - `frontend/src-tauri/src/audio/retranscription.rs:376-381` — the dead `_sherpa_onnx_engine` binding.
  - `frontend/src-tauri/src/audio/retranscription.rs:386` — `const MAX_SEGMENT_SAMPLES: usize = 25 * 16000;`.
  - `frontend/src-tauri/src/audio/retranscription.rs:447-457` — the funasr segment branch (`unwrap().clone()` + `segment.samples.clone()` + `spawn_blocking`).
  - `frontend/src-tauri/src/audio/retranscription.rs:458-464` — the parakeet segment branch (borrow across the `.await` at L462).
  - `frontend/src-tauri/src/audio/retranscription.rs:465-470` — the sherpa segment branch (per-segment `get_or_init_sherpa_onnx`).
  - `frontend/src-tauri/src/audio/retranscription.rs:651-654` — the silent `unwrap_or_else(|e| e.into_inner())` poison recovery.
  - `frontend/src-tauri/src/audio/retranscription.rs:872-898` — `get_or_init_funasr` (L876 `.get()`, L889-891 `FunasrEngine::new`, L893-894 `.set()`).
  - `frontend/src-tauri/src/audio/retranscription.rs:905` / `:923` — the analogous `SHERPA_ONNX_ENGINE` get/set pair.
  - `frontend/src-tauri/src/audio/import.rs:35` / `:38` — the SEPARATE statics in `import.rs` (do not conflate them).
  - `frontend/src-tauri/Cargo.toml:122` — `tokio` `full` feature (`tokio::sync::OnceCell` is available; `std::sync::OnceLock::get_or_init` needs no dependency at all).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `retranscription.rs` prove: (a) the engine-cache initialisation helper is idempotent and race-free — extract the get/init/set sequence into a testable helper (e.g. `fn init_funasr_cache(slot: &OnceLock<Arc<FunasrEngine>>, model_dir: &Path) -> Result<Arc<FunasrEngine>, …>` implemented with `get_or_init`) and assert that calling it twice yields the SAME `Arc` (pointer equality) and returns `Ok` on the second call (the current code returns `Err("FunASR engine already initialized")` — that is the red state); (b) the same for the sherpa cache helper; (c) a structural guard test using `include_str!("retranscription.rs")` asserting that the per-segment `get_or_init_sherpa_onnx` call inside the segment loop is gone — count occurrences of `get_or_init_sherpa_onnx` EXCLUDING the `async fn get_or_init_sherpa_onnx` definition line and expect exactly 1 (the pre-loop call; at 5549e15 `get_or_init` generally appears at L362/367/372/378, L645/763/873/901, and `get_or_init_sherpa_onnx` specifically at the pre-loop call L378 + definition L901, so a bare "appears only once" assertion has no red state).
  - `grep -n '_sherpa_onnx_engine' frontend/src-tauri/src/audio/retranscription.rs` returns NO hits.
  - `grep -n 'unwrap_or_else(|e| e.into_inner())' frontend/src-tauri/src/audio/retranscription.rs` shows a `log::warn!` on the preceding line.
  - `cargo test -p meetily retranscription` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily retranscription` — the double-init test returns the same `Arc` and `Ok`.
  - Failure: temporarily restore the get/init/set pattern in the helper and re-run — the test MUST fail with "FunASR engine already initialized" (red→green evidence).
  Evidence: `.omo/evidence/task-26-fork-review-fixes.txt`
  Commit: Y | `fix(retranscription): remove per-segment engine churn and fix the funasr init race`

- [x] 27. `frontend/src-tauri/src/audio/recording_preferences.rs` — make the backend-config sync atomic and verify the reported dead `map_err` (2 findings)
  What to do / Must NOT do:
  1. **Dead `.map_err` after `?` (low/maintainability, lines 210-216 / `open_recordings_folder` L209-236).** The review claims a dead `?`/`.map_err` pair in `open_recordings_folder` (its cited anchors L228/L229 are actually `#[cfg(target_os = "macos")]` / `{`; the real `.map_err`+`?` pairs are L210-212 and L215-216). At this commit every `.map_err` in `open_recordings_folder` is a REAL error-type conversion performed BEFORE the `?` (`load_recording_preferences(...).await.map_err(|e| format!("Failed to load preferences: {}", e))?` at L210-212 and `ensure_recordings_directory(...).map_err(|e| format!("Failed to create directory: {}", e))?` at L215-216), so the finding is NOT REPRODUCIBLE. Verify this with the compiler (`cargo check` must show no `unused_map`/unreachable-code warning for those lines) and by reading L209-236. If — and only if — a genuinely dead `.map_err` exists, remove it; otherwise change NOTHING here and record "NOT REPRODUCIBLE at 5549e15" in the evidence file with the compiler output as proof.
  2. **Load-then-set race on the global `BACKEND_CONFIG` (medium/bug, lines 108-122).** `load_recording_preferences` (macOS only, L116-120) reads `crate::audio::capture::get_current_backend()` into `p.system_audio_backend`, and `save_recording_preferences` (L193-200) later writes the preference back into the global backend config. Two separate read-modify-write cycles against the same global ⇒ a concurrent change is lost. `audio/capture/backend_config.rs` (which owns `BACKEND_CONFIG`, `backend_config.rs:133-150`) is OUT OF SCOPE. Fix INSIDE this file: add ONE atomic sync helper in `recording_preferences.rs` (e.g. `pub(crate) fn sync_backend_config(preferences: &RecordingPreferences)`) that reads the current backend and applies the preference to `BACKEND_CONFIG` in a single, non-interleaved operation, and route BOTH the load path (L116-120) and the save path through it so no load-then-set window exists. ALSO add a doc comment on `set_audio_backend` (L294) stating that `set_audio_backend` must be used independently of `set_recording_preferences` (they write the same global), and note the same constraint on `set_recording_preferences`.

  **Must NOT do:** do **not** edit `audio/capture/backend_config.rs` (out of scope); do **not** change the on-disk `recording_preferences.json` schema or the `RecordingPreferences` struct fields; do **not** change the macOS/Windows/Linux folder-opening behaviour; do **not** change the `set_audio_backend` / `set_recording_preferences` Tauri command signatures (the frontend contract is fixed); do **not** add a dependency.
  Parallelization: Wave 4 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/recording_preferences.rs:96` — `pub async fn load_recording_preferences<R: Runtime>(...)`.
  - `frontend/src-tauri/src/audio/recording_preferences.rs:100-106` — `app.store("recording_preferences.json")`.
  - `frontend/src-tauri/src/audio/recording_preferences.rs:108-128` — the store-get + `serde_json::from_value::<RecordingPreferences>` block; L116-120 the macOS-only `get_current_backend()` read into `p.system_audio_backend`.
  - `frontend/src-tauri/src/audio/recording_preferences.rs:187` — the `load_recording_preferences(&app)` call site.
  - `frontend/src-tauri/src/audio/recording_preferences.rs:192-200` — the `set_recording_preferences` Tauri command; `save_recording_preferences` itself is at `:140-180`, and its global write is `crate::audio::capture::set_current_backend(backend);` at L172 (block L167-174).
  - `frontend/src-tauri/src/audio/recording_preferences.rs:209-236` — `open_recordings_folder` (L210-212 load + `map_err` + `?`; L215-216 `ensure_recordings_directory` + `map_err` + `?`; L218 path stringify; L220-226 Windows `explorer`; L228-234 macOS `open`; L236+ other OS).
  - `frontend/src-tauri/src/audio/recording_preferences.rs:294` — `pub async fn set_audio_backend(backend: String) -> Result<(), String>`.
  - `frontend/src-tauri/src/audio/capture/backend_config.rs:133-150` — OUT OF SCOPE: `BACKEND_CONFIG`, `get_current_backend()`, `set_current_backend(backend)`, `get_available_backends()`.
  - `frontend/src-tauri/src/audio/capture/backend_config.rs:111-119` — the `BackendConfig` interior-mutability pattern (`pub fn get(&self)` L111-113, `pub fn set(&self, backend: AudioCaptureBackend)` L116-119; L127-129 is `pub fn reset(&self)`).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0 with NO new warnings about unused `map_err` results.
  - New `#[cfg(test)]` tests in `recording_preferences.rs` prove: (a) the new sync helper is a single call site for applying the backend preference — assert (via a pure, testable helper that returns the value to apply, e.g. `fn desired_backend(preferences: &RecordingPreferences) -> Option<String>`) that the load path and the save path produce the SAME value for the same input; (b) the doc constraint on `set_audio_backend` / `set_recording_preferences` is present (`grep -n 'must be used independently of' frontend/src-tauri/src/audio/recording_preferences.rs` returns a hit).
  - The evidence file records the compiler output proving whether a dead `.map_err` exists; the finding is marked RESOLVED or NOT REPRODUCIBLE accordingly.
  - `cargo test -p meetily recording_preferences` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily recording_preferences` — the helper tests pass and the load/save paths agree.
  - Failure: temporarily reintroduce the two-step load-then-set in the save path and re-run — the test that asserts a single atomic apply MUST fail (red→green evidence).
  Evidence: `.omo/evidence/task-27-fork-review-fixes.txt`
  Commit: Y | `fix(preferences): apply the backend config atomically`

- [x] 28. `frontend/src-tauri/src/database/manager.rs` — non-blocking recovery filesystem calls, path-based WAL/SHM sidecars, and loud incompatibility logging (4 findings)
  What to do / Must NOT do:
  1. **Silent database loss (medium/other, lines 47-50).** On an incompatible schema, the code backs up and recreates the database while logging only `log::warn!("Database migration failed (incompatible existing database): {}. Backing it up and recreating a fresh one.", err_str)`. Raise the severity to `log::error!` (and add the exact backup path to the message so the user can recover the data), and add an explicit, documented statement of the data-loss behaviour. Do NOT change the recovery behaviour itself (back up + recreate) — that is a product decision, not a code defect. A configuration flag that lets the user refuse the automatic recreate is OPTIONAL: only add it if the existing config/settings surface (already in scope) can carry it WITHOUT a schema change; otherwise document the behaviour instead.
  2. **Blocking `fs::rename` inside async (high/performance, lines 60-62).** `fs::rename(db_path, &backup_path).map_err(|e| sqlx::Error::Io(e))?` blocks the async runtime. Use `tokio::fs::rename` (the `tokio` `full` feature is enabled — `frontend/src-tauri/Cargo.toml:122`) and map the error with `sqlx::Error::Io(e)` exactly as before.
  3. **WAL/SHM sidecar paths built by string formatting (medium/bug, lines 64-67).** `format!("{}-wal", tauri_db_path)` / `format!("{}-shm", tauri_db_path)` corrupt paths that contain non-UTF-8 or that need correct parent/extension semantics. Build the sidecar paths from the `Path`: `fn sidecar_path(db: &Path, suffix: &str) -> PathBuf { let mut name = db.file_name().unwrap_or_default().to_os_string(); name.push(suffix); db.with_file_name(name) }` ⇒ `/a/b/app.db` + `-wal` ⇒ `/a/b/app.db-wal`. Extract this as a small pure helper so it can be unit-tested.
  4. **Blocking `fs::remove_file` inside async (high/performance, lines 68-68).** Replace with `tokio::fs::remove_file` (keeping the existing `log::warn!` on failure). The call is `fs::remove_file(sidecar_path)` at `frontend/src-tauri/src/database/manager.rs:68` (imported via `use std::fs;` at L2).

  **Must NOT do:** do **not** change the migration detection strings (L41-43) or the backup naming scheme (`db_path.with_extension(format!("sqlite.bak-{}", ts))`, L54-58); do **not** change the SQL schema; do **not** edit `database/repositories/setting.rs` (out of scope — its two findings are DEFERRED to the upstream PR, T33); do **not** change the public function signatures of this module; do **not** add a dependency.
  Parallelization: Wave 4 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/database/manager.rs:38-39` — the comment about backing up and recreating a fresh database.
  - `frontend/src-tauri/src/database/manager.rs:40` — `let err_str = migrate_err.to_string();`.
  - `frontend/src-tauri/src/database/manager.rs:41-43` — the `incompatible` detection (`"previously applied" || "checksum" || "version mismatch"`).
  - `frontend/src-tauri/src/database/manager.rs:44-45` — `if !incompatible { return Err(migrate_err.into()); }`.
  - `frontend/src-tauri/src/database/manager.rs:47-50` — the `log::warn!` that must become `log::error!` + the backup path.
  - `frontend/src-tauri/src/database/manager.rs:51` — `pool.close().await;`.
  - `frontend/src-tauri/src/database/manager.rs:53` — `let db_path = Path::new(tauri_db_path);`.
  - `frontend/src-tauri/src/database/manager.rs:54-58` — the timestamp + `backup_path` computation.
  - `frontend/src-tauri/src/database/manager.rs:60-62` — the blocking `fs::rename`.
  - `frontend/src-tauri/src/database/manager.rs:64-72` — the WAL/SHM sidecar loop (string-formatted paths + blocking `std::fs::remove_file`).
  - `frontend/src-tauri/src/database/manager.rs:74-75` — `log::info!("Creating a fresh database at {}", …); Sqlite::create_database(tauri_db_path).await?;`.
  - `frontend/src-tauri/Cargo.toml:122` — `tokio` `full` feature (`tokio::fs` available).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `manager.rs` prove: (a) `sidecar_path(Path::new("/a/b/app.db"), "-wal") == PathBuf::from("/a/b/app.db-wal")` and `sidecar_path(Path::new("/a/b/app.db"), "-shm") == PathBuf::from("/a/b/app.db-shm")`; (b) a path containing a non-UTF-8 byte is handled by the `Path`-based helper (construct such a path with `OsString::from_vec` on unix and assert the sidecar keeps the parent directory intact) — on non-unix, assert the equivalent `Path` semantics; (c) the incompatibility branch logs at error level — assert via a source-level guard (`include_str!("manager.rs")` contains `log::error!` in the incompatible branch and no longer contains the old `log::warn!("Database migration failed (incompatible existing database)` string; match the message text without the `log::warn!(` prefix, e.g. `grep -c 'Database migration failed (incompatible existing database)'`).
  - `grep -nE '\bstd::fs::remove_file|[^:]\bfs::rename' frontend/src-tauri/src/database/manager.rs` returns NO hits (both replaced by `tokio::fs`; `tokio::fs::rename` contains `fs::rename` but is preceded by `:` so it must not match).
  - `grep -n 'format!("{}-wal"\|format!("{}-shm"' frontend/src-tauri/src/database/manager.rs` returns NO hits.
  - `cargo test -p meetily database::manager` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily database::manager` — the sidecar-path tests pass.
  - Failure: temporarily revert `sidecar_path` to the `format!` version and re-run — the test MUST fail (red→green evidence).
  Evidence: `.omo/evidence/task-28-fork-review-fixes.txt`
  Commit: Y | `fix(database): use async fs and path-based sidecars for db recovery`

- [x] 29. `frontend/src-tauri/src/lib.rs` — bound the exit cleanup, stop the double stat, observe async init failures, and own the cross-todo call sites (3 findings + call-site integration)
  What to do / Must NOT do:
  1. **Unbounded `block_on` on the main thread at exit (medium/performance, lines 922-955).** The `RunEvent::Exit` handler calls `tauri::async_runtime::block_on(async { … db_manager.cleanup().await …; force_shutdown_sidecar().await … })` on the main thread, which can deadlock (the runtime's workers may themselves be blocked) and can hang the app on quit. Replace it with a BOUNDED cleanup: run the cleanup on a dedicated thread/runtime (e.g. `tauri::async_runtime::spawn` + an explicit, short `tokio::time::timeout` for each step, or `std::thread::spawn` + a bounded wait) so the UI thread is never blocked indefinitely. The cleanup must still run to completion for the normal case; only the pathological case is bounded. Record the chosen mechanism in the evidence file.
  2. **Two stat calls per file (low/performance, lines 159-161).** `directory_size_mb`'s inner `dir_size` calls `entry.file_type()` (L157) and then `entry.metadata()` (L160) — two stat calls per file (NOT two `metadata()` calls; each is called once). Capture the metadata ONCE: use `entry.metadata()` once and derive both `is_dir()` and `len()` from it (`Metadata::is_dir()`/`is_file()` and `metadata.len()`), removing the separate `file_type()` call — the current `file_type()` result is only used to decide recursion vs. file size.
  3. **Dropped `JoinHandle`s hide init failures (medium/maintainability, lines 575-580).** `tauri::async_runtime::spawn(...)` is used at L575 (notification manager), L606 (whisper init), L622 (parakeet init), L630 (summary model manager) and the returned `JoinHandle` is dropped, so a failing initialisation is invisible. Capture each `JoinHandle` and observe it: either await the critical ones (whisper/parakeet init) inside `setup` and log a clear error on failure, or store the `JoinHandle`s (or the `Result`s) in the app state so failures can be surfaced. Use `log::error!` for failures; do NOT panic the app on an init failure (the existing code comments state that a failure must not crash the app — keep that behaviour).
  4. **Call-site integration owned by THIS todo (one file per todo).** Two other todos change signatures that are invoked FROM this file: T10 makes `crate::funasr_onnx::engine::initialize_models_directory` return `Result<(), FunasrError>` (call site at L616) and T17 changes `crate::sherpa_onnx::commands::initialize_models_directory` (call site at L619). Because this file is owned by T29, T29 must apply BOTH call-site updates (handle the `Result` — log an error instead of ignoring it) and must keep `lib.rs:917-918` (the `funasr_download_model` / `funasr_cancel_download` registrations) consistent with T14's command signatures. T10 and T17 must NOT edit this file themselves; they record the required change in their own evidence files.

  **Must NOT do:** do **not** change the Tauri command names or the `invoke_handler` registration list (except where a T10/T14/T17 signature change requires it); do **not** change the `RunEvent::Exit` cleanup SEMANTICS (the DB cleanup and the sidecar shutdown must still happen); do **not** remove the `tray-icon` feature or otherwise change product behaviour to work around a Linux `appindicator3` pkg-config gap (that is an ENVIRONMENT gap, not a code defect — record the build error and report it instead); do **not** edit the `205-218` duplicated-notification block (DEFERRED to the upstream PR, T33); do **not** change the `perf_debug!` / `perf_trace!` macros (L6-33); do **not** add a dependency.
  Parallelization: Wave 4 | Blocked by: T1, T10, T17 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/lib.rs:1-4` — the imports (`serde`, `std::sync::atomic::{AtomicBool, Ordering}`, `std::sync::Mutex as StdMutex`).
  - `frontend/src-tauri/src/lib.rs:6-33` — the `perf_debug!` / `perf_trace!` conditional logging macros.
  - `frontend/src-tauri/src/lib.rs:38-58` — the module declarations (`pub mod audio; pub mod funasr_onnx; pub mod sherpa_onnx; pub mod whisper_engine; …`).
  - `frontend/src-tauri/src/lib.rs:142-148` — `struct FunasrModelEntry`.
  - `frontend/src-tauri/src/lib.rs:152-169` — `pub(crate) fn directory_size_mb(path: &std::path::Path) -> f64` and its inner `dir_size` (L155 `read_dir`, L157 `file_type()`, L159-161 `metadata()`).
  - `frontend/src-tauri/src/lib.rs:565` — "Application setup complete".
  - `frontend/src-tauri/src/lib.rs:568-570` — the tray creation.
  - `frontend/src-tauri/src/lib.rs:574-600` — the notification-manager `tauri::async_runtime::spawn`.
  - `frontend/src-tauri/src/lib.rs:603` — `whisper_engine::commands::set_models_directory`.
  - `frontend/src-tauri/src/lib.rs:606-610` — the `whisper_init` spawn (JoinHandle dropped).
  - `frontend/src-tauri/src/lib.rs:613` — the parakeet models directory.
  - `frontend/src-tauri/src/lib.rs:616` — `crate::funasr_onnx::engine::initialize_models_directory(&_app.handle());` (T10's call site).
  - `frontend/src-tauri/src/lib.rs:619` — `crate::sherpa_onnx::commands::initialize_models_directory(&_app.handle());` (T17's call site).
  - `frontend/src-tauri/src/lib.rs:621-626` — the `parakeet_init` spawn (JoinHandle dropped).
  - `frontend/src-tauri/src/lib.rs:628-642` — the summary model-manager spawn (JoinHandle dropped).
  - `frontend/src-tauri/src/lib.rs:657-661` — the `setup`-time `block_on` of `initialize_database_on_startup` (must keep its "must not crash the app" behaviour; it is NOT the flagged site).
  - `frontend/src-tauri/src/lib.rs:678-689` — the `.on_window_event(...)` handler.
  - `frontend/src-tauri/src/lib.rs:690+` — the `.invoke_handler(tauri::generate_handler![…])` list; L917-918 the funasr download/cancel registrations.
  - `frontend/src-tauri/src/lib.rs:920-921` — `.build(...).expect("error while building tauri application")`.
  - `frontend/src-tauri/src/lib.rs:922-955` — the `.run(|_app_handle, event| …)` closure and the `RunEvent::Exit` `block_on` (the flagged site).
  Acceptance criteria (agent-executable):
  - `cargo check -p meetily` exits 0.
  - New `#[cfg(test)]` tests in `lib.rs` prove: (a) a pure helper that classifies a `std::fs::DirEntry`'s metadata ONCE is used — e.g. extract `fn file_len(meta: &std::fs::Metadata) -> u64` and assert it returns the byte length; (b) a structural guard test using `include_str!("lib.rs")` asserting the source contains `tokio::fs` / the chosen bounded-cleanup mechanism and no longer contains the bare `tauri::async_runtime::block_on(async {` inside the `RunEvent::Exit` arm (scope the guard to the `RunEvent::Exit` arm substring rather than the whole file, because the identical literal also exists at `lib.rs:657` for `initialize_database_on_startup` which must be kept); (c) the `setup` block still contains the `initialize_database_on_startup` call (the guard must not have removed it).
  - `grep -n 'tauri::async_runtime::spawn' frontend/src-tauri/src/lib.rs` shows every spawn's `JoinHandle` is either bound to a variable or awaited — no bare `tauri::async_runtime::spawn(async { … });` statement left (the commented-out L647 site is exempt).
  - the `dir_size` body contains exactly ONE stat call (`entry.metadata()`) and `entry.file_type()` no longer appears inside `dir_size` (it may remain elsewhere, e.g. L470 — at 5549e15 `entry.metadata()` already occurs exactly once in the whole file at L160 and `entry.file_type()` twice at L157/L470, so a whole-file count has no red state; scope the guard to the `dir_size` body).
  - `cargo test -p meetily` passes with no new failures.
  QA scenarios (name the exact tool + invocation):
  - Happy: `cargo test -p meetily` — the lib.rs guard tests pass and the whole suite is green.
  - Failure: if the Linux build fails with an `appindicator3` / `ayatana-appindicator3` pkg-config error, capture the exact `pkg-config` output in the evidence file and report it as an ENVIRONMENT gap — do NOT disable the `tray-icon` feature.
  Evidence: `.omo/evidence/task-29-fork-review-fixes.txt`
  Commit: Y | `refactor(app): bound exit cleanup and observe async init failures`

^- [x] 30. `frontend/src/lib/funasr.ts` — normalise invoke rejections, type the invoke generics, and support per-download cancellation (4 findings)
  What to do / Must NOT do:
  1. **Unused exported progress interface (medium/maintainability, lines 3-9).** The review claims `FunasrDownloadProgress` is exported but never consumed. This is NOT REPRODUCIBLE at this commit: it IS consumed — `frontend/src/components/FunasrModelManager.tsx:6` imports it and `:73` uses it as the generic of `listen<FunasrDownloadProgress>('funasr:download-progress', …)`. Verify this by reading those two lines, then record "NOT REPRODUCIBLE at 5549e15 — consumed by FunasrModelManager.tsx:6,73" in the evidence file and change NOTHING here. Do **not** delete the interface (deleting it would break `FunasrModelManager.tsx`, which is OUT OF SCOPE).
  2. **Unhandled `invoke` rejection (high/bug, lines 12-18).** `downloadModel()` / `cancelDownload()` reject with the raw Tauri error and no user-facing message. Wrap the `invoke` calls in `try/catch` and rethrow a normalised `Error` whose message is user-facing and preserves the cause: e.g. `throw new Error(\`Failed to download the FunASR model: ${describe(err)}\`)` / `Failed to cancel the FunASR model download: ${describe(err)}`, where `describe(err)` handles the `unknown` type (`err instanceof Error ? err.message : String(err)`). Export a small pure helper `describeInvokeError(err: unknown): string` so it is unit-testable.
  3. **Missing `invoke` return-type generic (low/maintainability, lines 12-18).** `invoke('funasr_download_model')` resolves to `any`. Add explicit generics: `invoke<string>('funasr_download_model')` (the download id — see item 4) and `invoke<void>('funasr_cancel_download', …)`.
  4. **Stateless cancellation (high/bug, lines 16-18).** `cancelDownload()` has no way to target a specific download. T14 changes the backend to per-download `Arc<AtomicBool>` handles: `funasr_download_model` returns a download id and `funasr_cancel_download` accepts an OPTIONAL id (`None` ⇒ cancel the most recent active download — this preserves the existing single-download UI contract; `Some(id)` ⇒ cancel that download). Implement the matching frontend contract here: `FunasrAPI.downloadModel(): Promise<string>` returns the id; `FunasrAPI.cancelDownload(downloadId?: string): Promise<void>` calls `invoke<void>('funasr_cancel_download', { downloadId: downloadId ?? null })`. The id argument MUST stay OPTIONAL because `FunasrModelManager.tsx` (OUT OF SCOPE) calls `FunasrAPI.cancelDownload()` with no argument — a required parameter would break the TypeScript build. **This contract ONLY holds after T14 lands** — at 5549e15 `funasr_download_model` returns `()` and `funasr_cancel_download()` takes NO parameter, so T30 cannot be landed independently (its Blocked-by already lists T14; do not attempt to verify this contract against the pre-T14 backend).
  5. **Progress-payload mismatch (high/bug, lines 3-9).** The TS `FunasrDownloadProgress` interface (`progress`, `downloaded_mb`, `total_mb`, `speed_mbps`, `status`) does NOT match the Rust `DownloadProgress` struct the backend actually emits (`funasr_onnx/commands.rs:11-18`: `bytes_downloaded: u64`, `total_bytes: u64`, `percent: f64`, `speed_mbps: f64`, `stage: String`) — the consumer reads `event.payload.progress` (FunasrModelManager.tsx:76) and `event.payload.status` (:77), fields the backend never sends. Align the TS interface with the Rust struct: add the Rust-shaped fields (`bytes_downloaded`, `total_bytes`, `percent`, `stage`) and keep the old fields marked `@deprecated` (the consumer is OUT OF SCOPE, so removing them would break the build). Keep the event name `funasr:download-progress` unchanged. Record the field mapping in the evidence file.

  6. **Add the shared `saveFunasrModelSelection` helper (consumed by T34; the two component call sites are fixed in T34, so that finding is counted there, not here).** Export, in addition to item 2's `describeInvokeError`:

     ```ts
     export async function saveFunasrModelSelection(modelName: string): Promise<void> {
       try {
         await invoke<void>('api_save_transcript_config', {
           provider: 'funasr',
           model: modelName,
           apiKey: null,
         });
       } catch (err) {
         throw new Error(`Failed to save the FunASR model selection: ${describeInvokeError(err)}`);
       }
     }
     ```

     The point is that exactly one place knows the invoke name/shape, so `FunasrModelManager.tsx` and `TranscriptSettings.tsx` cannot drift (T34 replaces their local un-awaited / un-caught calls with this helper). Add two assertions to `frontend/tests/lib/funasr.test.ts`: (a) the helper calls the mocked `invoke` exactly once with `('api_save_transcript_config', { provider: 'funasr', model: 'x', apiKey: null })`; (b) when the mocked `invoke` rejects, the helper rejects with a message beginning `Failed to save the FunASR model selection: `. This item adds no new finding to T30's count (the defect it enables fixing is counted in T34).

  **Must NOT do:** do **not** edit `frontend/src/components/FunasrModelManager.tsx` or `frontend/src/components/TranscriptSettings.tsx` (out of scope for T30 — **T34 owns the bounded behavioural edit of exactly those two files**; T30 must not touch either); do **not** change the Tauri event name `funasr:download-progress` (the event NAME stays; the payload shape DOES change — see item 5); do **not** rename the Tauri commands; do **not** remove the exported `FunasrDownloadProgress` interface; do **not** swallow errors — the rethrown error must remain catchable by the existing `try/catch` blocks in the consumer; do **not** add a dependency (use the existing `@tauri-apps/api/core`).
  Parallelization: Wave 5 | Blocked by: T1, T14 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src/lib/funasr.ts:1` — `import { invoke } from '@tauri-apps/api/core';`.
  - `frontend/src/lib/funasr.ts:3-9` — the exported `FunasrDownloadProgress` interface (`progress`, `downloaded_mb`, `total_mb`, `speed_mbps`, `status`).
  - `frontend/src/lib/funasr.ts:11-19` — the `FunasrAPI` class with `downloadModel()` (L12-14) and `cancelDownload()` (L16-18).
  - `frontend/src/components/FunasrModelManager.tsx:6` — `import { FunasrAPI, FunasrDownloadProgress } from '@/lib/funasr';` (proves the interface IS consumed).
  - `frontend/src/components/FunasrModelManager.tsx:73-79` — `listen<FunasrDownloadProgress>('funasr:download-progress', …)` (the event name and payload shape; generic on L73, event-name literal on L74, call closes at L79).
  - `frontend/src/components/FunasrModelManager.tsx:89-114` — `handleDownload` (already wraps `FunasrAPI.downloadModel()` in try/catch + toast).
  - `frontend/src/components/FunasrModelManager.tsx:116-127` — `handleCancelDownload` (calls `FunasrAPI.cancelDownload()` with NO argument — the reason the id parameter must be optional).
  - `frontend/src-tauri/src/funasr_onnx/commands.rs:142-144` and `:251-255` — the backend download/cancel implementation that T14 changes (T14's per-download cancel): `:142-144` is the `funasr_download_model` command and `:251-255` is the `funasr_cancel_download` command (`CANCEL_DOWNLOAD.store(true, Ordering::Relaxed)`).
  - `frontend/src-tauri/src/lib.rs:917-918` — the `funasr_download_model` / `funasr_cancel_download` command registrations (T29 owns this file).
  - `frontend/tests/lib/` — the bun:test test directory where the new test file must live.
  Acceptance criteria (agent-executable):
  - `bun test` (run from `frontend/`) passes with no new failures.
  - A new `frontend/tests/lib/funasr.test.ts` (bun:test) proves, with `@tauri-apps/api/core` mocked via `mock.module`: (a) `downloadModel()` returns the download id string returned by the mocked `invoke`; (b) when `invoke` rejects with `new Error('boom')`, `downloadModel()` rejects with a message that contains `boom` AND a human-readable prefix (assert the message contains "Failed to download the FunASR model"); (c) `cancelDownload()` with no argument calls `invoke` with `{ downloadId: null }`; (d) `cancelDownload('abc')` calls `invoke` with `{ downloadId: 'abc' }`; (e) `describeInvokeError` returns the message for an `Error` and a string for a non-Error value.
  - `grep -n "invoke('funasr_download_model')\|invoke('funasr_cancel_download')" frontend/src/lib/funasr.ts` returns NO hits (every invoke carries an explicit generic).
  - `grep -n 'FunasrDownloadProgress' frontend/src/components/FunasrModelManager.tsx` still returns hits (the interface was NOT deleted — the build must not break).
  - `pnpm exec tsc --noEmit` (run from `frontend/`) exits 0. If the repo has no type-check script, record that and rely on `bun test` + the Next.js build (`pnpm run build`) — record which was used. (Do NOT use `npx tsc` — npx may download a package.)
  QA scenarios (name the exact tool + invocation):
  - Happy: `bun test` from `frontend/` — all five assertions above pass.
  - Failure: temporarily drop the `try/catch` in `downloadModel()` and re-run — the rejection-message test MUST fail (red→green evidence).
  Evidence: `.omo/evidence/task-30-fork-review-fixes.txt`
  Commit: Y | `fix(funasr-ui): normalise download errors and support per-download cancel`

- [x] 31. `frontend/src/hooks/useTranscriptionModels.ts` — fetch the four providers concurrently, de-duplicate the mapping, and surface provider failures (4 findings)
  What to do / Must NOT do:
  1. **Sequential provider fetches (high/performance, lines 58-124).** The hook awaits `whisper_get_available_models`, then `parakeet_get_available_models`, then `api_scan_funasr_models`, then `sherpa_onnx_scan_models` one after another — four round trips in series. Fetch them CONCURRENTLY with `Promise.allSettled` over a typed array of provider descriptors, then flatten the fulfilled values in the ORIGINAL order (whisper, parakeet, funasr, sherpa-onnx) so the model list ordering is unchanged.
  2. **Duplicated fetch-then-filter-map (medium/maintainability, lines 63-124).** The same `.filter(...).map(...)` block is repeated four times with only the command, the filter predicate and the display prefix differing. Extract ONE typed helper, e.g. `async function fetchProviderModels<T>(command: string, provider: ModelOption['provider'], displayPrefix: string, isUsable: (item: T) => boolean): Promise<ModelOption[]>`, and use it for all four providers. Export the helper so it can be unit-tested. Keep the exact display strings unchanged: `🏠 Whisper: `, `⚡ Parakeet: `, `🎯 FunASR (Paraformer): `, `🏷️ Sherpa-ONNX: `.
  3. **Provider failures only console.logged (medium/other, lines 74-124).** When a provider's fetch rejects, the error is only `console.error`-ed, so the user sees a silently missing provider. Collect the failed providers and SURFACE them: add a `failedProviders: string[]` state to the hook, push the provider name whenever its promise rejects, and RETURN it from the hook (an additive change — existing consumers keep compiling). The hook MUST still call `setLoadingModels(false)` when the fetches finish — put it in a `finally` block so a rejected provider can never leave the hook stuck in the loading state.
  4. **Brittle config→provider mapping (medium/maintainability, lines 132-140).** The `configuredMatch` predicate hardcodes four `||` branches with the special case `'localWhisper' → 'whisper'`. Replace it with a lookup map: `const PROVIDER_CONFIG_MAP: Record<string, ModelOption['provider']> = { localWhisper: 'whisper', whisper: 'whisper', parakeet: 'parakeet', funasr: 'funasr', 'sherpa-onnx': 'sherpa-onnx' };` and match with `allModels.find((m) => m.provider === PROVIDER_CONFIG_MAP[configuredProvider] && m.name === configuredModel)`. Export the map.

  **Must NOT do:** do **not** change the hook's existing return fields (`availableModels`, `selectedModelKey`, `setSelectedModelKey`, `loadingModels`, `fetchModels`, `resetSelection`) — only ADD `failedProviders`; do **not** change the `RawModelInfo` / `FunasrModelEntry` / `SherpaOnnxModelEntry` / `ModelOption` interfaces; do **not** change the Tauri command names; do **not** change the model-list ordering; do **not** change the display strings; do **not** remove the `userSelectedRef` manual-selection guard; do **not** edit the consumers of this hook; do **not** add a dependency.
  Parallelization: Wave 5 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src/hooks/useTranscriptionModels.ts:1-2` — the imports (`useState`, `useCallback`, `useRef`, `invoke`).
  - `frontend/src/hooks/useTranscriptionModels.ts:4-15` — `RawModelInfo`, `ModelOption` (the `provider` union `'whisper' | 'parakeet' | 'funasr' | 'sherpa-onnx'`).
  - `frontend/src/hooks/useTranscriptionModels.ts:17-29` — `FunasrModelEntry`, `SherpaOnnxModelEntry`.
  - `frontend/src/hooks/useTranscriptionModels.ts:31-34` — `TranscriptModelConfig`.
  - `frontend/src/hooks/useTranscriptionModels.ts:45-56` — the hook signature, the state, `userSelectedRef` and `setSelectedModelKeyWithTracking`.
  - `frontend/src/hooks/useTranscriptionModels.ts:58-124` — `fetchModels` and the four sequential try/catch blocks (whisper L63-76, parakeet L79-92, funasr L95-108, sherpa L111-124).
  - `frontend/src/hooks/useTranscriptionModels.ts:126` — `setAvailableModels(allModels)`.
  - `frontend/src/hooks/useTranscriptionModels.ts:128-140` — the configured-provider lookup and the brittle 4-branch predicate.
  - `frontend/src/hooks/useTranscriptionModels.ts:142-151` — the default-model selection guarded by `userSelectedRef.current`.
  - `frontend/src/hooks/useTranscriptionModels.ts:153` — `setLoadingModels(false)`.
  - `frontend/src/hooks/useTranscriptionModels.ts:156-159` — `resetSelection`.
  - `frontend/src/hooks/useTranscriptionModels.ts:161-168` — the hook's return object.
  - `frontend/tests/lib/` — the bun:test test directory.
  Acceptance criteria (agent-executable):
  - `bun test` (run from `frontend/`) passes with no new failures.
  - A new `frontend/tests/lib/useTranscriptionModels.test.ts` (bun:test, with `@tauri-apps/api/core` mocked via `mock.module`) proves: (a) the four provider commands are invoked CONCURRENTLY — assert that all four `invoke` mocks were called before any of them resolves (use deferred promises and assert all four were called after a single tick); (b) the flattened model list keeps the provider ORDER whisper → parakeet → funasr → sherpa-onnx; (c) when ONE provider rejects, the other three still populate `availableModels`, `failedProviders` contains the failing provider, and `loadingModels` is `false` afterwards; (d) `PROVIDER_CONFIG_MAP['localWhisper'] === 'whisper'` and the configured-model resolution picks the whisper model when the config says `localWhisper`; (e) the duplicated filter/map block exists exactly once — assert via a source guard that `frontend/src/hooks/useTranscriptionModels.ts` contains exactly ONE occurrence of `.filter(` after the refactor (at 5549e15 it contains 4: L66, L82, L98, L114; the shared helper's filter uses the `isUsable` callback, so the literal `.filter((m) =>` may disappear entirely — count `.filter(`, not `.filter((m) =>`).
  - `grep -c "await invoke" frontend/src/hooks/useTranscriptionModels.ts` returns exactly 4 — ONE `await invoke` inside the shared `fetchProviderModels` helper (called 4× via `Promise.allSettled`), and ZERO `await invoke` at the top level of the hook body outside the helper (no sequential awaits between the four fetches).
  - `grep -n 'failedProviders' frontend/src/hooks/useTranscriptionModels.ts` returns hits in the state declaration, the rejection handler, and the returned object.
  QA scenarios (name the exact tool + invocation):
  - Happy: `bun test` from `frontend/` — the concurrency, ordering, partial-failure and mapping tests pass.
  - Failure: temporarily make one provider reject and REMOVE the `finally { setLoadingModels(false) }` — the test asserting `loadingModels === false` MUST fail (red→green evidence).
  Evidence: `.omo/evidence/task-31-fork-review-fixes.txt`
  Commit: Y | `perf(models): fetch transcription models concurrently and surface failures`

- [ ] 32. `.github/verify-windows-dlls.ps1` — harden the Windows DLL verification script (9 findings) — STATIC VERIFICATION ONLY
  **This todo has NO runtime verification available: `pwsh` / `powershell` are NOT installed on this machine. TDD is therefore NOT applicable here — the todo is verified statically (source-level assertions), and the evidence file MUST state this limitation explicitly and record that execution is deferred to the Windows CI job that runs this script. Do not claim the script was executed.**
  What to do / Must NOT do:
  1. **COM availability check (low/maintainability, lines 45-54).** The `WindowsInstaller.Installer` COM object is used through `InvokeMember` for every call; if the type is not registered on the runner, the COM exception falls into the catch and the whole installer check silently degrades to the `msiexec` fallback. Add an EXPLICIT presence check for the WindowsInstaller COM type at the start of the installer check (e.g. try to create the object and report explicitly that the check was skipped because the COM type is unavailable) so the degradation is visible in the log instead of silent.
  2. **Repo-root resolution (low/maintainability, line 9).** `Resolve-Path (Join-Path $PSScriptRoot '..')` (line 9) assumes the script is exactly one directory deep. Resolve the root with `git -C $PSScriptRoot rev-parse --show-toplevel` and fall back to the current `Resolve-Path` expression when `git` is unavailable; make the resolved root overridable by a parameter/environment variable.
  3. **Full recursive enumeration of the extracted MSI (low/performance, lines 56-58 / 65).** The `msiexec /a` fallback extracts the whole MSI and then `Get-ChildItem -Path $tmp -Recurse -File` (line 58) enumerates every file to build `$names`; the 7-Zip path does the same at line 65. Prefer querying the MSI `File`/`FileName` tables through the already-present WindowsInstaller COM object; in the `msiexec` fallback, list only the EXPECTED file names (e.g. filter for `onnxruntime*.dll`) instead of enumerating everything.
  4. **Non-deterministic 7-Zip invocation (low/maintainability, lines 86-87).** `Get-Command 7z -ErrorAction SilentlyContinue` (line 86) does not set `$LASTEXITCODE`. Invoke the resolved tool deterministically with the call operator (`& $sevenZip.Source …`) and keep the existing `$LASTEXITCODE` check at line 64.
  5. **Over-strict `onnxruntime.dll` count check (low/maintainability, lines 98-100).** `$ortHits.Count -ne 1` (line 99, message at line 100) breaks on versioned names (`onnxruntime64_*.dll`, `onnxruntime-*.dll`). Relax to a documented PATTERN match that still requires exactly ONE match (e.g. `-match '^onnxruntime.*\.dll$'` — a real anchored regex; the naive glob `onnxruntime*.dll` is NOT a valid PowerShell regex for this and would match neither versioned name) and add a comment documenting the single-version delivery invariant.
  6. **Temp directory leaked on failure (medium/maintainability, lines 119-124 / 157).** `$tempDir` is created at L119-120 but only removed on the success path (L157). Wrap the whole `$tempDir` usage in `try { … } finally { Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue }` so `Fail()` and unhandled exceptions also clean up.
  7. **DLL search-order hijacking vector (low/security, line 127 / 134).** Prepending `$tempDir` to `$env:PATH` (line 127) before loading DLLs lets any same-named DLL in that directory win the search. REMOVE the `$env:PATH` prepend and load the DLL with an explicit, restricted search order.
  8. **Fragile DLL search order (low/maintainability, lines 129-141 / 143-146).** Replace the PATH-based load with `LoadLibraryExW` using `LOAD_LIBRARY_SEARCH_APPLICATION_DIR (0x00002000) | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR (0x00001000) | LOAD_LIBRARY_SEARCH_SYSTEM32 (0x00000800)` (= `0x00003800`), implemented via the existing `Add-Type` C# P/Invoke helper (block at L129-141, where `LoadLibraryW` is declared at L134 and `GetLastError` at L139). Keep the absolute path resolution of the loaded library (L143) and document the flag values in a comment.
  9. **Incomplete Win32 error-code hints (low/maintainability, lines 147-152).** Extend the hint map (with the `Fail` message at L152) with `1114` (DLL init failure), `1115` (resource load cycle) and `1261` (image cannot have the same TEB size).

  **Must NOT do:** do **not** change the script's purpose, its inputs, its exit codes, or the CI job that invokes it; do **not** remove the `msiexec /a` fallback entirely (keep it as the degraded path, just make it cheaper); do **not** change the expected DLL names the script looks for; do **not** disable the check when the COM type is missing — REPORT the degradation; do **not** add a runtime PowerShell dependency to the repo; do **not** touch other workflow files.
  Parallelization: Wave 5 | Blocked by: T1 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `.github/verify-windows-dlls.ps1:9` — the repo-root resolution (`$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path`).
- **Line-number provenance:** every line number below was verified against the 159-line `.github/verify-windows-dlls.ps1` at commit `5549e15`. The file is **byte-identical** in the main checkout and in the worktree (verified with `diff`), so the same numbers apply in both places. If a number no longer matches, locate the construct by its quoted text rather than by line number.
  - `.github/verify-windows-dlls.ps1:45-54` — the COM-object extraction path (`OpenView` at L45, the `StringData` read at L50) and the `Write-Warning` fallback at L54.
  - `.github/verify-windows-dlls.ps1:56-58` — the `msiexec /a` administrative-extract fallback (`Start-Process msiexec` at L56) plus the recursive enumeration at L58; the extraction temp dir is removed at L70.
  - `.github/verify-windows-dlls.ps1:64-65` — the `$LASTEXITCODE` exit-code check (keep it) and the 7-Zip enumeration at L65.
  - `.github/verify-windows-dlls.ps1:78` — the installer scan (`Get-ChildItem -Path $targetDir -Recurse -File -Include *.exe, *.msi`).
  - `.github/verify-windows-dlls.ps1:86-87` — `Get-Command 7z -ErrorAction SilentlyContinue` and the `if (-not $sevenZip)` guard (the `& $sevenZip.Source …` invocation follows).
  - `.github/verify-windows-dlls.ps1:98-100` — the `$ortHits` filter (L98), the `$ortHits.Count -ne 1` check (L99) and its `Fail` message (L100).
  - `.github/verify-windows-dlls.ps1:119-124` — the `$tempDir` lifecycle (created L119-120, DLLs copied L122-124); the success-path cleanup is at L157.
  - `.github/verify-windows-dlls.ps1:127` — the `$env:PATH` prepend (`$env:PATH = "$tempDir;$env:PATH"`) — **this line must be deleted**.
  - `.github/verify-windows-dlls.ps1:129-141` — the `Add-Type` P/Invoke block (`LoadLibraryW` declared at L134, `GetLastError` at L139) — **replace `LoadLibraryW` with `LoadLibraryExW` here**.
  - `.github/verify-windows-dlls.ps1:143-146` — the absolute path resolution (`$appLibFull` at L143), the load call at L144 and `GetLastError` at L146.
  - `.github/verify-windows-dlls.ps1:147-152` — the Win32 error-code hint map and the `Fail` message to extend.
  Acceptance criteria (agent-executable):
  - The evidence file explicitly states: "No PowerShell runtime is available in this environment (`pwsh` and `powershell` are not installed) — this todo is verified STATICALLY only; execution is deferred to the Windows CI job." (this sentence must be present verbatim in the evidence file).
  - Static assertions (all via `grep -n` over `.github/verify-windows-dlls.ps1`, each recorded in the evidence file): (a) a `try {` … `finally {` pair wraps the `$tempDir` block; (b) `Remove-Item -Path $tempDir` appears inside the `finally` block; (c) the file NO LONGER contains `$env:PATH =` (the hijack vector is gone); (d) the file contains `LoadLibraryExW` and the flag constant `0x00003800` (or the three named flags); (e) the file contains an explicit WindowsInstaller COM presence check; (f) `$ortHits.Count -ne 1` is gone and a pattern-based check (`onnxruntime*.dll`) is present; (g) the hint map contains `1114`, `1115` and `1261`; (h) `git -C $PSScriptRoot rev-parse --show-toplevel` is present; (i) `& $sevenZip.Source` is present.
  - `grep -c 'Get-ChildItem .*-Recurse' .github/verify-windows-dlls.ps1` is 0 (at 5549e15 it is 4 — L58, L65, L78, L108 — so this criterion has a real red state; the literal `Get-ChildItem -Recurse` never occurs in the file, so the old pattern was vacuously green).
  - The script's exit codes and the names of the DLLs it verifies are unchanged (diff the file and confirm only the nine findings' areas changed).
  QA scenarios (name the exact tool + invocation):
  - Happy: all static assertions above pass and the diff touches only the flagged regions.
  - Failure: temporarily remove the `finally` cleanup and confirm the static assertion (b) fails — i.e. the acceptance check is real and not vacuous (record the failing grep output as red→green evidence).
  - Failure: record that runtime execution is impossible here; do NOT claim the script was run.
  Evidence: `.omo/evidence/task-32-fork-review-fixes.txt`
  Commit: Y | `ci(windows): harden the DLL verification script`

- [x] 34. `frontend/src/components/FunasrModelManager.tsx` — persist the FunASR model selection before notifying the parent, and stop leaving unhandled save rejections in `TranscriptSettings.tsx` (2 findings)
  **PRIMARY file: `frontend/src/components/FunasrModelManager.tsx`. Satellite edit (enumerated, must appear in the evidence file): `frontend/src/components/TranscriptSettings.tsx` — the two model-select handlers.**
  What to do / Must NOT do:
  1. **`selectModel` must persist first, notify second, and only claim success when the save succeeded.** Current code (`frontend/src/components/FunasrModelManager.tsx:129-149`) calls `onModelSelect(modelName)` BEFORE awaiting the save, and calls `toast.success(...)` after a `try/catch` that only `console.error`s — so the user is told "Switched to X" even when persistence failed. Replace the body with:

     ```ts
     const selectModel = useCallback(async (modelName: string) => {
       if (autoSave) {
         try {
           await saveFunasrModelSelection(modelName);
         } catch (err) {
           toast.error(err instanceof Error ? err.message : 'Failed to save the FunASR model selection');
           return;
         }
       }
       if (onModelSelect) {
         onModelSelect(modelName);
       }
       toast.success(`Switched to ${modelName}`, { duration: 3000 });
     }, [onModelSelect, autoSave]);
     ```

     Import `saveFunasrModelSelection` from `@/lib/funasr` (it is added by T30 item 6). On failure the parent is NOT notified and the UI does not claim success — that is the intended behaviour change.
  2. **Remove the now-unused `invoke` import** at `frontend/src/components/FunasrModelManager.tsx:3` IF this was its only use. Verify first: `grep -n 'invoke(' frontend/src/components/FunasrModelManager.tsx` — after item 1 the only remaining `invoke` references must be none (the download path already goes through `FunasrAPI`). Keep the `listen` import (L2) and the `{ FunasrAPI, FunasrDownloadProgress }` import (L6).
  3. **`TranscriptSettings.tsx`: await + catch both model-select saves.** `handleFunasrModelSelect` (L102-L112) and `handleSherpaOnnxModelSelect` (L114-L124) both call `invoke('api_save_transcript_config', …)` fire-and-forget (no `await`, no `.catch`) and then `onModelSelect()` unconditionally. Make each handler `async`, and:

     ```ts
     const handleFunasrModelSelect = async (modelName: string) => {
       setTranscriptModelConfig({ ...transcriptModelConfig, provider: 'funasr', model: modelName });
       try {
         await saveFunasrModelSelection(modelName);
       } catch (err) {
         toast.error(err instanceof Error ? err.message : 'Failed to save the FunASR model selection');
       }
       if (onModelSelect) {
         onModelSelect();
       }
     };
     ```

     For `handleSherpaOnnxModelSelect` there is no shared helper (T30's helper is FunASR-specific) — keep the direct `invoke` but wrap it: `try { await invoke('api_save_transcript_config', { provider: 'sherpa-onnx', model: modelName, apiKey: null }); } catch (err) { toast.error(…); }`. Add `import { toast } from 'sonner';` (the file does not import it today) and keep the `invoke` import (still used by the sherpa handler and any other call sites — verify with `grep -n 'invoke(' frontend/src/components/TranscriptSettings.tsx`).
  4. **Do not change any other behaviour** of these two components: no JSX/layout changes, no prop-signature changes, no renaming, no new dependencies, no switching `setTranscriptModelConfig` to a functional update, no touching `handleDownload` (which already try/catches `FunasrAPI.downloadModel()`).
  5. **Record the test-coverage reality in the evidence file, verbatim:** the frontend test harness is `bun:test` with `mock.module` and does NOT include `@testing-library/react` or `jsdom` (`frontend/package.json:103` only has `@types/bun`), so these two components cannot be rendered in a test. The red→green pair for this todo therefore lives in `frontend/tests/lib/funasr.test.ts` (T30 item 6 asserts `saveFunasrModelSelection` calls `invoke` once with the exact arguments and rejects with the `Failed to save the FunASR model selection: ` prefix); the component layer is verified by the static assertions below plus `pnpm exec tsc --noEmit` and a recorded manual-QA note (no display on this host — defer the click-through, state that plainly).

  **Must NOT do:** do **not** edit any other file under `frontend/src/components/**` (this todo's components exception is exactly these two files); do **not** change `FunasrModelManager`'s props or its `listen<FunasrDownloadProgress>` handler (T30's payload work depends on it); do **not** delete the `FunasrDownloadProgress` import; do **not** swallow the save error silently (it must reach `toast.error`); do **not** add `@testing-library/react`/`jsdom` or any other dependency; do **not** re-order `setTranscriptModelConfig` (keep it before the save so the UI reflects the choice immediately — the error toast is the rollback signal).
  Parallelization: Wave 5 | Blocked by: T1, T30 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src/components/FunasrModelManager.tsx:1-6` — imports; L3 `import { invoke } from '@tauri-apps/api/core';` becomes unused after item 1; L5 `import { toast } from 'sonner';`; L6 `import { FunasrAPI, FunasrDownloadProgress } from '@/lib/funasr';`.
  - `frontend/src/components/FunasrModelManager.tsx:129-149` — the defective `selectModel` (parent callback before persistence; success toast after a swallowing catch).
  - `frontend/src/components/FunasrModelManager.tsx:73-79` — `listen<FunasrDownloadProgress>('funasr:download-progress', …)`; must stay untouched.
  - `frontend/src/components/TranscriptSettings.tsx:1-11` — imports; L1 `useState/useEffect`, L2 `import { invoke } from '@tauri-apps/api/core';`, no `toast` import.
  - `frontend/src/components/TranscriptSettings.tsx:102-112` — `handleFunasrModelSelect` (fire-and-forget `invoke`, then `onModelSelect()`).
  - `frontend/src/components/TranscriptSettings.tsx:114-124` — `handleSherpaOnnxModelSelect` (same pattern, `provider: 'sherpa-onnx'`).
  - `frontend/src/lib/funasr.ts` — `saveFunasrModelSelection` and `describeInvokeError` are added there by T30 items 2 and 6.
  - `frontend/tests/lib/funasr.test.ts` — the helper's red→green tests (T30).
  - `frontend/package.json:103` — only `@types/bun`; no DOM/component test stack (why item 5 records the limitation).
  - `/home/tango/repo/meetily/.omo/evidence/w2-core-scan_v3.json` — the scan comments this todo addresses: `FunasrModelManager.tsx:129-146` (parent callback ordering) and `TranscriptSettings.tsx:102-112` (fire-and-forget save).
  Acceptance criteria (agent-executable):
  - `grep -n "invoke('api_save_transcript_config'" frontend/src/components/FunasrModelManager.tsx` returns NO hits, and `grep -n 'saveFunasrModelSelection' frontend/src/components/FunasrModelManager.tsx` returns hits at the import and inside `selectModel`.
  - `grep -c 'invoke(' frontend/src/components/FunasrModelManager.tsx` returns `0` (the import was removed in item 2); `grep -n 'listen<FunasrDownloadProgress>' frontend/src/components/FunasrModelManager.tsx` still returns exactly one hit.
  - `grep -n -A2 'api_save_transcript_config' frontend/src/components/TranscriptSettings.tsx` shows both call sites inside a `try` block preceded by `await`; `grep -n "toast from 'sonner'" frontend/src/components/TranscriptSettings.tsx` returns one hit.
  - `pnpm exec tsc --noEmit` (from `frontend/`) exits 0 — no unused-import or `Promise`-return errors. (Never `npx tsc`.)
  - `bun test` from `frontend/` is green, including `frontend/tests/lib/funasr.test.ts`.
  - `pnpm exec eslint src/components/FunasrModelManager.tsx src/components/TranscriptSettings.tsx` (from `frontend/`) exits 0 (proves no unused `invoke` import was left behind).
  QA scenarios (name the exact tool + invocation):
  - Happy (automated): `cd frontend && bun test tests/lib/funasr.test.ts` — the T30 helper assertions pass (exact invoke args; rejection prefix).
  - Failure (automated): the red→green pair is in T30 item 6 — drop the `try/catch` from `saveFunasrModelSelection` and the rejection-message assertion MUST fail. For the component layer, the failing state is a static one: before item 1, `grep -c 'invoke(' FunasrModelManager.tsx` is ≥1 and shows the parent callback above the `await`; after item 1 it is 0. Record both greps under `RED:` and `GREEN:`.
  - Manual (deferred, no display on this host): select a FunASR model with the backend stopped → expect an error toast and NO "Switched to …" success toast; record as deferred with this exact command-free description.
  Evidence: `.omo/evidence/task-34-fork-review-fixes.txt`
  Commit: Y | `fix(funasr-ui): persist model selection before notifying the parent`

- [ ] 33. Upstream deferral PR — package the 10 deferred upstream-origin findings as a standalone branch + PR body (10 findings, no new source file in the fork branch)
  **Scope of this todo: it produces a BRANCH + COMMITS + a PR description — it does NOT push, does NOT open a PR, and does NOT touch the fork branch. The user pushes and opens the PR.**
  What to do / Must NOT do:
  1. Create a SEPARATE worktree `/home/tango/repo/meetily/.worktrees/upstream-deferrals` on a NEW branch `fix/upstream-review-deferrals` based on `upstream/main` (the upstream base commit). **Never commit anything from this todo onto `fix/fork-review-fixes`.** The fork branch must remain free of upstream-origin changes.
  1a. **Verify the upstream base is the reviewed commit.** `upstream/a2cb62e` is NOT a resolvable ref (it aborts with "ambiguous argument"), so use `upstream/main` and assert the SHA: `git -C /home/tango/repo/meetily rev-parse upstream/main | grep -q '^a2cb62e' || { echo "FAIL: upstream/main is not the reviewed upstream base a2cb62e"; exit 1; }`. If this fails, **stop and report** — do not create the upstream worktree on an unexpected base.
  2. Fix the 10 deferred findings IN THE UPSTREAM WORKTREE ONLY:
     - `frontend/src-tauri/src/audio/import.rs:200-208` — blocking `std::fs::File::open` inside async paths ⇒ use `tokio::fs::File`.
     - `frontend/src-tauri/src/audio/import.rs:752-768` — use `pool.begin()` instead of `acquire` + `begin`.
     - `frontend/src-tauri/src/audio/retranscription.rs:572-591` — the DB commit happens BEFORE `write_transcripts_json` (L591) / `write_retranscription_metadata` (L606), so a crash leaves stale JSON ⇒ write the JSON/metadata first and commit the DB last (or make the sequence idempotent and recoverable).
     - `frontend/src-tauri/src/audio/retranscription.rs:572-591` (performance duplicate entry) — the same site's cost/latency aspect; resolve it together with the ordering fix above and note the merge in the evidence file.
     - `frontend/src-tauri/src/audio/retranscription.rs:751-758` — the hardcoded `id = '1'` query with a silent fallback to `DEFAULT_WHISPER_MODEL`, and the `error!` inside `ok_or_else` that floods the logs.
     - `frontend/src-tauri/src/audio/retranscription.rs:751-759` — the adjacent variant of the same defect (same function, second entry) ⇒ resolve together.
     - `frontend/src-tauri/src/audio/transcription/engine.rs:60-61` — the redundant `app.clone()`.
     - `frontend/src-tauri/src/database/repositories/setting.rs:194-204` — the hardcoded `provider='parakeet'` / `model=DEFAULT_PARAKEET_MODEL` INSERT that ignores the caller's parameters.
     - `frontend/src-tauri/src/database/repositories/setting.rs:303-307` — `serde_json::Error` converted into `sqlx::Error::Protocol` loses the source ⇒ preserve the source chain.
     - `frontend/src-tauri/src/lib.rs:205-218` — the duplicated notification code between `start_recording` and `start_recording_with_devices_and_meeting` ⇒ extract one shared helper.
  3. Write the PR body to `.omo/evidence/task-33-upstream-pr-body.md`. It MUST contain: (a) a one-paragraph problem statement; (b) a per-finding table — finding id, file:line, what was wrong, what changed; (c) the PROVENANCE section proving each finding is a FORK-ORIGIN regression and not upstream behaviour (use the recorded provenance data at `/tmp/opencode/provenance.csv` / `/tmp/opencode/provenance-summary.md`, and re-derive it if those files are gone: for each of the 10 findings, show the upstream file's corresponding code and the fork's diff); (d) the test/verification commands and their output; (e) an explicit note that the other 1355 upstream findings were NOT touched.
  4. Re-run the whitespace-stripped substring provenance test for ALL 161 findings (151 in-fork + 10 deferred) to prove that no out-of-scope upstream finding was accidentally modified.
  5. Verify the upstream worktree builds and tests green: `cargo check -p meetily` and `cargo test -p meetily` (run in `/home/tango/repo/meetily/.worktrees/upstream-deferrals`).

  **Must NOT do:** do **not** push the branch; do **not** open the PR (the user does); do **not** rebase, amend, force-push, or otherwise rewrite git history; do **not** commit any upstream-origin change onto `fix/fork-review-fixes`; do **not** touch any of the 1355 out-of-scope upstream findings; do **not** change dependency versions; do **not** modify the archived `backend/` directory; do **not** include the fork-only fixes (T2-T32) in the upstream branch.
  **Explicitly out of scope (recorded, not fixed here):** `.github/workflows/**` action pinning. All four workflows use mutable major-version tags (`actions/checkout@v4`, `pnpm/action-setup@v4`, `actions/setup-node@v4`, `actions/cache@v4`, `dtolnay/rust-toolchain@stable`, `swatinem/rust-cache@v2`, `humbletim/install-vulkan-sdk@v1.2`, `digicert/ssm-code-signing@v1.1.1`, `tauri-apps/tauri-action@v0`, `actions/upload-artifact@v4`). That is pre-existing upstream style, not a fork regression, and the fork's CI edits were limited to Windows CPU builds — SHA-pinning belongs in a separate supply-chain hardening PR, not in this plan.
  Parallelization: Wave 5 | Blocked by: T1, T29 | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `frontend/src-tauri/src/audio/import.rs:200-208` — the blocking `std::fs::File::open` in async paths.
  - `frontend/src-tauri/src/audio/import.rs:752-768` — the `acquire` + `begin` pattern to replace with `pool.begin()`.
  - `frontend/src-tauri/src/audio/retranscription.rs:572` (DB commit), `:591` (`write_transcripts_json`), `:606` (`write_retranscription_metadata`) — the ordering defect.
  - `frontend/src-tauri/src/audio/retranscription.rs:751-758` and `:751-759` — the hardcoded `id = '1'` query, the silent `DEFAULT_WHISPER_MODEL` fallback, and the `error!` inside `ok_or_else` (the same pattern also appears in `get_configured_parakeet_model` around L829).
  - `frontend/src-tauri/src/audio/transcription/engine.rs:60-61` — the redundant `app.clone()`.
  - `frontend/src-tauri/src/database/repositories/setting.rs:194-204` — the hardcoded INSERT.
  - `frontend/src-tauri/src/database/repositories/setting.rs:303-307` — the lossy `sqlx::Error::Protocol` conversion.
  - `frontend/src-tauri/src/lib.rs:205-218` — the duplicated notification code.
  - `/tmp/opencode/provenance.csv` and `/tmp/opencode/provenance-summary.md` — the recorded provenance data (re-derive if absent).
  - The upstream base commit `upstream/main`.
  Acceptance criteria (agent-executable):
  - `git -C /home/tango/repo/meetily/.worktrees/upstream-deferrals log --oneline -1` shows the branch `fix/upstream-review-deferrals` based on `upstream/main`; `git -C /home/tango/repo/meetily/.worktrees/fork-review-fixes log --oneline` shows NO upstream-origin commit (the fork branch is untouched by this todo).
  - `git -C /home/tango/repo/meetily rev-parse upstream/main` prints `a2cb62e827da7ef59f65064c97233efb2313878e` — the reviewed upstream base; the step-1a guard passed before the worktree was created.
  - `git -C /home/tango/repo/meetily/.worktrees/upstream-deferrals diff --name-only upstream/main` lists ONLY the files that carry the 10 deferred findings (`audio/import.rs`, `audio/retranscription.rs`, `audio/transcription/engine.rs`, `database/repositories/setting.rs`, `src-tauri/src/lib.rs`) — no other file.
  - `cargo check -p meetily` and `cargo test -p meetily` exit 0 in the upstream worktree (record the output; the known pre-existing failures listed in the Scope section may still appear — they must be identical to the baseline).
  - `.omo/evidence/task-33-upstream-pr-body.md` exists and contains all five required sections (problem statement, per-finding table, provenance, verification, out-of-scope statement).
  - The whitespace-stripped substring provenance test over all 161 findings is re-run and its output is pasted into the evidence file.
  QA scenarios (name the exact tool + invocation):
  - Happy: `git diff --name-only upstream/main` in the upstream worktree lists exactly the five expected files, and the fork branch's log contains no upstream-origin commit.
  - Failure: `git -C /home/tango/repo/meetily/.worktrees/fork-review-fixes status --porcelain` MUST be empty for the deferred files — if any of the 10 findings was accidentally committed on the fork branch, this todo FAILED: remove it and record the incident.
  Evidence: `.omo/evidence/task-33-fork-review-fixes.txt` (+ `.omo/evidence/task-33-upstream-pr-body.md`)
  Commit: Y | `fix(upstream): address the deferred upstream-origin review findings`




## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
> Every check below is agent-executed with zero human intervention. Each verifier writes its full report to `.omo/evidence/f<N>-<name>.md` in the worktree and prints a final `VERDICT: PASS` / `VERDICT: FAIL` line. A FAIL blocks completion: fix the cause, re-run the affected todo, then re-run the whole wave.

- [ ] F1. Plan compliance audit → `.omo/evidence/f1-plan-compliance.md`
  - [ ] F1.1 Read `.omo/plans/fork-review-fixes.md`; assert every todo T1–T33 is marked `[x]` and list any that is not.
  - [ ] F1.2 Assert one evidence file exists per todo: `.omo/evidence/task-<N>-fork-review-fixes.txt` for N = 1..33 (T33 may additionally have `task-33-upstream-pr-body.md`). List every missing file.
  - [ ] F1.3 Assert every evidence file contains a **red→green** record: a failing-test output captured BEFORE the fix (with the failure reason) and a passing-test output AFTER it. A todo whose evidence file lacks both is FAIL.
  - [ ] F1.4 Cross-check coverage against the authoritative finding list `.omo/evidence/fork-review-inscope.txt` (159 findings in the file + 2 in T34 = **161**): exactly 151 must be addressed on the fork branch and exactly 10 must be deferred to T33. The raw list carries **159** headers = **157 headers in fork-origin files** + **2 in `setting.rs`** (a wholly upstream-origin file, excluded from the Scope table). Of the 157, **149** are the in-fork findings in the list and the remaining **8** are T33's deferred findings that live in fork-origin files (`import.rs` ×2, `retranscription.rs` ×4, `transcription/engine.rs` ×1, `lib.rs` ×1); T34 contributes the 2 additional findings. **Subtract the 10 deferred findings listed verbatim in Todo 33 from the raw 159, then add T34's 2** (159 − 10 + 2 = 151), build the per-file count table from those **151 in-fork headers only**, and diff it against the `Per-file finding counts` table in `## Scope` — any mismatch is FAIL. Additionally assert that the 8 deferred headers that sit in fork-origin files ARE present in the raw list (they are excluded from the Scope-table comparison, not missing) and that 149 + 2 + 8 + 2 = 161.
  - [ ] F1.5 Assert the dependency matrix was respected: for every todo, its commit timestamp/position is after every todo listed in its `Depends on` column. Report any inversion.
  - [ ] F1.6 Assert the special obligations were honoured, by grepping the evidence files:
        - T32's evidence contains, verbatim: `No PowerShell runtime is available in this environment (`pwsh` and `powershell` are not installed) — this todo is verified STATICALLY only; execution is deferred to the Windows CI job.`
        - T30's evidence contains the label `NOT REPRODUCIBLE` (the `FunasrDownloadProgress` finding) and states that the interface was NOT deleted.
        - T14's evidence contains the label `COMMAND CONTRACT (consumed by T30)` with the final `funasr_download_model` / `funasr_cancel_download` signatures.
        - T10's and T17's evidence contain the `REQUIRED lib.rs CHANGE` heading (T10's must name the change T29 applied; T17's must state that the required change is zero).
        - T16's evidence shows the `current_model()` test (red→green) and T24's evidence shows it consuming `SherpaOnnxEngine::current_model()` rather than the literal `"sensevoice"`.
  - [ ] F1.7 Assert each **code-touching** todo produced exactly ONE conventional commit with the exact subject line specified in that todo's `Commit:` line, and that the commit touches only that todo's file (plus its test file / evidence file). **T1 is exempt** — its `Commit:` line is `N` (setup only, no commit). **T33's single commit lives on the separate upstream branch** `fix/upstream-review-deferrals`, not on `fix/fork-review-fixes`, and by design touches the 5 files carrying the 10 deferred findings. The fork branch therefore carries **32** commits (T2–T34, excluding T33) and the upstream branch **1** (T33) — **33** in total, not 34.
  - [ ] F1.8 Assert the plan file itself was not rewritten mid-flight: the todo list is append-only, the header order is unchanged, and `## TL;DR (For humans)`, `## Commit strategy` and `## Success criteria` are non-empty.
  - Verdict: **PASS** only if every sub-check passes. Otherwise **FAIL** with the exact list of violations.

- [ ] F2. Code quality review → `.omo/evidence/f2-code-quality.md`
  - [ ] F2.1 `cargo check -p meetily` (worktree root) → exit 0, no errors.
  - [ ] F2.2 `cargo clippy -p meetily --all-targets` → zero new findings versus `.omo/evidence/t1-clippy-baseline.txt` (both files exist). A new finding attributable to this work is a FAIL (fix it in that todo's file; do not silence it with `#[allow]`).
  - [ ] F2.3 `cargo test -p meetily` → the set of failing tests MUST be identical to the T1 baseline (`test_calculate_buffer_timeout_bluetooth` and the ONNX `munmap_chunk` corruption in the full run are pre-existing — see `### Known pre-existing issues`). Any NEW failure is a FAIL.
  - [ ] F2.4 `bun test` from `frontend/` → no NEW failures versus the T1 baseline, compared **per test file**: for every test file present in the T1 baseline, the set of failing test names must be identical (a whole-suite result-set diff is not a reliable criterion — bun's `mock.module` registrations are global and leak across files, so one file's mock can change another file's result; isolate the mocks per file with `mock.restore()` where a file needs them); the new `frontend/tests/lib/funasr.test.ts` and `frontend/tests/lib/useTranscriptionModels.test.ts` must be present and green.
  - [ ] F2.5 `cd frontend && pnpm run lint` → zero new errors versus `.omo/evidence/t7-eslint-baseline.txt` (both files exist — the baseline is produced by T7, because the `eslint` binary does not exist before T7 installs it); no `any` introduced by the TS changes; no `console.log` left behind in `useTranscriptionModels.ts`.
  - [ ] F2.6 `bash -n frontend/build-gpu.sh` → exit 0 (shellcheck is not installed; `bash -n` is the available check).
  - [ ] F2.7 Rust quality assertions, verified by grep over the diff:
        - No `unwrap()` / `expect()` added in non-test code by this work (existing ones unchanged).
        - No `unsafe` block added.
        - No `panic!`/`unreachable!` added in library code.
        - No `clippy::allow`, `#[allow(dead_code)]`, or `#[allow(unused)]` added to hide a finding.
        - Every new public item (`pub fn`, `pub struct`, `pub enum`) carries a doc comment.
        - No `OnceLock`/global mutable state introduced without the poisoning/TOCTOU handling the finding asked for.
  - [ ] F2.8 Confirm the deliberate non-changes were honoured: `frontend/src-tauri/src/funasr_onnx/engine.rs:54` still contains the transcript-echo log (T10 must not have touched it), and `FunasrEngine` still has exactly ONE `current_model()`.
  - Verdict: **PASS** only if every sub-check passes. Otherwise **FAIL** with the exact list of violations.

- [ ] F3. Real manual QA → `.omo/evidence/f3-manual-qa.md`
  - [ ] F3.1 Report the host capability first: `echo $DISPLAY`, `command -v pwsh powershell shellcheck`, `pkg-config --exists libappindicator3-0.1 && echo present`. Record honestly what is and is not available — do NOT claim a GUI session or a PowerShell run that did not happen.
  - [ ] F3.2 Production build of the web layer: `pnpm run build` from `frontend/` → exit 0 (Next.js static export). This proves the T30/T31 TypeScript changes compile in the real production path, not only under `bun test`.
  - [ ] F3.3 Production build of the native layer: `cargo build -p meetily` → exit 0. If it stops on the missing Linux `libappindicator3-dev` pkg-config entry, record that verbatim as an **environment gap** (documented in `### Known pre-existing issues`), not a product defect, and verify the build with the tray feature disabled (`cargo build -p meetily --no-default-features` if that is how the manifest allows it) to prove the code itself compiles.
  - [ ] F3.4 Behavioural run of the affected subsystems, as real executables, not mocks:
        - `cargo test -p meetily funasr -- --nocapture` → the per-download cancel contract tests (T14) pass.
        - `cargo test -p meetily sherpa -- --nocapture` → the `current_model()` resolution test (T16) passes.
        - `cargo test -p meetily retranscription -- --nocapture` → T26's tests pass.
        - `bun test tests/lib/funasr.test.ts` and `bun test tests/lib/useTranscriptionModels.test.ts` from `frontend/` → T30/T31's tests pass.
  - [ ] F3.5 Verify the two user-visible symptoms are actually gone at the unit level: a failed FunASR download surfaces a human-readable message (T30) and the model list loads all four providers concurrently with a visible failure list instead of silently dropping a provider (T31). Quote the passing assertions.
  - [ ] F3.6 Record what could NOT be exercised on this host (GUI interaction, Windows/PowerShell runtime, macOS/Linux audio devices) and the exact command a human must run in those environments to close the gap. Do not claim coverage you do not have.
  - Verdict: **PASS** when every runnable check passes and every non-runnable check is explicitly and honestly recorded as deferred with its closing command. **FAIL** on any real failure or on any unsupported success claim.

- [ ] F4. Scope fidelity → `.omo/evidence/f4-scope-fidelity.md`
  - [ ] F4.1 `git -C .worktrees/fork-review-fixes diff --name-only 5549e15..HEAD` → the changed set must be EXACTLY: the 33 in-scope files from the `## Scope` table, plus the new files `frontend/src-tauri/src/build_env.rs` (created by T3/T4 — the relocated build-script test helpers), `frontend/tests/lib/funasr.test.ts`, `frontend/tests/lib/useTranscriptionModels.test.ts`, and `frontend/tests/scripts/build-gpu-path-resolution.test.sh`, plus `frontend/pnpm-lock.yaml` (expected only because of T7's `eslint` devDependency), plus files under `.omo/`. Any other path is a FAIL.
  - [ ] F4.2 Assert forbidden paths are untouched: `frontend/src-tauri/src/database/repositories/setting.rs`, `backend/**`, `frontend/src-tauri/src/main.rs`, the workspace-root `Cargo.toml`, `frontend/src/components/**` **except** the two files T34 owns (`frontend/src/components/FunasrModelManager.tsx`, `frontend/src/components/TranscriptSettings.tsx`), `frontend/src/app/**`. No model weights may be added to the repository: the changed-file list contains **no** `*.onnx`, `*.mvn`, `*.gguf` or other model-weight file, and no new `models/` directory was created. (`frontend/models/**` is **not** a valid check — that directory does not exist in this repository, so the assertion would be vacuous.)
  - [ ] F4.3 Dependency-fidelity assertions over the diff of `frontend/src-tauri/Cargo.toml`, `llama-helper/Cargo.toml`, `frontend/package.json`:
        - `ort` is still `2.0.0-rc.10` and the Windows `load-dynamic` feature is still present; ort's default features are NOT re-enabled.
        - `llama-cpp-2` / `llama-cpp-sys-2` are still `=0.1.146`.
        - The 5 `pnpm.overrides` prosemirror pins are byte-identical to the baseline (the diff on those lines is empty).
        - The only dependency ADDITION anywhere is the `eslint` devDependency from T7; no other version string changed.
  - [ ] F4.4 Assert the platform-specific structure survived: the Windows sherpa `default-features=false, features=["shared"]` split vs the non-Windows static build is intact, and the DirectML execution provider still exists and is now gated by `target_os`.
  - [ ] F4.5 Re-run the whitespace-stripped substring provenance test for ALL 161 findings against the fork branch: the 151 fork-origin findings must no longer match their original source text, and the 10 upstream-origin findings must STILL match byte-for-byte (proving they were not modified).
  - [ ] F4.6 Assert the upstream branch is cleanly separated: `git -C .worktrees/upstream-deferrals diff --name-only upstream/main` lists only the five files carrying the 10 deferred findings, and `git -C .worktrees/fork-review-fixes diff --name-only 5549e15..HEAD` contains NONE of the 10 deferred findings' code changes.
  - [ ] F4.7 Git-hygiene assertions: linear history with **one commit per code-touching todo** — 32 on `fix/fork-review-fixes` (T2–T34) and 1 on `fix/upstream-review-deferrals` (T33), T1 contributing none (no squashed mega-commit) — no `rebase`/`amend`/`reset --hard` in the reflog, and **nothing pushed** — the remote-tracking refs for both branches are unchanged/absent.
  - [ ] F4.8 Assert the main checkout at `/home/tango/repo/meetily` is byte-identical to its pre-work state outside `.omo/` (`git -C /home/tango/repo/meetily status --porcelain` shows only `.omo/` paths).
  - Verdict: **PASS** only if every sub-check passes. Otherwise **FAIL** with the exact list of violations.

## Commit strategy

**Model — one conventional commit per code-touching todo (T2–T34 on the fork branch, T33 on the upstream branch; T1 commits nothing), linear history, nothing pushed.**

| Property | Decision |
| --- | --- |
| Working branch | `fix/fork-review-fixes` (exists, at `5549e15`) |
| Worktree | `/home/tango/repo/meetily/.worktrees/fork-review-fixes` — every command runs here; the main checkout is never modified outside `.omo/` |
| Commits | **33** in total — exactly one per code-touching todo: **32** on `fix/fork-review-fixes` (T2…T34) + **1** on `fix/upstream-review-deferrals` (T33). **T1 produces no commit** (setup only). Subject line exactly as written in that todo's `Commit:` line, body = one line listing the addressed finding ids |
| History | Linear, forward-only. **No `rebase`, no `amend`, no `reset --hard`, no squashing** — a reviewer must be able to read the history todo-by-todo |
| Message style | Conventional Commits (`fix:`, `perf:`, `refactor:`, `test:`, `chore:`, `ci:`) with a scoped area, e.g. `fix(funasr-ui): normalise download errors and support per-download cancel` |
| Evidence | Each commit's todo leaves `.omo/evidence/task-<N>-fork-review-fixes.txt` **before** the commit is created, so the evidence and the change land together |
| Upstream branch | `fix/upstream-review-deferrals` in the second worktree `.worktrees/upstream-deferrals`, based on `upstream/main`, **one** commit: `fix(upstream): address the deferred upstream-origin review findings` |
| Push | **None.** No `push`, no `force-push`, no tag, no PR creation. Both branches stay local; the user pushes and opens the PR from the T33 PR body |
| Cross-branch rule | The fork branch contains **zero** upstream-origin changes; the upstream branch contains **zero** fork-only changes. The two commit sets never mix |

**Commit ordering** (waves are execution order, commit order is the same linear sequence): T1 (setup, **no commit**) → T2…T8 → T21, T9 → T10 → T11 → T12, T13, T14 → T15…T22 → T23…T29 → T30, T31, T32, T34 → T33 (on the separate upstream branch).

**Wave-order note (why T21 lands first inside Wave 2):** T11 calls `get_onnx_providers`, and T21 is what changes that function — so T21 must be committed **before** T11, otherwise the wave order is not a valid topological order and F1.5 fails. T21 was therefore **promoted from Wave 3 into Wave 2**, where it lands FIRST. This keeps every wave inside the 5–8-todo budget *and* keeps the wave order topological. (The alternative — demoting T11 into Wave 3 — was rejected because it would put 9 todos in Wave 3 and break this plan's own wave-sizing rule. Either way the "T11 committed before the todo it depends on" inversion is gone.)

**What a commit may contain:** the todo's own source file, its new/updated test file, its evidence file, and nothing else. Any todo that needs a second source file must record it in the evidence file under `REQUIRED <file> CHANGE (applied by T<N>)` and be applied by the todo that owns that file.

## Success criteria

The work is **complete** only when every line below is true and F1–F4 have all returned `VERDICT: PASS`.

**Coverage**
- [ ] **151 / 151** fork-origin findings fixed across the **33** in-scope files — verified against `.omo/evidence/fork-review-inscope.txt`, not against memory.
- [ ] **10 / 10** upstream-origin findings fixed **on the separate upstream branch only**, packaged as a review-ready commit + the 5-section PR body at `.omo/evidence/task-33-upstream-pr-body.md`.
- [ ] **0** of the **1355** out-of-scope upstream findings touched.
- [ ] `frontend/src-tauri/src/database/repositories/setting.rs` is byte-identical on the fork branch.

**Process**
- [ ] **34 / 34** todos complete, each with its evidence file and a recorded **red→green** TDD transition.
- [ ] Exactly **one** conventional commit per code-touching todo — **33** in total (32 on `fix/fork-review-fixes` for T2–T34, 1 on `fix/upstream-review-deferrals` for T33; T1 commits nothing) — linear history, **nothing pushed**.
- [ ] The main checkout is untouched outside `.omo/`.

**Build & test**
- [ ] `cargo check -p meetily` → exit 0.
- [ ] `cargo test -p meetily` → the failing set is **identical** to the T1 baseline (no new failures).
- [ ] `bun test` (from `frontend/`) → no new failures versus the T1 baseline (compared per test file), with the two new test files green.
- [ ] `eslint .` (from `frontend/`) → no new errors versus `.omo/evidence/t7-eslint-baseline.txt`. `bash -n frontend/build-gpu.sh` → exit 0. `pnpm run build` → exit 0.
- [ ] `cargo clippy -p meetily --all-targets` → **zero new findings** versus `.omo/evidence/t1-clippy-baseline.txt`.

**Hygiene**
- [ ] No new runtime dependency; the only addition is T7's `eslint` devDependency. `ort`, `llama-cpp-2` and the 5 prosemirror pins are byte-identical to the baseline.
- [ ] The Windows `ort` `load-dynamic` feature, the Windows sherpa shared/static split and the `target_os`-gated DirectML provider are all intact.
- [ ] The whitelist of changed paths in F4.1 is exactly matched.
- [ ] The whitespace-stripped provenance test proves the 10 upstream findings are byte-identical on the fork branch.

**Verification**
- [ ] F1, F2, F3, F4 each wrote a report to `.omo/evidence/f<N>-*.md` and each returned `VERDICT: PASS`.
- [ ] Every environment-limited check (Windows/PowerShell runtime, GUI session) is explicitly recorded as deferred **with the exact command a human must run** — never as passed.
