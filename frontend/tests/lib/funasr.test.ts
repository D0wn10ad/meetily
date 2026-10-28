import { describe, expect, mock, test } from 'bun:test';

// Inject a mock `invoke` directly into the funasr module. bun's `mock.module`
// is global and leaks across test files, so a plain `@tauri-apps/api/core`
// mock would be overwritten by whichever test file ran first. The module
// exposes `__setInvoke` as a test-only hook to avoid that fragility.
const invoke = mock(async (command: string, args?: unknown): Promise<unknown> => {
  if (command === 'funasr_download_model') return 'dl-123';
  if (command === 'funasr_cancel_download') return undefined;
  if (command === 'api_save_transcript_config') return undefined;
  throw new Error(`Unexpected command: ${command}`);
});

const { FunasrAPI, describeInvokeError, saveFunasrModelSelection, __setInvoke } = await import(
  '../../src/lib/funasr'
);
__setInvoke(invoke);

describe('FunasrAPI', () => {
  test('downloadModel returns the download id from invoke', async () => {
    invoke.mockClear();
    const id = await FunasrAPI.downloadModel();
    expect(id).toBe('dl-123');
    expect(invoke).toHaveBeenCalledWith('funasr_download_model');
  });

  test('downloadModel rejects with a human-readable message on invoke failure', async () => {
    invoke.mockClear();
    invoke.mockImplementationOnce(async () => {
      throw new Error('boom');
    });
    await expect(FunasrAPI.downloadModel()).rejects.toThrow(
      /Failed to download the FunASR model: boom/
    );
  });

  test('cancelDownload with no argument passes downloadId null', async () => {
    invoke.mockClear();
    await FunasrAPI.cancelDownload();
    expect(invoke).toHaveBeenCalledWith('funasr_cancel_download', { downloadId: null });
  });

  test('cancelDownload with an id passes that id', async () => {
    invoke.mockClear();
    await FunasrAPI.cancelDownload('abc');
    expect(invoke).toHaveBeenCalledWith('funasr_cancel_download', { downloadId: 'abc' });
  });

  test('cancelDownload rejects with a human-readable message on invoke failure', async () => {
    invoke.mockClear();
    invoke.mockImplementationOnce(async () => {
      throw new Error('nope');
    });
    await expect(FunasrAPI.cancelDownload()).rejects.toThrow(
      /Failed to cancel the FunASR model download: nope/
    );
  });
});

describe('describeInvokeError', () => {
  test('returns the message for an Error', () => {
    expect(describeInvokeError(new Error('kaboom'))).toBe('kaboom');
  });

  test('returns a string for a non-Error value', () => {
    expect(describeInvokeError('plain string')).toBe('plain string');
    expect(describeInvokeError(42)).toBe('42');
  });
});

describe('saveFunasrModelSelection', () => {
  test('calls invoke exactly once with the funasr config shape', async () => {
    invoke.mockClear();
    await saveFunasrModelSelection('x');
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith('api_save_transcript_config', {
      provider: 'funasr',
      model: 'x',
      apiKey: null,
    });
  });

  test('rejects with a human-readable message on invoke failure', async () => {
    invoke.mockClear();
    invoke.mockImplementationOnce(async () => {
      throw new Error('save failed');
    });
    await expect(saveFunasrModelSelection('x')).rejects.toThrow(
      /^Failed to save the FunASR model selection: save failed/
    );
  });
});