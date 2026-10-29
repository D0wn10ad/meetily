# Upstream review deferrals — 10 findings

## (a) Problem statement

A code review of the Meetily codebase surfaced 10 findings that are inherited from the
upstream `main` branch (they exist identically on `upstream/main` and were not introduced
by the fork). Because the fork's review plan scopes its fixes to fork-origin regressions,
these 10 upstream-inherited problems are addressed here in a standalone branch based on
`upstream/main` so they can be merged upstream independently of the fork's own fixes. The
findings span blocking I/O on the async runtime, a non-idiomatic transaction start, a
crash-consistency ordering defect in retranscription persistence, hardcoded singleton-row
queries with silent default fallbacks, a lossy error conversion, a redundant clone, and
duplicated notification code.

## (b) Per-finding table

| # | File:line (upstream) | What was wrong | What changed |
|---|----------------------|----------------|--------------|
| 1 | `audio/import.rs:200-208` | `extract_duration_from_metadata` did blocking `std::fs::File::open` + a synchronous symphonia probe on the async runtime. | Made the function `async`, wrapping the whole body in `tokio::task::spawn_blocking` (symphonia's `MediaSourceStream` needs a sync `std::io::Read`, so `tokio::fs::File` cannot be used directly). `validate_audio_file` is now `async` and awaits it; both async callers await it; the 5 unit tests became `#[tokio::test]`. |
| 2 | `audio/import.rs:752-768` | `create_meeting_with_transcripts` used `pool.acquire()` + `sqlx::Connection::begin(&mut *conn)`. | Replaced with the idiomatic `pool.begin().await`. |
| 3 | `audio/retranscription.rs:572-591` | The DB `tx.commit()` ran BEFORE `write_transcripts_json` / `write_retranscription_metadata`, so a crash could leave the DB updated but the JSON files stale. | Reordered: the JSON/metadata writes now happen BEFORE `tx.commit()`, so a crash cannot leave the DB and JSON out of sync. |
| 4 | `audio/retranscription.rs:572-591` (perf duplicate) | Same site's cost/latency aspect (duplicate entry). | Resolved together with #3; noted as merged in the evidence file. |
| 5 | `audio/retranscription.rs:751-758` | `get_configured_whisper_model` hardcoded `WHERE id = '1'`, silently fell back to `DEFAULT_WHISPER_MODEL`, and used `error!` inside `ok_or_else` (log flooding). | Query now uses `LIMIT 1` (singleton table, no row-id assumption); silent fallback replaced with an explicit error; `error!` removed from `ok_or_else`. |
| 6 | `audio/retranscription.rs:751-759` | Adjacent variant of the same defect. | Resolved together with #5. |
| 7 | `audio/transcription/engine.rs:60-61` | Redundant `app.clone()` passed to `api_get_transcript_config` (whose `_app` param is unused). | All 3 occurrences changed to `app.clone(), app.state(), None` (single clone; `app.state()` borrows without cloning). |
| 8 | `database/repositories/setting.rs:194-204` | `save_transcript_api_key` hardcoded `provider='parakeet'` and `model=DEFAULT_PARAKEET_MODEL` in the INSERT, ignoring the caller's `provider` param and clobbering the stored model. | INSERT now binds the actual `provider`, preserves the existing model via a `COALESCE((SELECT model ...), $2)` subquery, and uses a provider-appropriate default only on a fresh insert. |
| 9 | `database/repositories/setting.rs:303-307` | `serde_json::Error` was converted to `sqlx::Error::Protocol(Box<str>)`, losing the source chain. | Converted to `sqlx::Error::Decode(Box::new(e))` (sqlx 0.8 `Decode(Box<dyn Error + Send + Sync>)`), preserving the source. |
| 10 | `src/lib.rs:205-218` | The "recording started" notification block was duplicated between `start_recording` and `start_recording_with_devices_and_meeting` (one had an extra success-log branch). | Extracted a single shared `notify_recording_started` helper and called it from both paths, unifying behaviour. |

## (c) PROVENANCE

The plan text describes these as "FORK-ORIGIN regressions", but empirical verification
shows this wording is **inverted**: all 10 findings are **upstream-inherited** — they exist
identically on `upstream/main` (`a2cb62e827da7ef59f65064c97233efb2313878e`). This
discrepancy is recorded as a deviation.

Whitespace-stripped substring provenance test over the 10 deferred findings
(`existing_code` from `w2-core-scan_v3.json` vs. the file content on `upstream/main` and
vs. the fixed upstream worktree):

```
present_on_upstream_main | absent_in_fixed_upstream_wt | finding
True  | True | 1.  audio/import.rs:200-208
True  | True | 2.  audio/import.rs:752-768
False | True | 3.  audio/retranscription.rs:572-591
False | True | 4.  audio/retranscription.rs:572-591
True  | True | 5.  audio/retranscription.rs:751-758
True  | True | 6.  audio/retranscription.rs:751-759
True  | True | 7.  audio/transcription/engine.rs:60-61
True  | True | 8.  database/repositories/setting.rs:194-204
False | True | 9.  database/repositories/setting.rs:303-307
True  | True | 10. src/lib.rs:205-218
```

7/10 findings match the exact whitespace-stripped snippet on `upstream/main`. Findings
3, 4 and 9 show `False` for the substring test **only because the fork reformatted the
code** (multi-line `emit_progress(...)` vs single-line; `.map_err(|e| {` vs
`.map_err(|e| sqlx::Error::Protocol(`). The underlying defect is verified present on
`upstream/main` by direct code reading:

- #3/#4: `git show upstream/main:.../retranscription.rs` L460 `tx.commit().await` precedes
  L472 `write_transcripts_json` and L483 `write_retranscription_metadata`.
- #9: `git show upstream/main:.../setting.rs` L299-301 shows
  `serde_json::from_str(&json).map_err(|e| sqlx::Error::Protocol(format!("Invalid JSON in customOpenAIConfig: {}", e).into()))`.

All 10 findings are absent from the fixed upstream worktree (`absent_in_fixed_upstream_wt`
= True for all), confirming each was addressed.

## (d) Test / verification commands and output

Run in `/home/tango/repo/meetily/.worktrees/upstream-deferrals/frontend/src-tauri`:

```
$ cargo check -p meetily
cargo check: 0 errors, 24 warnings (19 crates)
```
(24 warnings, all pre-existing dead-code warnings; baseline was 25 — one fewer because the
now-unused `DEFAULT_WHISPER_MODEL`/`DEFAULT_PARAKEET_MODEL` import was removed.)

```
$ cargo test -p meetily --lib -- --skip audio --skip sherpa
test result: FAILED. 151 passed; 1 failed; 0 ignored; 0 measured; 94 filtered out
```
The 1 failure is `whisper_engine::...::cancelling_a_stalled_download_waits_for_cleanup_before_retry_can_start`
(a timing-sensitive HTTP-mock test that times out on this headless host) — pre-existing,
unrelated to these changes.

```
$ cargo test -p meetily
test result: FAILED. 243 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out
```
The 1 failure is `audio::device_detection::tests::test_calculate_buffer_timeout_bluetooth`
(`159.999996ms` vs `160ms` float precision) — the documented pre-existing baseline failure.

No new failures were introduced by these changes.

## (e) Out-of-scope statement

The other 1355 upstream findings were NOT touched. `git diff --name-only upstream/main`
in this worktree lists exactly the five files that carry the 10 deferred findings:
`frontend/src-tauri/src/audio/import.rs`, `frontend/src-tauri/src/audio/retranscription.rs`,
`frontend/src-tauri/src/audio/transcription/engine.rs`,
`frontend/src-tauri/src/database/repositories/setting.rs`, and
`frontend/src-tauri/src/lib.rs`. No dependency versions were changed, the archived
`backend/` directory was not modified, and no fork-only fixes (T2–T32) are included in
this branch. The `.github/workflows/**` mutable major-version action pinning is
explicitly out of scope (pre-existing upstream style) and belongs in a separate
supply-chain hardening PR.