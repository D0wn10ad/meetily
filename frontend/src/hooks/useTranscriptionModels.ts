import { useState, useCallback, useRef } from 'react';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export interface RawModelInfo {
  name: string;
  size_mb: number;
  status: 'Available' | 'Missing' | { Downloading: { progress: number } } | { Error: string };
}

export interface ModelOption {
  provider: 'whisper' | 'parakeet' | 'funasr' | 'sherpa-onnx';
  name: string;
  displayName: string;
  size_mb: number;
}

interface FunasrModelEntry {
  name: string;
  path: string;
  valid: boolean;
  size_mb: number;
}

interface SherpaOnnxModelEntry {
  name: string;
  path: string;
  valid: boolean;
  size_mb: number;
}

interface TranscriptModelConfig {
  provider?: string;
  model?: string;
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

/**
 * Maps a saved config provider value to the `ModelOption['provider']` used in the
 * model list. `localWhisper` is the legacy config value for the whisper provider.
 */
export const PROVIDER_CONFIG_MAP: Record<string, ModelOption['provider']> = {
  localWhisper: 'whisper',
  whisper: 'whisper',
  parakeet: 'parakeet',
  funasr: 'funasr',
  'sherpa-onnx': 'sherpa-onnx',
};

/**
 * Fetch one provider's models and map the usable entries to `ModelOption`s.
 *
 * Shared by all four providers so the fetch-then-filter-map block exists once.
 * `isUsable` decides which raw entries are kept (e.g. `status === 'Available'`
 * for whisper/parakeet, `valid` for funasr/sherpa-onnx).
 */
export async function fetchProviderModels<T>(
  command: string,
  provider: ModelOption['provider'],
  displayPrefix: string,
  isUsable: (item: T) => boolean
): Promise<ModelOption[]> {
  const raw = await invoke<T[]>(command);
  return raw
    .filter(isUsable)
    .map((m) => ({
      provider,
      name: (m as { name: string }).name,
      displayName: `${displayPrefix}${(m as { name: string }).name}`,
      size_mb: (m as { size_mb: number }).size_mb,
    }));
}

/**
 * Custom hook for fetching and managing transcription models (Whisper and Parakeet).
 *
 * This hook centralizes the model fetching logic that was previously duplicated
 * in ImportAudioDialog and RetranscribeDialog components.
 *
 * @param transcriptModelConfig - User's saved model configuration from context
 * @returns Object containing available models, selected model key, loading state, and fetch function
 */
export function useTranscriptionModels(transcriptModelConfig: TranscriptModelConfig | undefined) {
  const [availableModels, setAvailableModels] = useState<ModelOption[]>([]);
  const [selectedModelKey, setSelectedModelKey] = useState<string>('');
  const [loadingModels, setLoadingModels] = useState(false);
  const [failedProviders, setFailedProviders] = useState<string[]>([]);
  // Track whether the user has manually changed the model selection
  const userSelectedRef = useRef(false);

  // Wrap setSelectedModelKey to track user-initiated changes
  const setSelectedModelKeyWithTracking = useCallback((key: string) => {
    userSelectedRef.current = true;
    setSelectedModelKey(key);
  }, []);

  const fetchModels = useCallback(async () => {
    setLoadingModels(true);
    const failedProviders: string[] = [];

    // Fetch all four providers CONCURRENTLY, then flatten in the original order
    // (whisper, parakeet, funasr, sherpa-onnx) so the model-list ordering is unchanged.
    const results = await Promise.allSettled([
      fetchProviderModels<RawModelInfo>(
        'whisper_get_available_models',
        'whisper',
        '🏠 Whisper: ',
        (m) => m.status === 'Available'
      ),
      fetchProviderModels<RawModelInfo>(
        'parakeet_get_available_models',
        'parakeet',
        '⚡ Parakeet: ',
        (m) => m.status === 'Available'
      ),
      fetchProviderModels<FunasrModelEntry>(
        'api_scan_funasr_models',
        'funasr',
        '🎯 FunASR (Paraformer): ',
        (m) => m.valid
      ),
      fetchProviderModels<SherpaOnnxModelEntry>(
        'sherpa_onnx_scan_models',
        'sherpa-onnx',
        '🏷️ Sherpa-ONNX: ',
        (m) => m.valid
      ),
    ]);

    const providerNames = ['whisper', 'parakeet', 'funasr', 'sherpa-onnx'] as const;
    const allModels: ModelOption[] = [];
    results.forEach((result, i) => {
      if (result.status === 'fulfilled') {
        allModels.push(...result.value);
      } else {
        console.error(`Failed to fetch ${providerNames[i]} models:`, result.reason);
        failedProviders.push(providerNames[i]);
      }
    });

    setAvailableModels(allModels);
    setFailedProviders(failedProviders);

    // Set default model based on user's saved configuration
    const configuredProvider = transcriptModelConfig?.provider || '';
    const configuredModel = transcriptModelConfig?.model || '';

    // Try to match the configured model via the provider lookup map
    const configuredMatch = allModels.find(
      (m) => m.provider === PROVIDER_CONFIG_MAP[configuredProvider] && m.name === configuredModel
    );

    // Only set default model if user hasn't manually selected one
    if (!userSelectedRef.current) {
      if (configuredMatch) {
        // Use the configured model if available
        setSelectedModelKey(`${configuredMatch.provider}:${configuredMatch.name}`);
      } else if (allModels.length > 0) {
        // Fall back to first available model
        setSelectedModelKey(`${allModels[0].provider}:${allModels[0].name}`);
      }
    }

    setLoadingModels(false);
  }, [transcriptModelConfig]);

  // Reset user selection tracking (call when dialog opens fresh)
  const resetSelection = useCallback(() => {
    userSelectedRef.current = false;
  }, []);

  return {
    availableModels,
    selectedModelKey,
    setSelectedModelKey: setSelectedModelKeyWithTracking,
    loadingModels,
    failedProviders,
    fetchModels,
    resetSelection,
  };
}