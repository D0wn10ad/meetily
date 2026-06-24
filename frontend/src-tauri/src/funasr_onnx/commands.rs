use std::sync::atomic::{AtomicBool, Ordering};
use std::path::{Path, PathBuf};
use std::fs;
use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;
use serde_yaml::Value;
use tokio::io::AsyncWriteExt;

static CANCEL_DOWNLOAD: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub speed_mbps: f64,
    pub stage: String,
}

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("models").join("funasr").join("paraformer-large"))
}

async fn download_file(
    url: &str,
    dest: &Path,
    app: &AppHandle,
    base_progress: f64,
    weight: f64,
    file_total_bytes: u64,
    stage_label: &str,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("Meetily/1.0")
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
        response
            .content_length()
            .unwrap_or(0)
    };

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("Failed to create file {}: {}", dest.display(), e))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start = std::time::Instant::now();

    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        if CANCEL_DOWNLOAD.load(Ordering::Relaxed) {
            let _ = fs::remove_file(dest);
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

        let file_percent = if total > 0 {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let overall = base_progress + weight * (file_percent / 100.0);

        let _ = app.emit(
            "funasr:download-progress",
            DownloadProgress {
                bytes_downloaded: downloaded,
                total_bytes: total,
                percent: overall.min(100.0),
                speed_mbps,
                stage: stage_label.to_string(),
            },
        );
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

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
) -> Result<(), String> {
    // Try HF first
    match download_file(hf_url, dest, app, base_progress, weight, file_total_bytes, stage_label).await
    {
        Ok(()) => return Ok(()),
        Err(e) => {
            log::warn!("HF download failed ({}), falling back to ModelScope", e);
            // Clean up any partial file before retry
            let _ = fs::remove_file(dest);
        }
    }

    // Fall back to ModelScope
    download_file(ms_url, dest, app, base_progress, weight, file_total_bytes, stage_label).await
}

#[tauri::command]
pub async fn funasr_download_model(app: AppHandle) -> Result<(), String> {
    CANCEL_DOWNLOAD.store(false, Ordering::Relaxed);

    let dir = models_dir(&app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create models dir: {}", e))?;

    let model_path = dir.join("model.onnx");
    let cmvn_path = dir.join("am.mvn");
    let tokens_path = dir.join("tokens.json");

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
        return Ok(());
    }

    let base_url = "https://huggingface.co/funasr/Paraformer-large/resolve/main";
    let ms_base_url = "https://modelscope.cn/models/iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch/resolve/master";

    // 1) ONNX model file (95% of total weight)
    log::info!("Downloading FunASR model.onnx (~700MB)...");
    download_with_fallback(
        &format!("{}/model_quant.onnx", base_url),
        &format!("{}/model_quant.onnx", ms_base_url),
        &model_path,
        &app,
        0.0,
        0.95,
        700_000_000,
        "downloading_model",
    )
    .await?;

    // 2) am.mvn (2.5%)
    log::info!("Downloading am.mvn...");
    download_with_fallback(
        &format!("{}/am.mvn", base_url),
        &format!("{}/am.mvn", ms_base_url),
        &cmvn_path,
        &app,
        0.95,
        0.025,
        50_000,
        "downloading_cmvn",
    )
    .await?;

    // 3) config.yaml (2%) → extract tokens.json from it
    log::info!("Downloading config.yaml for token extraction...");
    let cfg_path = dir.join("config.yaml");
    download_with_fallback(
        &format!("{}/config.yaml", base_url),
        &format!("{}/config.yaml", ms_base_url),
        &cfg_path,
        &app,
        0.975,
        0.02,
        60_000,
        "downloading_config",
    )
    .await?;

    // Extract token_list from config.yaml and write tokens.json (0.5%)
    log::info!("Extracting token_list from config.yaml...");
    let content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Failed to read config.yaml: {}", e))?;
    let yaml: Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.yaml: {}", e))?;
    let token_list = yaml["token_list"]
        .as_sequence()
        .ok_or_else(|| "Missing token_list in config.yaml".to_string())?
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect::<Vec<String>>();
    let tokens_json = serde_json::to_string(&token_list)
        .map_err(|e| format!("Failed to serialize tokens: {}", e))?;
    std::fs::write(&tokens_path, &tokens_json)
        .map_err(|e| format!("Failed to write tokens.json: {}", e))?;
    // Clean up config.yaml (not needed at runtime)
    std::fs::remove_file(&cfg_path)
        .map_err(|e| format!("Failed to remove config.yaml: {}", e))?;

    // Emit completion
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
    Ok(())
}

#[tauri::command]
pub async fn funasr_cancel_download() -> Result<(), String> {
    CANCEL_DOWNLOAD.store(true, Ordering::Relaxed);
    Ok(())
}
