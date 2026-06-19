# Meetily — Agent Guidance

## Project

**Meetily** — Privacy-first AI meeting assistant. Tauri 2.x desktop app (Rust + Next.js 14 + React 18 + Tailwind CSS). All local: no cloud dependency.

## Workspace & Entrypoints

- Cargo workspace: `frontend/src-tauri` (main app, lib crate `app_lib`) + `llama-helper` (sidecar binary for LLM inference).
- Rust entry: `frontend/src-tauri/src/main.rs` → `app_lib::run()` (defined in `lib.rs`). **Do NOT edit `main.rs`.**
- Frontend entry: `frontend/src/app/page.tsx` (Next.js App Router).
- Tauri config: `frontend/src-tauri/tauri.conf.json` — version, CSP, bundle targets, updater.
- Version source of truth: `tauri.conf.json`, **not** `package.json` or `Cargo.toml`.

## Essential Commands (run from `frontend/`)

```bash
pnpm install                  # Install JS deps (pnpm v8, NOT npm/yarn)
pnpm run dev                  # Next.js dev server only (port 3118)
pnpm run tauri:dev            # Full Tauri dev (auto-detects GPU via scripts/tauri-auto.js)
pnpm run tauri:build          # Production build
pnpm run tauri:dev:cpu        # Tauri dev without GPU acceleration
pnpm run tauri:dev:metal      # macOS Metal
pnpm run tauri:dev:cuda       # NVIDIA CUDA
pnpm run tauri:dev:vulkan     # AMD/Intel Vulkan
pnpm run lint                 # Next.js lint (ESLint, no config file in repo)

# Helper scripts (macOS/Linux):
./clean_run.sh [info|debug|trace]   # Full clean + install + dev (removes node_modules!)
./clean_build.sh [info|debug|trace] # Full clean + production build
./build-gpu.sh                      # Auto-detect GPU + production build
./dev-gpu.sh                        # Auto-detect GPU + dev mode

# Cargo from workspace root:
cargo build -p meetily              # Rust only (no frontend)
cargo build -p llama-helper         # Sidecar binary only
cargo check                         # Rust type-check only
```

Rust features for GPU: `metal`, `coreml`, `cuda`, `vulkan`, `hipblas`, `openblas`, `openmp`. macOS defaults to Metal+CoreML; Windows/Linux default to CPU.

## Testing

```bash
# Rust tests
cargo test -p meetily               # Unit/integration tests for the Tauri backend

# Frontend tests (uses bun:test — NOT Jest or Vitest)
bun test                            # Run from frontend/ directory
# Tests located in: frontend/tests/lib/*.test.ts/.mjs/.js
```

No test runner is configured in `package.json`. Tests use `bun:test` independently.

## Key Architecture

- **Communication**: Tauri commands (frontend → Rust via `invoke()`) + events (Rust → frontend via `app.emit()`).
- **State**: React context (`SidebarProvider`) + Rust atomics (`RecordingState`).
- **Audio pipeline** (`audio/pipeline.rs`): dual path — recording (mixed mic+system) and transcription (VAD-filtered → Whisper).
- **Two audio directories**: `audio/` (primary, modularized) and `audio_v2/` (experimental, 9 files — `lib.rs` is the entrypoint). Prefer `audio/` for new work unless v2 migration is explicit.
- **Database**: SQLite via `sqlx`. Schema is embedded in `database/` module.
- **Whisper**: `whisper_engine/whisper_engine.rs`. Models: dev in `frontend/models/`, prod in app data dir (`$APPDATA/Meetily/models/` or `~/Library/Application Support/Meetily/models/`).
- **llama-helper**: Built separately, placed in `frontend/src-tauri/binaries/`, declared as Tauri `externalBin` sidecar.
- **FFmpeg**: Downloaded and bundled at build time (`build/ffmpeg.rs`). Cached in `binaries/` directory.
- **Naming convention**: Audio devices are "microphone" and "system" (never "input"/"output").

## Constraints & Gotchas

1. **Next.js `output: 'export'`** (static export) — no SSR, no API routes. `reactStrictMode: false` (BlockNote/TipTap compatibility).
2. **ProseMirror version-pinned** via pnpm overrides (`prosemirror-model@1.25.3`, etc.) — do NOT bump these independently. Webpack aliases keep BlockNote/TipTap on the same singleton (`next.config.js`).
3. **CSP** in `tauri.conf.json` allows `connect-src` to `localhost:11434` (Ollama), `localhost:5167`, `localhost:8178`. Add new LLM endpoints here.
4. **macOS entitlements** (`entitlements.plist`): microphone, audio output, screen capture required.
5. **Linux build dependencies**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `libopenblas-dev`, `cmake`, `llvm`. WebKit version is pinned in CI (`2.44.0-2`).
6. **App identifier**: `com.meetily.ai` — used in path resolution, do not change.
7. **Crate name vs lib name**: Package is `meetily`, lib crate is `app_lib`. Import as `use app_lib::...` in Rust code.
8. **Logging macros**: `perf_debug!()` and `perf_trace!()` compile to no-ops in release builds. Use `log::debug!()` for persistent debug output.
9. **Backend directory** `backend/` is archived Python/FastAPI legacy. Do not create or modify files there. All features go through Tauri Rust core.

## Git Workflow

- `main` — production; `devtest` — integration/testing; feature branches from `devtest`.
- PRs target `devtest`, not `main`.
- Commit format: conventional commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`).

## CI Artifacts

- macOS: `.dmg` + `.app.tar.gz` + `.sig`
- Windows: `.msi` + `.msi.sig` + `.exe` (NSIS) + `.exe.sig`
- Linux (22.04): `.deb` — Linux (24.04): `.AppImage` + `.rpm`
- Updater signatures always generated. Code signing optional (macOS Apple ID, Windows DigiCert).
