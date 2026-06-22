# Dev Build Troubleshooting

## 1. Database Migration Checksum Mismatch

**Error:**
```
thread 'main' panicked at lib.rs:512:14:
Failed to initialize database: "migration 20250916100000 was previously applied but has been modified"
```

**Root cause:** The dev build has modified SQL migration files that don't match the checksums stored in the existing SQLite database at `%APPDATA%\com.meetily.ai\meeting_minutes.sqlite`. This happens when migrations are changed after the database was first created.

**Fix:** Delete the SQLite database and let it be recreated:

```powershell
Remove-Item "$env:APPDATA\com.meetily.ai\meeting_minutes.sqlite"
pnpm run tauri:dev
```

The database will be created fresh with the new migration checksums. You'll go through onboarding again.

---

## 2. Stale `.next` Cache → Long Unresponsiveness / ChunkLoadError

**Symptoms:**
- App appears hung for several minutes after Tauri backend starts
- Browser console shows: `ChunkLoadError: Loading chunk app/layout.js failed (timeout)`
- Next.js log shows `⚠ Fast Refresh had to perform a full reload due to a runtime error`
- Pages eventually work after a hard refresh

**Root cause:** When the Tauri dev binary panics/aborts mid-compilation (e.g., from a database crash), the `.next` build cache is left in a corrupted state with partial/stale JS chunks. On the next launch, Next.js serves these stale chunks which the browser cannot load, causing the timeout and error. After a forced full reload, Next.js recompiles correctly.

**Timeline (real example):**
```
04:47:20  Tauri starts, compiles / and /favicon.ico
04:47:20  ← 3.5 min gap of unresponsiveness →
04:50:44  /settings compiled, Fast Refresh triggers full reload
04:50:44  Everything works after reload
```

**Fix:**
```powershell
# Before restarting after a Tauri crash:
Remove-Item -Recurse -Force .next
pnpm run tauri:dev
```

**Prevention:** Always clean `.next` cache if the Tauri process crashed during the previous run.

---

## 3. FunASR Provider Selection Not Persisting

**Symptom:** Selecting "🎯 FunASR (Paraformer - Chinese)" in Settings > Transcription works temporarily, but after navigating away and coming back, it falls back to Parakeet/Whisper.

**Root cause:** The `api_save_transcript_config` backend call was never invoked for the FunASR provider. The Whisper and Parakeet providers save via their ModelManager components (`WhisperModelManager.tsx`, `ParakeetModelManager.tsx`) which call `invoke('api_save_transcript_config')` when a model is selected. FunASR had no ModelManager and no equivalent save trigger.

**Fix (already applied):** Added a save trigger in `TranscriptSettings.tsx` when `funasr` is selected:

```typescript
if (provider === 'funasr') {
    setTranscriptModelConfig({ ...transcriptModelConfig, provider: 'funasr', model: '' });
    invoke('api_save_transcript_config', { provider: 'funasr', model: '', apiKey: null });
}
```

The Rust backend (`api/api.rs`, `audio/transcription/engine.rs`) already supports `funasr` as a provider — it was only the frontend save path that was missing.

---

## 4. Windows: GPU / CPU Launch Settings

When running `pnpm run tauri:dev` on Windows, you may see:

```
⚠️  Vulkan detected but missing dependencies - falling back to CPU
   Missing: VULKAN_SDK environment variable
   Missing: BLAS_INCLUDE_DIRS environment variable
```

The auto-detection script (`scripts/tauri-auto.js`) checks for GPU capabilities and falls back to CPU if not found.

### CPU Mode (Default)

```powershell
pnpm run tauri:dev
pnpm run tauri:dev:cpu
```

This uses `--no-default-features --features platform-default`. Works on any Windows machine without extra setup.

### NVIDIA CUDA GPU

```powershell
# Install CUDA toolkit first, then:
pnpm run tauri:dev:cuda
```

### AMD / Intel GPU (Vulkan)

Requires Vulkan SDK + BLAS:
```powershell
# Set environment variables:
$env:VULKAN_SDK = "C:\VulkanSDK\1.3.296.0"  # adjust version
$env:BLAS_INCLUDE_DIRS = "C:\vcpkg\installed\x64-windows\include"
pnpm run tauri:dev:vulkan
```

### CPU with OpenBLAS Optimization

```powershell
pnpm run tauri:dev  # (default CPU mode)
# Or for production:
cargo build --release --features openblas
```

### Notes

- macOS defaults to Metal + CoreML
- Windows defaults to CPU (Vulkan/CUDA are opt-in)
- Linux defaults to CPU (Vulkan/CUDA are opt-in)
- Benchmark: **GPU is 5-10x faster** than CPU for Whisper transcription

---

## 5. Single Instance Conflict

**Symptom:** Running `pnpm run tauri:dev` fails silently or the built version launches instead of the dev version.

**Root cause:** The app uses Tauri's single-instance plugin. If an installed version of Meetily is already running, the dev binary will fail to start or hand off to the existing instance.

**Fix:**
```powershell
# Kill all running Meetily processes
taskkill /F /IM meetily.exe
pnpm run tauri:dev
```

Check for running instances in Task Manager or with:
```powershell
Get-Process meetily* -ErrorAction SilentlyContinue
```

---

## 6. Next.js Dev Server Unreachable

**Symptom:** `curl http://localhost:3118` fails or browser shows "connection refused."

**Checklist:**
1. Is `pnpm run tauri:dev` or `pnpm run dev` running in a terminal?
2. Are you using **`http://`** not `https://`? The dev server uses HTTP.
3. Is the port `3118` in use? Check with `netstat -ano | findstr :3118`
4. Did the Tauri process crash? Check terminal for Rust panics.
5. Is an old Next.js process on the port? Kill with `taskkill /F /IM node.exe`

---

## Quick Reference

```powershell
# Full clean restart for dev work:
taskkill /F /IM meetily.exe 2>$null
taskkill /F /IM node.exe 2>$null
Remove-Item -Recurse -Force .next -ErrorAction SilentlyContinue
Remove-Item "$env:APPDATA\com.meetily.ai\meeting_minutes.sqlite" -ErrorAction SilentlyContinue
pnpm run tauri:dev

# Just clean stale cache (after Tauri crash):
Remove-Item -Recurse -Force .next
pnpm run tauri:dev

# Kill meeting_minutes.sqlite DB and .next cache (after migration change):
Remove-Item "$env:APPDATA\com.meetily.ai\meeting_minutes.sqlite"
Remove-Item -Recurse -Force .next
pnpm run tauri:dev
```
