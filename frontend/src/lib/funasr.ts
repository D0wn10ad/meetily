import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export interface FunasrDownloadProgress {
  /** @deprecated Use `percent` instead. */
  progress: number;
  /** @deprecated Use `bytes_downloaded` instead. */
  downloaded_mb: number;
  /** @deprecated Use `total_bytes` instead. */
  total_mb: number;
  speed_mbps: number;
  /** @deprecated Use `stage` instead. */
  status: string;
  bytes_downloaded: number;
  total_bytes: number;
  percent: number;
  stage: string;
}

/**
 * Normalise an unknown rejection value into a human-readable string.
 *
 * `invoke` rejects with a string in Tauri 2, but callers may also surface an
 * `Error`; normalising both keeps user-facing messages consistent.
 */
export function describeInvokeError(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

/**
 * The `invoke` function used to talk to the Rust backend.
 *
 * Exposed as a module-level binding so tests can inject a mock without relying
 * on bun's global `mock.module` (which leaks across test files). Production
 * callers never pass an argument and always use the real Tauri `invoke`.
 */
export let invoke: typeof tauriInvoke = tauriInvoke;

/** Test-only hook: replace the `invoke` binding with a mock. */
export function __setInvoke(fn: (cmd: string, args?: unknown) => Promise<unknown>): void {
  invoke = fn as typeof tauriInvoke;
}

export class FunasrAPI {
  static async downloadModel(): Promise<string> {
    try {
      return await invoke<string>('funasr_download_model');
    } catch (err) {
      throw new Error(`Failed to download the FunASR model: ${describeInvokeError(err)}`);
    }
  }

  static async cancelDownload(downloadId?: string): Promise<void> {
    try {
      await invoke<void>('funasr_cancel_download', { downloadId: downloadId ?? null });
    } catch (err) {
      throw new Error(`Failed to cancel the FunASR model download: ${describeInvokeError(err)}`);
    }
  }
}

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