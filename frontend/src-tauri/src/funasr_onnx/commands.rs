use serde::Serialize;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;

// Approximate artifact sizes (bytes) used ONLY for progress display when the
// server omits Content-Length. Provenance: observed sizes of the
// funasr/Paraformer-large artifacts at the time of writing —
// model_quant.onnx ~700 MB, am.mvn ~50 KB, config.yaml ~60 KB.
const MODEL_APPROX_BYTES: u64 = 700_000_000;
const CMVN_APPROX_BYTES: u64 = 50_000;
const CONFIG_APPROX_BYTES: u64 = 60_000;

// Progress weights per download stage. They sum to 0.995; the remaining 0.005
// covers token extraction from config.yaml (a step that downloads no bytes).
const WEIGHT_MODEL: f64 = 0.95;
const WEIGHT_CMVN: f64 = 0.025;
const WEIGHT_CONFIG: f64 = 0.02;

// Per-download cancellation registry. A Vec (not a HashMap) preserves
// insertion order so `cancel(None)` can target the most recent still-active
// download, preserving the single-download UI contract.
static DOWNLOADS: Mutex<Vec<(String, Arc<AtomicBool>)>> = Mutex::new(Vec::new());
static DOWNLOAD_SEQ: AtomicU64 = AtomicU64::new(0);

// Clone is REQUIRED by Tauri's `Emitter::emit` bound (broadcast to N listeners),
// not by any in-file clone site — see evidence for the plan item 1 disposition.
#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub speed_mbps: f64,
    pub stage: String,
}

/// Pure progress calculation, extracted so it is testable without network.
/// An unknown or zero total is treated as indeterminate and yields 0.0 (never
/// NaN); the result is always finite and clamped to 0.0..=100.0.
fn progress_percent(downloaded: u64, total: Option<u64>) -> f64 {
    let Some(total) = total.filter(|t| *t > 0) else {
        return 0.0;
    };
    let raw = (downloaded as f64 / total as f64) * 100.0;
    if raw.is_finite() {
        raw.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Extract `token_list` from a parsed config.yaml. Non-string entries are an
/// error naming the offending entry (never silently converted to "").
fn extract_token_list(yaml: &Value) -> Result<Vec<String>, String> {
    let seq = yaml["token_list"]
        .as_sequence()
        .ok_or_else(|| "Missing token_list in config.yaml".to_string())?;
    let mut out = Vec::with_capacity(seq.len());
    for (idx, v) in seq.iter().enumerate() {
        match v.as_str() {
            Some(s) => out.push(s.to_string()),
            None => {
                return Err(format!(
                    "token_list entry {} is not a string: {:?}",
                    idx, v
                ))
            }
        }
    }
    Ok(out)
}

fn downloads_lock() -> std::sync::MutexGuard<'static, Vec<(String, Arc<AtomicBool>)>> {
    // Poison-safe: a panic in one download path must not permanently wedge
    // cancellation for every future download.
    DOWNLOADS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Register a download under `id` and return its cancel handle. Must be called
/// before streaming starts so a cancel arriving during setup is not lost.
fn register_download(id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    downloads_lock().push((id.to_string(), flag.clone()));
    flag
}

fn unregister_download(id: &str) {
    downloads_lock().retain(|(existing, _)| existing != id);
}

/// Cancel the download registered under `id`. Returns false when the id is
/// unknown (the command layer treats that as idempotent Ok).
fn cancel_download_handle(id: &str) -> bool {
    let mut guard = downloads_lock();
    if let Some(pos) = guard.iter().position(|(existing, _)| existing == id) {
        let (_, flag) = guard.remove(pos);
        flag.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}

/// Cancel the most recent still-active download (UI contract: the manager
/// calls cancel with no id). Skips already-cancelled stale entries.
fn cancel_most_recent() -> bool {
    let mut guard = downloads_lock();
    while let Some((_, flag)) = guard.pop() {
        if !flag.load(Ordering::Relaxed) {
            flag.store(true, Ordering::Relaxed);
            return true;
        }
    }
    false
}

/// The `.part` sibling used for atomic writes: download to `<name>.part`,
/// then rename onto the real destination only after the file is complete.
fn part_path(dest: &Path) -> PathBuf {
    let mut name = dest.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    dest.with_file_name(name)
}

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir
        .join("models")
        .join("funasr")
        .join("paraformer-large"))
}

async fn download_file(
    url: &str,
    dest: &Path,
    app: &AppHandle,
    base_progress: f64,
    weight: f64,
    file_total_bytes: u64,
    stage_label: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<(), String> {
    // Timeouts are mandatory: without them a stalled connection hangs the
    // download (and the progress UI) forever.
    let client = reqwest::Client::builder()
        .user_agent("Meetily/1.0")
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed for {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), url));
    }

    let total = if file_total_bytes > 0 {
        file_total_bytes
    } else {
        response.content_length().unwrap_or(0)
    };

    // Atomic write: stream into `<dest>.part`, promote with rename on success.
    let tmp = part_path(dest);
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("Failed to create file {}: {}", tmp.display(), e))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start = std::time::Instant::now();

    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            // Drop the handle BEFORE removal: on Windows an open handle makes
            // remove_file fail. tokio::fs keeps this on the async runtime.
            drop(file);
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err("Download cancelled by user".to_string());
        }

        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        let len = chunk.len() as u64;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += len;

        let elapsed = start.elapsed().as_secs_f64();
        let speed_mbps = if elapsed > 0.0 {
            (downloaded as f64 / 1_048_576.0) / elapsed
        } else {
            0.0
        };

        let file_percent = progress_percent(downloaded, if total > 0 { Some(total) } else { None });
        let overall = base_progress + weight * (file_percent / 100.0);

        let _ = app.emit(
            "funasr:download-progress",
            DownloadProgress {
                bytes_downloaded: downloaded,
                total_bytes: total,
                percent: overall,
                speed_mbps,
                stage: stage_label.to_string(),
            },
        );
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;
    drop(file);

    // Integrity gate: never promote an empty artifact to the real path.
    let written = tokio::fs::metadata(&tmp)
        .await
        .map_err(|e| format!("Failed to stat {}: {}", tmp.display(), e))?
        .len();
    if written == 0 {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(format!("Downloaded file is empty: {}", dest.display()));
    }

    tokio::fs::rename(&tmp, dest)
        .await
        .map_err(|e| format!("Failed to finalize {}: {}", dest.display(), e))?;

    Ok(())
}

/// Try downloading from HuggingFace first, fall back to ModelScope on error.
async fn download_with_fallback(
    hf_url: &str,
    ms_url: &str,
    dest: &Path,
    app: &AppHandle,
    base_progress: f64,
    weight: f64,
    file_total_bytes: u64,
    stage_label: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<(), String> {
    // Try HF first
    match download_file(
        hf_url,
        dest,
        app,
        base_progress,
        weight,
        file_total_bytes,
        stage_label,
        cancel,
    )
    .await
    {
        Ok(()) => return Ok(()),
        Err(e) => {
            log::warn!("HF download failed ({}), falling back to ModelScope", e);
            // Clean up any partial file before retry
            let _ = tokio::fs::remove_file(part_path(dest)).await;
        }
    }

    // Fall back to ModelScope
    download_file(
        ms_url,
        dest,
        app,
        base_progress,
        weight,
        file_total_bytes,
        stage_label,
        cancel,
    )
    .await
}

#[tauri::command]
pub async fn funasr_download_model(app: AppHandle) -> Result<String, String> {
    // Per-download id + handle, registered BEFORE any streaming starts.
    let download_id = format!("funasr-{}", DOWNLOAD_SEQ.fetch_add(1, Ordering::Relaxed));
    let cancel = register_download(&download_id);

    let dir = models_dir(&app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create models dir: {}", e))?;

    let model_path = dir.join("model.onnx");
    let cmvn_path = dir.join("am.mvn");
    let tokens_path = dir.join("tokens.json");
    let cfg_path = dir.join("config.yaml");

    // If all files already exist, skip
    if model_path.exists() && cmvn_path.exists() && tokens_path.exists() {
        log::info!("FunASR model already downloaded, skipping");
        let _ = app.emit(
            "funasr:download-progress",
            DownloadProgress {
                bytes_downloaded: 0,
                total_bytes: 0,
                percent: 100.0,
                speed_mbps: 0.0,
                stage: "already_downloaded".to_string(),
            },
        );
        unregister_download(&download_id);
        return Ok(download_id);
    }

    let base_url = "https://huggingface.co/funasr/Paraformer-large/resolve/main";
    let ms_base_url = "https://modelscope.cn/models/iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch/resolve/master";

    let result = async {
        // 1) ONNX model file (WEIGHT_MODEL of total progress)
        log::info!("Downloading FunASR model.onnx (~700MB)...");
        download_with_fallback(
            &format!("{}/model_quant.onnx", base_url),
            &format!("{}/model_quant.onnx", ms_base_url),
            &model_path,
            &app,
            0.0,
            WEIGHT_MODEL,
            MODEL_APPROX_BYTES,
            "downloading_model",
            &cancel,
        )
        .await?;

        // 2) am.mvn (WEIGHT_CMVN)
        log::info!("Downloading am.mvn...");
        download_with_fallback(
            &format!("{}/am.mvn", base_url),
            &format!("{}/am.mvn", ms_base_url),
            &cmvn_path,
            &app,
            WEIGHT_MODEL,
            WEIGHT_CMVN,
            CMVN_APPROX_BYTES,
            "downloading_cmvn",
            &cancel,
        )
        .await?;

        // 3) config.yaml (WEIGHT_CONFIG) → extract tokens.json from it
        log::info!("Downloading config.yaml for token extraction...");
        download_with_fallback(
            &format!("{}/config.yaml", base_url),
            &format!("{}/config.yaml", ms_base_url),
            &cfg_path,
            &app,
            WEIGHT_MODEL + WEIGHT_CMVN,
            WEIGHT_CONFIG,
            CONFIG_APPROX_BYTES,
            "downloading_config",
            &cancel,
        )
        .await?;

        // Extract token_list FIRST (tokens.json written before config.yaml is
        // touched), so a failure here never orphans a half-updated state.
        log::info!("Extracting token_list from config.yaml...");
        let content = std::fs::read_to_string(&cfg_path)
            .map_err(|e| format!("Failed to read config.yaml: {}", e))?;
        let yaml: Value =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config.yaml: {}", e))?;
        let token_list = extract_token_list(&yaml)?;
        let tokens_json = serde_json::to_string(&token_list)
            .map_err(|e| format!("Failed to serialize tokens: {}", e))?;
        std::fs::write(&tokens_path, &tokens_json)
            .map_err(|e| format!("Failed to write tokens.json: {}", e))?;
        Ok::<(), String>(())
    }
    .await;

    match result {
        Ok(()) => {
            // tokens.json is already written; config.yaml removal is cleanup
            // and must not fail the whole download (a leftover file is harmless).
            if let Err(e) = std::fs::remove_file(&cfg_path) {
                log::warn!("Failed to remove config.yaml (non-fatal): {}", e);
            }
            unregister_download(&download_id);
            let _ = app.emit(
                "funasr:download-progress",
                DownloadProgress {
                    bytes_downloaded: 0,
                    total_bytes: 0,
                    percent: 100.0,
                    speed_mbps: 0.0,
                    stage: "completed".to_string(),
                },
            );
            log::info!("FunASR model download complete");
            Ok(download_id)
        }
        Err(e) => {
            // Remove partial artifacts so the next run's "already downloaded"
            // check can never pass with a corrupt model.
            for p in [&model_path, &cmvn_path, &cfg_path] {
                let _ = tokio::fs::remove_file(part_path(p)).await;
            }
            unregister_download(&download_id);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn funasr_cancel_download(download_id: Option<String>) -> Result<(), String> {
    let cancelled = match download_id.as_deref() {
        Some(id) => cancel_download_handle(id),
        None => cancel_most_recent(),
    };
    if !cancelled {
        log::info!("No active download to cancel (id: {:?})", download_id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_percent_known_total() {
        assert_eq!(progress_percent(50, Some(100)), 50.0);
    }

    #[test]
    fn test_progress_percent_unknown_total_is_finite_and_bounded() {
        let p = progress_percent(50, None);
        assert!(p.is_finite(), "unknown total must never yield NaN");
        assert!((0.0..=100.0).contains(&p));
    }

    #[test]
    fn test_progress_percent_zero_total_is_finite() {
        let p = progress_percent(0, Some(0));
        assert!(p.is_finite());
        assert!((0.0..=100.0).contains(&p));
    }

    #[test]
    fn test_progress_percent_clamps_over_100() {
        assert_eq!(progress_percent(150, Some(100)), 100.0);
    }

    /// Single test fn on purpose: the registry is process-global, and one
    /// sequential test avoids cross-test races on insertion order.
    #[test]
    fn test_per_download_cancel_registry() {
        // Isolation: cancelling one handle leaves a concurrent one untouched.
        let a = register_download("t-iso-a");
        let b = register_download("t-iso-b");
        assert!(cancel_download_handle("t-iso-a"));
        assert!(a.load(Ordering::Relaxed));
        assert!(!b.load(Ordering::Relaxed), "other download must be unaffected");

        // Unknown id: not found (command layer maps this to idempotent Ok).
        assert!(!cancel_download_handle("t-does-not-exist"));

        // Most-recent: targets the latest still-active handle only.
        let c = register_download("t-rec-c");
        let d = register_download("t-rec-d");
        assert!(cancel_most_recent());
        assert!(d.load(Ordering::Relaxed), "most recent must be cancelled");
        assert!(!c.load(Ordering::Relaxed), "older download must be unaffected");

        unregister_download("t-iso-b");
        unregister_download("t-rec-c");
    }

    #[test]
    fn test_extract_token_list_valid() {
        let yaml: Value =
            serde_yaml::from_str("token_list:\n  - \"a\"\n  - \"b\"").unwrap();
        assert_eq!(
            extract_token_list(&yaml).unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn test_extract_token_list_non_string_entry_errors() {
        let yaml: Value =
            serde_yaml::from_str("token_list:\n  - 1.5\n  - \"a\"").unwrap();
        let err = extract_token_list(&yaml).unwrap_err();
        assert!(
            err.contains("1.5"),
            "error must name the offending entry, got: {err}"
        );
    }

    #[test]
    fn test_extract_token_list_missing() {
        let yaml: Value = serde_yaml::from_str("other: 1").unwrap();
        assert!(extract_token_list(&yaml).is_err());
    }

    #[test]
    fn test_part_path_appends_suffix() {
        let dest = Path::new("/tmp/models/model.onnx");
        assert_eq!(part_path(dest), PathBuf::from("/tmp/models/model.onnx.part"));
    }
}
