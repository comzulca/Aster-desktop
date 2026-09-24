import { afterEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_MODEL } from './types';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => ({ accepted: true, requestId: 'req-1' })) }));

afterEach(() => {
  delete (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__;
  vi.clearAllMocks();
});

describe('narrow native IPC envelope', () => {
  it('wraps send intent as the single input parameter expected by the Rust DTO', async () => {
    (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ = {};
    const { invoke } = await import('@tauri-apps/api/core');
    const { nativeBridge } = await import('./bridge');
    const input = { conversationId: 'conv-1', requestId: 'req-1', modelId: DEFAULT_MODEL, message: 'Hola' };
    await nativeBridge.sendMessage(input);
    expect(invoke).toHaveBeenCalledWith('send_message', { input });
    expect(JSON.stringify(vi.mocked(invoke).mock.calls[0][1])).not.toContain('approved');
  });

  it('denies calls when the native host is absent', async () => {
    const { nativeBridge } = await import('./bridge');
    await expect(nativeBridge.sendMessage({ conversationId: 'c', requestId: 'r', modelId: DEFAULT_MODEL, message: 'Hi' })).rejects.toThrow('HOST_UNAVAILABLE');
  });
});
