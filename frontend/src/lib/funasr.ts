import { invoke } from '@tauri-apps/api/core';

export interface FunasrDownloadProgress {
  progress: number;
  downloaded_mb: number;
  total_mb: number;
  speed_mbps: number;
  status: string;
}

export class FunasrAPI {
  static async downloadModel(): Promise<void> {
    await invoke('funasr_download_model');
  }

  static async cancelDownload(): Promise<void> {
    await invoke('funasr_cancel_download');
  }
}
