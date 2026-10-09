import { afterEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_MODEL } from './types';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => ({ accepted: true, requestId: 'req_1' })) }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(async () => () => {}) }));
afterEach(() => { delete (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__; vi.clearAllMocks(); });
describe('closed Meta/persistence IPC envelope (T-055)', () => {
  it('uses one input DTO with no authorisation or renderer transcript', async () => {
    (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ = {};
    const { invoke } = await import('@tauri-apps/api/core'); const { nativeBridge } = await import('./bridge');
    const input = { conversationId: 'c', requestId: 'req_1', modelId: DEFAULT_MODEL, message: 'Synthetic example', dataClass: 'synthetic' as const, reasoningEffort: 'medium' as const, attachmentIds: [] };
    await nativeBridge.sendMessage(input); expect(invoke).toHaveBeenLastCalledWith('send_message', { input });
    await nativeBridge.loadConversation({ conversationId: 'c' }); expect(invoke).toHaveBeenLastCalledWith('load_conversation', { input: { conversationId: 'c' } });
    await nativeBridge.deleteConversation({ conversationId: 'c' }); expect(invoke).toHaveBeenLastCalledWith('delete_conversation', { input: { conversationId: 'c' } });
    await nativeBridge.cancelMessage('req_1'); expect(invoke).toHaveBeenLastCalledWith('cancel_message', { requestId: 'req_1' });
    const reconcile = { requestId: 'req_1', confirmedChargeNanoGbp: '1' }; await nativeBridge.reconcileBudget(reconcile); expect(invoke).toHaveBeenLastCalledWith('reconcile_budget', { input: reconcile });
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toMatch(/approved|newContent|apiKey/);
    expect(Object.keys(nativeBridge).join('|')).not.toMatch(/Project|Proposal/);
  });
  it('separates native Meta credential and terms commands from all legacy APIs', async () => {
    (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ = {};
    const { invoke } = await import('@tauri-apps/api/core'); const { nativeBridge } = await import('./bridge');
    await nativeBridge.configureCredential(); expect(invoke).toHaveBeenLastCalledWith('configure_meta_credential', undefined);
    await nativeBridge.removeCredential(); expect(invoke).toHaveBeenLastCalledWith('remove_meta_credential', undefined);
    await nativeBridge.acceptContributorTerms(); expect(invoke).toHaveBeenLastCalledWith('accept_contributor_terms', undefined);
    await nativeBridge.createConversation(); expect(invoke).toHaveBeenLastCalledWith('create_conversation', undefined);
    await nativeBridge.listConversations(); expect(invoke).toHaveBeenLastCalledWith('list_conversations', undefined);
  });
  it('fails closed when the native host is absent', async () => {
    const { nativeBridge } = await import('./bridge');
    await expect(nativeBridge.sendMessage({ conversationId: 'c', requestId: 'r', modelId: DEFAULT_MODEL, message: 'Hi', dataClass: 'synthetic', reasoningEffort: 'medium' })).rejects.toThrow('host_unavailable');
    await expect(nativeBridge.createConversation()).rejects.toThrow('host_unavailable');
    await expect(nativeBridge.windowAction!({ action: 'close' })).rejects.toThrow('host_unavailable');
    await expect(nativeBridge.subscribeClose!(() => {})).rejects.toThrow('host_unavailable');
  });
  it('uses only the bounded window DTO and main close-notification channel (T-069)', async () => {
    (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ = {};
    const { invoke } = await import('@tauri-apps/api/core'); const { listen } = await import('@tauri-apps/api/event'); const { nativeBridge } = await import('./bridge');
    for (const action of ['minimise','toggle_maximise','toggle_fullscreen','start_dragging','close'] as const) {
      await nativeBridge.windowAction!({ action }); expect(invoke).toHaveBeenLastCalledWith('window_action', { input: { action } });
    }
    const callback = vi.fn(); const unsubscribe = await nativeBridge.subscribeClose!(callback);
    expect(listen).toHaveBeenLastCalledWith('aster://close-request', expect.any(Function)); expect(typeof unsubscribe).toBe('function');
    (vi.mocked(listen).mock.calls.at(-1)![1] as (event: unknown) => void)({ payload: undefined }); expect(callback).toHaveBeenCalledOnce();
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toMatch(/target|label|path|commandLine|apiKey|grant|spend|credential/);
  });
  it('uses closed attachment intent and versioned one-shot shell scheduling commands, never paths or original bytes (T-075/T-077/T-080)', async () => {
    (window as typeof window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ = {};
    const { invoke } = await import('@tauri-apps/api/core'); const { nativeBridge } = await import('./bridge');
    await nativeBridge.selectAttachments!({ conversationId: 'a'.repeat(32) });
    expect(invoke).toHaveBeenLastCalledWith('select_attachments', { input: { conversationId: 'a'.repeat(32) } });
    await nativeBridge.removeAttachment!({ conversationId: 'a'.repeat(32), attachmentId: 'b'.repeat(32) });
    expect(invoke).toHaveBeenLastCalledWith('remove_attachment', { input: { conversationId: 'a'.repeat(32), attachmentId: 'b'.repeat(32) } });
    await nativeBridge.viewProcessedAttachment!({ conversationId: 'a'.repeat(32), requestId: 'c'.repeat(32), attachmentId: 'b'.repeat(32) });
    expect(invoke).toHaveBeenLastCalledWith('review_processed_attachment', { input: { conversationId: 'a'.repeat(32), requestId: 'c'.repeat(32), attachmentId: 'b'.repeat(32) } });
    await nativeBridge.shellReady!({ protocolVersion: 1 });
    expect(invoke).toHaveBeenLastCalledWith('startup_shell_ready', { input: { protocolVersion: 1 } });
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toMatch(/path|url|base64|approved|ownerHwnd|fileBytes|action/);
  });
  it('does not imitate file selection or readiness when there is no native host', async () => {
    const { nativeBridge } = await import('./bridge');
    await expect(nativeBridge.selectAttachments!({ conversationId: 'a'.repeat(32) })).rejects.toThrow('host_unavailable');
    await expect(nativeBridge.removeAttachment!({ conversationId: 'a'.repeat(32), attachmentId: 'b'.repeat(32) })).rejects.toThrow('host_unavailable');
    await expect(nativeBridge.viewProcessedAttachment!({ conversationId: 'a'.repeat(32), requestId: 'c'.repeat(32), attachmentId: 'b'.repeat(32) })).rejects.toThrow('host_unavailable');
    await expect(nativeBridge.shellReady!({ protocolVersion: 1 })).rejects.toThrow('host_unavailable');
  });
});
