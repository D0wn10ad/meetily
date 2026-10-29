import { afterEach, beforeEach, describe, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

// Inject a mock `invoke` directly into the hook module. bun's `mock.module` is
// global and leaks across test files (an earlier file mocking
// `@tauri-apps/api/core` would otherwise capture this module's binding), so the
// hook exposes `__setInvoke` as a test-only hook to avoid that fragility.
type Request = { command: string; resolve: (v: unknown) => void; reject: (r: unknown) => void };
const requests: Request[] = [];
const invoke = mock((command: string) => new Promise((resolve, reject) => {
  requests.push({ command, resolve, reject });
}));

const { useTranscriptionModels, PROVIDER_CONFIG_MAP, __setInvoke } = await import(
  '../../src/hooks/useTranscriptionModels'
);

type Config = { provider?: string; model?: string };
let state: ReturnType<typeof useTranscriptionModels>;
let renderer: ReactTestRenderer | undefined;
function View({ config }: { config?: Config }) {
  state = useTranscriptionModels(config);
  return <output>{state.availableModels.length}</output>;
}
beforeEach(() => {
  requests.length = 0;
  invoke.mockClear();
  __setInvoke(invoke as any);
});
afterEach(async () => {
  if (renderer) await act(async () => renderer!.unmount());
  renderer = undefined;
});

function request(command: string) {
  const r = requests.find((x) => x.command === command);
  expect(r).toBeDefined();
  return r!;
}
async function resolve(command: string, value: unknown) {
  await act(async () => { request(command).resolve(value); });
}
async function reject(command: string, reason: unknown) {
  await act(async () => { request(command).reject(reason); });
}
async function show(config?: Config) {
  await act(async () => {
    if (renderer) renderer.update(<View config={config} />);
    else renderer = create(<View config={config} />);
  });
  // The hook does not auto-fetch on mount; the consumer calls `fetchModels`.
  await act(async () => { void state.fetchModels(); });
}

const whisper = (name: string) => ({ name, size_mb: 1, status: 'Available' as const });
const parakeet = (name: string) => ({ name, size_mb: 2, status: 'Available' as const });
const funasr = (name: string) => ({ name, path: '/m', valid: true, size_mb: 3 });
const sherpa = (name: string) => ({ name, path: '/m', valid: true, size_mb: 4 });

describe('useTranscriptionModels', () => {
  test('fetches all four providers concurrently', async () => {
    await show();
    // After a single tick, all four commands must have been issued even though
    // none of their promises has resolved yet.
    await act(async () => {});
    expect(requests.map((r) => r.command).sort()).toEqual([
      'api_scan_funasr_models',
      'parakeet_get_available_models',
      'sherpa_onnx_scan_models',
      'whisper_get_available_models',
    ]);
  });

  test('flattens the model list in provider order whisper -> parakeet -> funasr -> sherpa-onnx', async () => {
    await show();
    await resolve('whisper_get_available_models', [whisper('w1'), whisper('w2')]);
    await resolve('parakeet_get_available_models', [parakeet('p1')]);
    await resolve('api_scan_funasr_models', [funasr('f1')]);
    await resolve('sherpa_onnx_scan_models', [sherpa('s1')]);
    expect(state.availableModels.map((m) => m.provider)).toEqual([
      'whisper',
      'whisper',
      'parakeet',
      'funasr',
      'sherpa-onnx',
    ]);
    expect(state.availableModels.map((m) => m.displayName)).toEqual([
      '🏠 Whisper: w1',
      '🏠 Whisper: w2',
      '⚡ Parakeet: p1',
      '🎯 FunASR (Paraformer): f1',
      '🏷️ Sherpa-ONNX: s1',
    ]);
  });

  test('one provider rejecting still populates the others and surfaces the failure', async () => {
    await show();
    await resolve('whisper_get_available_models', [whisper('w1')]);
    await reject('parakeet_get_available_models', new Error('parakeet down'));
    await resolve('api_scan_funasr_models', [funasr('f1')]);
    await resolve('sherpa_onnx_scan_models', [sherpa('s1')]);
    expect(state.availableModels.map((m) => m.provider)).toEqual([
      'whisper',
      'funasr',
      'sherpa-onnx',
    ]);
    expect(state.failedProviders).toEqual(['parakeet']);
    expect(state.loadingModels).toBe(false);
  });

  test('PROVIDER_CONFIG_MAP maps localWhisper to whisper and resolves the configured model', async () => {
    expect(PROVIDER_CONFIG_MAP['localWhisper']).toBe('whisper');
    expect(PROVIDER_CONFIG_MAP['whisper']).toBe('whisper');
    expect(PROVIDER_CONFIG_MAP['parakeet']).toBe('parakeet');
    expect(PROVIDER_CONFIG_MAP['funasr']).toBe('funasr');
    expect(PROVIDER_CONFIG_MAP['sherpa-onnx']).toBe('sherpa-onnx');

    await show({ provider: 'localWhisper', model: 'w1' });
    await resolve('whisper_get_available_models', [whisper('w1'), whisper('w2')]);
    await resolve('parakeet_get_available_models', [parakeet('p1')]);
    await resolve('api_scan_funasr_models', [funasr('f1')]);
    await resolve('sherpa_onnx_scan_models', [sherpa('s1')]);
    expect(state.selectedModelKey).toBe('whisper:w1');
  });
});