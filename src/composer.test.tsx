import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { StrictMode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, saved, selectClassification } from './test-fixtures';
import type { AttachmentSummary } from './types';

afterEach(() => { cleanup(); vi.restoreAllMocks(); });
async function ready() { await screen.findByText('Ready for separately reviewed requests.'); }
async function classify(label = 'Synthetic') {
  await userEvent.click(screen.getByRole('button', { name: 'Reasoning effort' }));
  await userEvent.click(screen.getByRole('menuitem', { name: /Classify content/ }));
  await userEvent.click(screen.getByRole('menuitemradio', { name: label }));
}

describe('v0.2.3 governed composer presentation (T-075/T-076)', () => {
  it('has a genuine nested classification menu and keeps effort and eligibility independent', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const trigger = screen.getByRole('button', { name: 'Reasoning effort' });
    expect(trigger.textContent).toContain('Medium');
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    await userEvent.click(trigger);
    const menu = screen.getByRole('menu', { name: 'Reasoning effort options' });
    expect(within(menu).getAllByRole('menuitemradio').map(item => item.textContent)).toEqual(['Minimal', 'Low', 'Medium', 'High', 'Extra high']);
    expect(within(menu).getByRole('menuitemradio', { name: 'Medium' }).getAttribute('aria-checked')).toBe('true');
    await userEvent.click(within(menu).getByRole('menuitemradio', { name: 'High' }));
    expect(trigger.textContent).toContain('High');
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    await classify();
    expect(trigger.textContent).toContain('High');
    expect(screen.getByLabelText('Content classification').textContent).toBe('Synthetic');
    expect(document.activeElement).toBe(trigger);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('supports nested arrows, Home/End, Escape, focus return and outside dismissal', async () => {
    render(<App bridge={fakeBridge().bridge}/>); await ready();
    const trigger = screen.getByRole('button', { name: 'Reasoning effort' }); trigger.focus();
    await userEvent.keyboard('{ArrowDown}');
    expect(document.activeElement).toBe(screen.getByRole('menuitemradio', { name: 'Minimal' }));
    await userEvent.keyboard('{End}{ArrowRight}');
    expect(document.activeElement).toBe(screen.getByRole('menuitemradio', { name: 'Synthetic' }));
    await userEvent.keyboard('{ArrowDown}');
    expect(document.activeElement).toBe(screen.getByRole('menuitemradio', { name: 'Eligible public' }));
    await userEvent.keyboard('{ArrowLeft}');
    expect(document.activeElement).toBe(screen.getByRole('menuitem', { name: /Classify content/ }));
    await userEvent.keyboard('{Escape}'); expect(document.activeElement).toBe(trigger);
    expect(screen.queryByRole('menu', { name: 'Reasoning effort options' })).toBeNull();
    await userEvent.click(trigger); await userEvent.click(screen.getByRole('textbox', { name: 'Message for Aster' }));
    expect(screen.queryByRole('menu', { name: 'Reasoning effort options' })).toBeNull();
  });

  it('keeps the plus visible but honestly unavailable when the host does not admit attachments', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const add = screen.getByRole('button', { name: 'Add attachments' }) as HTMLButtonElement;
    expect(add.disabled).toBe(true);
    expect(add.classList.contains('attachment-button')).toBe(true);
    expect(add.nextElementSibling?.classList.contains('fixed-model')).toBe(true);
    expect(add.title).toMatch(/unavailable/i);
    await userEvent.click(add); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(fake.bridge.createConversation).not.toHaveBeenCalled();
  });

  it('locks effort, classification and import behind a local dialogue', async () => {
    render(<App bridge={fakeBridge().bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: 'Reasoning effort' }));
    await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(screen.queryByRole('menu', { name: 'Reasoning effort options' })).toBeNull();
    expect((screen.getByRole('button', { name: 'Reasoning effort' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('button', { name: 'Add attachments' }) as HTMLButtonElement).disabled).toBe(true);
  });
});

// These explicit fake-host fixtures qualify renderer denial/state behaviour,
// never native import, PDF processing, visual review or provider compatibility.
function attachmentFixture(initial = [saved('a'.repeat(32), 'Eligible saved example')]) {
  const fake = fakeBridge({ ...baseline, capabilities: { ...baseline.capabilities, attachmentImport: true } }, initial);
  const item: AttachmentSummary = { id: 'b'.repeat(32), kind: 'text', label: 'Eligible example.txt', sourceBytes: 80, processedBytes: 80, pageCount: 0 };
  fake.bridge.selectAttachments = vi.fn(async () => [structuredClone(item)]);
  fake.bridge.removeAttachment = vi.fn(async () => ({ removed: true }));
  return { ...fake, item };
}
describe('explicit fake-host attachment intent/state denial fixtures (T-075/T-079/T-080)', () => {
  it('imports into a real conversation intent, preserves its draft and clears old classification', async () => {
    const fake = attachmentFixture([]); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: '  Exact synthetic draft\n🦊  ' } }); await classify();
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' }));
    await screen.findByRole('list', { name: 'Temporary attachments' });
    expect(area.value).toBe('  Exact synthetic draft\n🦊  ');
    expect(fake.bridge.selectAttachments).toHaveBeenCalledExactlyOnceWith({ conversationId: 'new_1' });
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled();
  });

  it('uses the complete host batch and removes only an opaque ID, with no filename/path authority', async () => {
    const fake = attachmentFixture(); render(<App bridge={fake.bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' }));
    await screen.findByText(fake.item.label);
    const replacement = { ...fake.item, id: 'c'.repeat(32), label: 'Complete replacement.pdf', kind: 'pdf' as const, pageCount: 2 };
    vi.mocked(fake.bridge.selectAttachments!).mockResolvedValue([replacement]);
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(replacement.label);
    expect(screen.queryByText(fake.item.label)).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: `Remove attachment ${replacement.label}` }));
    await waitFor(() => expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull());
    expect(fake.bridge.removeAttachment).toHaveBeenCalledExactlyOnceWith({ conversationId: 'a'.repeat(32), attachmentId: 'c'.repeat(32) });
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('revokes temporary IDs on a successful chat switch without losing bounded per-chat drafts', async () => {
    const fake = attachmentFixture([saved('a'.repeat(32), 'First', 2), saved('d'.repeat(32), 'Second', 1)]);
    render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: 'Keep first draft' } });
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    await userEvent.click(screen.getByRole('button', { name: /Second2 messages/ })); await screen.findByText(`Reply for ${'d'.repeat(32)}`);
    expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull();
    expect(fake.bridge.removeAttachment).toHaveBeenCalledExactlyOnceWith({ conversationId: 'a'.repeat(32), attachmentId: fake.item.id });
    await userEvent.click(screen.getByRole('button', { name: /First2 messages/ })); await screen.findByText(`Reply for ${'a'.repeat(32)}`);
    expect(area.value).toBe('Keep first draft'); expect(screen.queryByText(fake.item.label)).toBeNull();
  });

  it('can select locally without financial or credential readiness, but still cannot send', async () => {
    const fake = attachmentFixture(); vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...baseline, configured: false, availability: 'needs_credential', capabilities: { ...baseline.capabilities, attachmentImport: true } });
    render(<App bridge={fake.bridge}/>); await screen.findByText('Set up Meta in Settings before sending.');
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    expect(fake.bridge.configureCredential).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('fails closed on extra fields/unsafe labels, clears temporary IDs and preserves the exact draft', async () => {
    const fake = attachmentFixture(); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: '  Keep exact spacing  ' } });
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    vi.mocked(fake.bridge.selectAttachments!).mockResolvedValue([{ ...fake.item, path: 'C:\\private\\file.txt' }] as unknown as AttachmentSummary[]);
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' }));
    await screen.findByText(/invalid attachment metadata/);
    expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull(); expect(screen.queryByText(/C:\\private/)).toBeNull();
    expect(area.value).toBe('  Keep exact spacing  '); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    // Complete replacement is host-owned; the previous batch was discarded
    // before the second selector, not restored/revoked via old renderer IDs.
    expect(fake.bridge.removeAttachment).not.toHaveBeenCalled();
  });

  it('locks import and effort while the native picker is outstanding and does not auto-send', async () => {
    const fake = attachmentFixture(); let complete: ((items: AttachmentSummary[]) => void) | undefined;
    vi.mocked(fake.bridge.selectAttachments!).mockImplementation(() => new Promise(resolve => { complete = resolve; }));
    render(<App bridge={fake.bridge}/>); await ready(); await userEvent.click(screen.getByRole('button', { name: 'Add attachments' }));
    expect((screen.getByRole('button', { name: 'Add attachments' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('button', { name: 'Reasoning effort' }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.keyDown(window, { key: 'n', ctrlKey: true }); expect(fake.bridge.createConversation).not.toHaveBeenCalled();
    await act(async () => complete!([fake.item])); await screen.findByText(fake.item.label);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('sends opaque IDs only and discards them after declined review while retaining the unsent prompt', async () => {
    const fake = attachmentFixture(); vi.mocked(fake.bridge.sendMessage).mockRejectedValue('disclosure_denied');
    render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: 'Describe the eligible example.' } });
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    await selectClassification('synthetic'); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    await screen.findByText('The native disclosure review was declined. Nothing was sent.');
    expect(vi.mocked(fake.bridge.sendMessage).mock.calls[0][0].attachmentIds).toEqual([fake.item.id]);
    expect(JSON.stringify(vi.mocked(fake.bridge.sendMessage).mock.calls)).not.toMatch(/label|sourceBytes|processedBytes|path|base64|approved/);
    expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull(); expect(area.value).toBe('Describe the eligible example.');
    expect(fake.bridge.sendMessage).toHaveBeenCalledOnce();
  });

  it('does not restore active attachment IDs or classification on a renderer restart', async () => {
    const fake = attachmentFixture(); const view = render(<App bridge={fake.bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    view.unmount(); render(<App bridge={fake.bridge}/>); await ready();
    expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull();
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect(fake.bridge.selectAttachments).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('clears the complete previous batch while replacement is pending and keeps it discarded after cancellation', async () => {
    const fake = attachmentFixture(); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: 'Keep this exact synthetic draft.' } });
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    await selectClassification('synthetic');
    let finish: ((items: AttachmentSummary[]) => void) | undefined;
    vi.mocked(fake.bridge.selectAttachments!).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' }));
    expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull();
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    await act(async () => finish!([]));
    await screen.findByText('No replacement attachments were selected. The previous temporary attachments were discarded; your draft is unchanged and nothing was sent.');
    expect(area.value).toBe('Keep this exact synthetic draft.'); expect(screen.queryByText(fake.item.label)).toBeNull();
    expect(fake.bridge.removeAttachment).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    await selectClassification('synthetic'); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    await waitFor(() => expect(fake.bridge.sendMessage).toHaveBeenCalledOnce());
    expect(vi.mocked(fake.bridge.sendMessage).mock.calls[0][0].attachmentIds).toEqual([]);
  });

  it.each([
    ['attachment_capacity', 'The selected files or processed history exceed an Aster limit. No content was silently truncated; nothing was sent.'],
    ['attachment_unavailable', 'Attachment processing is unavailable. Temporary attachment identifiers were discarded; nothing was sent.'],
    ['attachment_convert_to_pdf', 'DOCX, XLSX and PPTX require safe external conversion to PDF. Convert the complete document with your own trusted tool, check every page, sheet, slide and print range, then attach the PDF. Aster does not run or require an Office converter.'],
  ])('uses safe British-English guidance for %s while retaining the draft and discarding prior IDs', async (code, text) => {
    const fake = attachmentFixture(); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: '  Keep exact spacing  ' } });
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(fake.item.label);
    await selectClassification('synthetic'); vi.mocked(fake.bridge.selectAttachments!).mockRejectedValue(code);
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText(text);
    expect(area.value).toBe('  Keep exact spacing  '); expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull();
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled();
  });
});

describe('bounded shell scheduling signal, not native paint proof (T-077)', () => {
  it('signals once after settled assets and two frames independently of status hydration, including StrictMode', async () => {
    const fake = fakeBridge(); fake.bridge.shellReady = vi.fn(async () => ({ accepted: true, visible: true }));
    vi.mocked(fake.bridge.getStatus).mockImplementation(() => new Promise(() => {}));
    const frames = new Map<number, FrameRequestCallback>(); let sequence = 0;
    vi.spyOn(window, 'requestAnimationFrame').mockImplementation(callback => { frames.set(++sequence, callback); return sequence; });
    vi.spyOn(window, 'cancelAnimationFrame').mockImplementation(id => { frames.delete(id); });
    const view = render(<StrictMode><App bridge={fake.bridge}/></StrictMode>);
    for (const image of document.querySelectorAll('img')) fireEvent.load(image);
    expect(fake.bridge.shellReady).not.toHaveBeenCalled();
    function frame() { const next = [...frames.entries()][0]; expect(next).toBeTruthy(); frames.delete(next[0]); act(() => next[1](0)); }
    frame(); expect(fake.bridge.shellReady).not.toHaveBeenCalled(); frame();
    await waitFor(() => expect(fake.bridge.shellReady).toHaveBeenCalledExactlyOnceWith({ protocolVersion: 1 }));
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled();
    view.unmount(); expect(frames.size).toBe(0);
  });

  it('cleans asset listeners and pending frames on unmount without signalling', async () => {
    const fake = fakeBridge(); fake.bridge.shellReady = vi.fn(async () => ({ accepted: true, visible: true }));
    const frames = new Map<number, FrameRequestCallback>(); let sequence = 0;
    vi.spyOn(window, 'requestAnimationFrame').mockImplementation(callback => { frames.set(++sequence, callback); return sequence; });
    vi.spyOn(window, 'cancelAnimationFrame').mockImplementation(id => { frames.delete(id); });
    const view = render(<App bridge={fake.bridge}/>); await ready();
    for (const image of document.querySelectorAll('img')) fireEvent.error(image);
    view.unmount(); expect(frames.size).toBe(0); expect(fake.bridge.shellReady).not.toHaveBeenCalled();
  });
});

function processedFixture(kind: AttachmentSummary['kind'] = 'pdf') {
  const conversation = saved('a'.repeat(32), 'Saved synthetic document');
  const item: AttachmentSummary = { id: 'b'.repeat(32), kind, label: kind === 'pdf' ? 'Synthetic pages.pdf' : 'Synthetic example.txt', sourceBytes: 100, processedBytes: 80, pageCount: kind === 'pdf' ? 2 : kind === 'image' ? 1 : 0 };
  conversation.entries[0].processedAttachments = [item];
  if (kind === 'text') conversation.entries[0].content += '\n\nSynthetic processed text content.';
  const fake = fakeBridge(baseline, [conversation]);
  fake.bridge.viewProcessedAttachment = vi.fn(async () => ({ viewed: true }));
  return { ...fake, item, conversation };
}

describe('display-only processed-history references and native local view (T-079/T-080)', () => {
  it('opens processed pages through the closed native local-view intent without restoring or sending originals', async () => {
    const fake = processedFixture(); render(<App bridge={fake.bridge}/>); await ready();
    const button = screen.getByRole('button', { name: `View processed pages for ${fake.item.label}` });
    expect(screen.getByText(/Reattach the original PDF for another complete native reading/)).toBeTruthy();
    expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull();
    await userEvent.click(button);
    await waitFor(() => expect(fake.bridge.viewProcessedAttachment).toHaveBeenCalledExactlyOnceWith({ conversationId: fake.conversation.id, requestId: fake.conversation.entries[0].requestId, attachmentId: fake.item.id }));
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Continue this synthetic conversation.' } });
    await selectClassification('synthetic'); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    await waitFor(() => expect(fake.bridge.sendMessage).toHaveBeenCalledOnce());
    expect(vi.mocked(fake.bridge.sendMessage).mock.calls[0][0].attachmentIds).toEqual([]);
    expect(JSON.stringify(vi.mocked(fake.bridge.sendMessage).mock.calls)).not.toContain(fake.item.id);
  });

  it('retains processed text metadata/content in the message without offering an unavailable page viewer', async () => {
    const fake = processedFixture('text');
    render(<App bridge={fake.bridge}/>); await ready();
    expect(screen.getByText('Processed text · local history only')).toBeTruthy();
    expect(screen.getByText(/Synthetic processed text content\./)).toBeTruthy();
    expect(screen.getByText('Processed text is included in the message above.')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /View processed text|View processed pages/ })).toBeNull();
    expect(fake.bridge.viewProcessedAttachment).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('keeps image/PDF local-view buttons honestly disabled without the native route', async () => {
    const fake = processedFixture('image'); delete fake.bridge.viewProcessedAttachment;
    render(<App bridge={fake.bridge}/>); await ready();
    const button = screen.getByRole('button', { name: `View processed pages for ${fake.item.label}` }) as HTMLButtonElement;
    expect(button.disabled).toBe(true); await userEvent.click(button); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('blocks local view while a dialogue is open and while the native viewer is outstanding', async () => {
    const fake = processedFixture(); let finish: ((value: { viewed: boolean }) => void) | undefined;
    vi.mocked(fake.bridge.viewProcessedAttachment!).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
    render(<App bridge={fake.bridge}/>); await ready();
    const button = screen.getByRole('button', { name: `View processed pages for ${fake.item.label}` }) as HTMLButtonElement;
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); expect(button.disabled).toBe(true);
    await userEvent.click(screen.getByRole('button', { name: 'Close dialogue' }));
    await userEvent.click(button); expect(button.disabled).toBe(true);
    fireEvent.keyDown(window, { key: 'n', ctrlKey: true }); expect(fake.bridge.createConversation).not.toHaveBeenCalled();
    expect((screen.getByRole('button', { name: 'Reasoning effort' }) as HTMLButtonElement).disabled).toBe(true);
    await act(async () => finish!({ viewed: false }));
    await waitFor(() => expect(button.disabled).toBe(false));
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('rejects a local-view receipt containing approval authority or arbitrary data', async () => {
    const fake = processedFixture();
    vi.mocked(fake.bridge.viewProcessedAttachment!).mockResolvedValue({ viewed: true, approved: true, path: 'C:\\private.pdf' } as { viewed: boolean });
    render(<App bridge={fake.bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: `View processed pages for ${fake.item.label}` }));
    await screen.findByText('The processed attachment is unavailable or no longer belongs to this saved request. Nothing was sent and no original file was reconstructed.');
    expect(screen.queryByText(/C:\\private/)).toBeNull(); expect(screen.queryByRole('list', { name: 'Temporary attachments' })).toBeNull();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
  });
});
