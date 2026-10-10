import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { GenerateCustomization } from './GenerateCustomization';
import { Customization } from './Customization';
import { App } from './App';
import { parseGeneratedDraft, validGenerationDescription, validGenerationReceipt } from './generation-ui';
import { baseline, fakeBridge, saved } from './test-fixtures';
import type { CustomizationDraft, CustomizationSnapshot, StreamEvent } from './types';

const draft: CustomizationDraft = { kind: 'skill', slug: 'synthetic-draft', name: 'Synthetic draft', description: 'Public fixture only', instructions: '<script>Inert instructions only</script>' };
afterEach(() => cleanup());
function fixture() {
  const fake = fakeBridge(); let callback: (event: StreamEvent) => void = () => {};
  let requestId = '';
  const unsubscribe = vi.fn();
  fake.bridge.subscribe = vi.fn(async value => { callback = value; return unsubscribe; });
  fake.bridge.generateCustomization = vi.fn(async input => { requestId = input.requestId; return { conversationId: 'generation_conversation', requestId, accepted: true as const }; });
  const onDraft = vi.fn(), onClose = vi.fn(), onBusyChange = vi.fn();
  const emit = (type: 'content' | 'terminal', value: string, overrides: Partial<StreamEvent> = {}) => callback({ requestId, conversationId: 'generation_conversation', type, ...(type === 'content' ? { text: value } : { status: value }), ...overrides } as StreamEvent);
  return { ...fake, onDraft, onClose, onBusyChange, unsubscribe, emit, setRequestId: (value: string) => { requestId = value; }, callback: () => callback };
}
async function description() {
  fireEvent.change(screen.getByRole('textbox', { name: 'Generation description' }), { target: { value: 'SYNTHETIC_DESCRIPTION_ONLY' } });
  fireEvent.change(screen.getByRole('combobox', { name: 'Generation content classification' }), { target: { value: 'synthetic' } });
}
async function start() { await description(); await userEvent.click(screen.getByRole('button', { name: 'Review and generate in Windows' })); await screen.findByText(/Waiting for the complete request-bound response/); }
describe('Muse-assisted inert drafts are request-bound and explicitly saved (T-089/T-093/T-096)', () => {
  it('requires bounded descriptions, exact receipts and one strict duplicate-free flat package of the requested kind', () => {
    expect(validGenerationDescription('😀'.repeat(4000))).toBe(true); expect(validGenerationDescription('😀'.repeat(4001))).toBe(false);
    for (const value of ['', ' ', '\0', '\ud800', 'text\u0001']) expect(validGenerationDescription(value)).toBe(false);
    expect(validGenerationReceipt({ conversationId: 'g', requestId: 'r', accepted: true }, 'r')).toBe(true);
    for (const receipt of [{ conversationId: 'g', requestId: 'other', accepted: true }, { conversationId: 'g', requestId: 'r', accepted: false }, { conversationId: 'g', requestId: 'r', accepted: true, approved: true }]) expect(validGenerationReceipt(receipt, 'r')).toBe(false);
    expect(parseGeneratedDraft(JSON.stringify(draft), 'skill')).toEqual(draft);
    for (const text of ['```json\n' + JSON.stringify(draft) + '\n```', JSON.stringify({ ...draft, kind: 'plugin' }), JSON.stringify({ ...draft, approved: true }), JSON.stringify({ ...draft, instructions: { command: 'run' } }), JSON.stringify(draft).replace('"kind":"skill"', '"kind":"skill","kind":"skill"'), JSON.stringify(draft).replace('"kind":"skill"', '"kind":"skill","k\\u0069nd":"skill"'), JSON.stringify({ ...draft, instructions: 'x'.repeat(12001) }), 'x'.repeat(65537)]) expect(parseGeneratedDraft(text, 'skill')).toBeNull();
  });
  it('does not infer consent, reads no local profile or repository, and never auto-saves or enables a completed draft', async () => {
    const fake = fixture(); fake.bridge.getPersonalization = vi.fn(); fake.bridge.getGitHubStatus = vi.fn(); fake.bridge.readGitHubFile = vi.fn(); fake.bridge.getCustomizationPackage = vi.fn(); fake.bridge.saveCustomization = vi.fn(); fake.bridge.setCustomizationEnabled = vi.fn();
    render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>);
    expect(fake.bridge.subscribe).not.toHaveBeenCalled(); expect(fake.bridge.generateCustomization).not.toHaveBeenCalled();
    fireEvent.change(screen.getByRole('textbox', { name: 'Generation description' }), { target: { value: 'SYNTHETIC_DESCRIPTION_ONLY' } });
    expect((screen.getByRole('button', { name: 'Review and generate in Windows' }) as HTMLButtonElement).disabled).toBe(true);
    await start();
    const input = vi.mocked(fake.bridge.generateCustomization!).mock.calls[0][0]; expect(Object.keys(input).sort().join('|')).toBe('dataClass|description|kind|requestId'); expect(input).toEqual({ requestId: expect.stringMatching(/^[a-f0-9]{32}$/), kind: 'skill', description: 'SYNTHETIC_DESCRIPTION_ONLY', dataClass: 'synthetic' });
    await act(async () => { fake.emit('content', JSON.stringify(draft)); }); expect(fake.onDraft).not.toHaveBeenCalled();
    await act(async () => { fake.emit('terminal', 'completed'); }); expect(fake.onDraft).toHaveBeenCalledWith(draft); expect(fake.unsubscribe).toHaveBeenCalledOnce();
    for (const method of [fake.bridge.getPersonalization, fake.bridge.getGitHubStatus, fake.bridge.readGitHubFile, fake.bridge.getCustomizationPackage, fake.bridge.saveCustomization, fake.bridge.setCustomizationEnabled, fake.bridge.sendMessage, fake.bridge.listConversations, fake.bridge.loadConversation]) expect(method).not.toHaveBeenCalled();
    expect(document.querySelector('script')).toBeNull();
  });
  it('accepts an early completed terminal only after the exact late receipt binds its conversation', async () => {
    const fake = fixture();
    fake.bridge.generateCustomization = vi.fn(async input => { fake.setRequestId(input.requestId); fake.emit('content', JSON.stringify(draft)); fake.emit('terminal', 'completed'); expect(fake.onDraft).not.toHaveBeenCalled(); return { conversationId: 'generation_conversation', requestId: input.requestId, accepted: true as const }; });
    render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>); await description(); await userEvent.click(screen.getByRole('button', { name: 'Review and generate in Windows' }));
    await waitFor(() => expect(fake.onDraft).toHaveBeenCalledWith(draft)); expect(fake.bridge.cancelMessage).not.toHaveBeenCalled();
  });
  it.each(['cancelled','truncated','refused','failed'])('rejects the %s terminal without creating a partial package or retrying', async status => {
    const fake = fixture(); render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>); await start();
    await act(async () => { fake.emit('content', JSON.stringify(draft)); fake.emit('terminal', status); }); await screen.findByText(/Generation did not complete successfully/);
    expect(fake.onDraft).not.toHaveBeenCalled(); expect(fake.bridge.generateCustomization).toHaveBeenCalledOnce();
  });
  it('rejects malformed or oversized content, ignores other requests/conversations and discards late cancelled results', async () => {
    const fake = fixture(); render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>); await start();
    await act(async () => { fake.emit('content', JSON.stringify(draft), { requestId: 'unrelated' }); fake.emit('terminal', 'completed', { conversationId: 'other_conversation' }); }); expect(fake.onDraft).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Stop generation' })); expect(fake.bridge.cancelMessage).toHaveBeenCalledOnce();
    await act(async () => { fake.emit('content', JSON.stringify(draft)); fake.emit('terminal', 'completed'); }); expect(fake.onDraft).not.toHaveBeenCalled();
    await start(); await act(async () => { fake.emit('content', 'x'.repeat(65537)); }); await screen.findByText(/exceeded 64 KiB/); expect(fake.onDraft).not.toHaveBeenCalled(); expect(fake.bridge.cancelMessage).toHaveBeenCalledTimes(2);
    await start(); await act(async () => { fake.callback()({ requestId: vi.mocked(fake.bridge.generateCustomization!).mock.calls.at(-1)![0].requestId, conversationId: 'generation_conversation', type: 'content', text: 'fragment', approved: true } as unknown as StreamEvent); }); await screen.findByText(/matching generation stream violated its contract/); expect(fake.onDraft).not.toHaveBeenCalled();
  });
  it('cancels on unmount and cannot reuse a late accepted generation receipt', async () => {
    const fake = fixture(); let resolveReceipt: ((value: { conversationId: string; requestId: string; accepted: true }) => void) | undefined;
    fake.bridge.generateCustomization = vi.fn(input => { fake.setRequestId(input.requestId); return new Promise<{ conversationId: string; requestId: string; accepted: true }>(resolve => { resolveReceipt = resolve; }); });
    const view = render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>); await description(); await userEvent.click(screen.getByRole('button', { name: 'Review and generate in Windows' }));
    await waitFor(() => expect(fake.bridge.generateCustomization).toHaveBeenCalledOnce()); view.unmount();
    await act(async () => { fake.emit('content', JSON.stringify(draft)); fake.emit('terminal', 'completed'); resolveReceipt!({ conversationId: 'generation_conversation', requestId: vi.mocked(fake.bridge.generateCustomization!).mock.calls[0][0].requestId, accepted: true }); });
    expect(fake.bridge.cancelMessage).toHaveBeenCalledOnce(); expect(fake.unsubscribe).toHaveBeenCalledOnce(); expect(fake.onDraft).not.toHaveBeenCalled();
  });
  it('opens a reviewed unsaved editor and persists only under a separate explicit Save', async () => {
    const fake = fixture(); let catalogue: CustomizationSnapshot = { revision: 0, entries: [] };
    fake.bridge.getCustomization = vi.fn(async () => structuredClone(catalogue)); fake.bridge.getCustomizationPackage = vi.fn(); fake.bridge.setCustomizationEnabled = vi.fn(); fake.bridge.importCustomization = vi.fn();
    fake.bridge.saveCustomization = vi.fn(async input => { catalogue = { revision: 1, entries: [{ id: `skill:${input.package.slug}`, kind: 'skill', name: input.package.name, description: input.package.description, enabled: false, readOnly: false, availability: 'instructions-only', entryRevision: 1 }] }; });
    render(<Customization bridge={fake.bridge} locked={false} credentialValidated/>); await waitFor(() => expect((screen.getByRole('button', { name: 'Generate with Muse Spark' }) as HTMLButtonElement).disabled).toBe(false));
    await userEvent.click(screen.getByRole('button', { name: 'Generate with Muse Spark' })); await start(); await act(async () => { fake.emit('content', JSON.stringify(draft)); fake.emit('terminal', 'completed'); });
    await screen.findByDisplayValue(draft.instructions); expect(fake.bridge.saveCustomization).not.toHaveBeenCalled(); expect(fake.bridge.setCustomizationEnabled).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Save package revision' })); await screen.findByText('A new immutable package revision was saved. No execution authority was granted.');
    expect(fake.bridge.saveCustomization).toHaveBeenCalledWith({ expectedRevision: 0, package: draft }); expect(fake.bridge.setCustomizationEnabled).not.toHaveBeenCalled(); expect(document.querySelector('.customization-page script')).toBeNull();
  });
  it.each(['missing','unverified','invalid','unavailable'] as const)('fresh %s proof denies generation before subscribing or creating its conversation', async credentialValidation => {
    const fake = fixture(); vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...baseline, credentialValidation, configured: credentialValidation !== 'missing', availability: 'needs_credential' });
    render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>); await description(); await userEvent.click(screen.getByRole('button', { name: 'Review and generate in Windows' }));
    await screen.findByText(/Configure and validate a Meta API key in Providers/); expect(fake.bridge.subscribe).not.toHaveBeenCalled(); expect(fake.bridge.generateCustomization).not.toHaveBeenCalled(); expect(fake.bridge.cancelMessage).not.toHaveBeenCalled();
  });
  it('denies generation when native terms, budget or active-request admission is not ready', async () => {
    const fake = fixture(); vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...baseline, availability: 'needs_budget', budget: { ...baseline.budget, configured: false } });
    render(<GenerateCustomization bridge={fake.bridge} kind="skill" locked={false} onDraft={fake.onDraft} onClose={fake.onClose} onBusyChange={fake.onBusyChange}/>); await description(); await userEvent.click(screen.getByRole('button', { name: 'Review and generate in Windows' }));
    await screen.findByText(/native provider admission status is not ready/); expect(fake.bridge.generateCustomization).not.toHaveBeenCalled(); expect(fake.bridge.subscribe).not.toHaveBeenCalled();
  });
  it('refreshes only metadata after an owned cancelled generation terminal while preserving the main chat and temporary imports', async () => {
    const status = { ...baseline, capabilities: { ...baseline.capabilities, attachmentImport: true } };
    const fake = fakeBridge(status, [saved('main_chat', 'Main chat retained')]);
    const listeners = new Set<(event: StreamEvent) => void>();
    fake.bridge.subscribe = vi.fn(async callback => { listeners.add(callback); return () => { listeners.delete(callback); }; });
    vi.mocked(fake.bridge.getStatus).mockImplementation(async () => structuredClone(status));
    let generationRequest = '';
    fake.bridge.generateCustomization = vi.fn(async input => { generationRequest = input.requestId; status.activeRequest = true; status.availability = 'busy'; fake.records.set('generated_chat', saved('generated_chat', 'Generated synthetic package')); return { requestId: input.requestId, conversationId: 'generated_chat', accepted: true as const }; });
    const catalogue = { revision: 0, entries: [] };
    fake.bridge.getCustomization = vi.fn(async () => catalogue); fake.bridge.getCustomizationPackage = vi.fn(); fake.bridge.saveCustomization = vi.fn(); fake.bridge.setCustomizationEnabled = vi.fn(); fake.bridge.importCustomization = vi.fn();
    fake.bridge.selectAttachments = vi.fn(async () => [{ id: 'a'.repeat(32), kind: 'text' as const, label: 'Synthetic retained.txt', sourceBytes: 80, processedBytes: 80, pageCount: 0 }]); fake.bridge.removeAttachment = vi.fn(async () => ({ removed: true }));
    render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'MAIN_DRAFT_RETAINED' } }); await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText('Synthetic retained.txt');
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Customize' })); await userEvent.click(screen.getByRole('button', { name: 'Open Customize workspace' })); await userEvent.click(await screen.findByRole('button', { name: 'Generate with Muse Spark' })); await start();
    await userEvent.click(screen.getByRole('button', { name: 'Stop generation' })); await waitFor(() => expect(fake.bridge.cancelMessage).toHaveBeenCalledOnce());
    await userEvent.click(screen.getByRole('button', { name: 'Close generation editor' }));
    const before = vi.mocked(fake.bridge.getStatus).mock.calls.length;
    await act(async () => { for (const callback of listeners) callback({ requestId: 'unknown', conversationId: 'other', type: 'terminal', status: 'completed' }); }); expect(fake.bridge.getStatus).toHaveBeenCalledTimes(before);
    status.activeRequest = false; status.availability = 'ready';
    await act(async () => { for (const callback of listeners) callback({ requestId: generationRequest, conversationId: 'generated_chat', type: 'terminal', status: 'cancelled' }); });
    await waitFor(() => expect((screen.getByRole('button', { name: 'Return to chat' }) as HTMLButtonElement).disabled).toBe(false));
    await userEvent.click(screen.getByRole('button', { name: 'Return to chat' }));
    expect(screen.getByText('Reply for main_chat')).toBeTruthy(); expect(screen.getByDisplayValue('MAIN_DRAFT_RETAINED')).toBeTruthy(); expect(screen.getByText('Synthetic retained.txt')).toBeTruthy(); expect(screen.getByText('Generated synthetic package')).toBeTruthy();
    expect(fake.bridge.loadConversation).toHaveBeenCalledExactlyOnceWith({ conversationId: 'main_chat' }); expect(fake.bridge.removeAttachment).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.saveCustomization).not.toHaveBeenCalled();
  });
});
