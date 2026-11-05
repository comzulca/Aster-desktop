import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, saved, selectClassification } from './test-fixtures';
import { DEFAULT_MODEL, type Conversation } from './types';

afterEach(() => cleanup());
async function ready() { await screen.findByText('Ready for separately reviewed requests.'); }
async function prepare(text = 'A wholly synthetic example') {
  fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: text } });
  await selectClassification('synthetic');
  await waitFor(() => expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(false));
}
describe('Aster v0.2 renderer journeys (T-049/T-060/T-061)', () => {
  it('explains expired policy without claiming a Windows or credential failure (T-097)', async () => {
    const fake = fakeBridge();
    vi.mocked(fake.bridge.acceptContributorTerms).mockRejectedValue('contributor_policy_expired');
    render(<App bridge={fake.bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await userEvent.click(screen.getByRole('button', { name: 'Review terms and eligibility in Windows' }));
    await screen.findByText("This build's Contributor policy review has expired. Install a reviewed engineering update; accepting again cannot extend it.");
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(fake.bridge.configureCredential).not.toHaveBeenCalled();
  });
  it('describes current v0.2.4 boundaries in News and separates GitHub reading from disabled chat tools', async () => {
    const fake = fakeBridge(); fake.bridge.verifyCredential = vi.fn(async () => ({ state: 'valid' as const }));
    render(<App bridge={fake.bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: 'Help' })); await userEvent.click(screen.getByRole('menuitem', { name: "What's New" }));
    const news = screen.getByRole('dialog', { name: "What's New" });
    for (const text of [/v0\.2\.4: the owner-supplied interface/, /current-key authentication proof/, /Personalization is exclusively local/, /fresh ordinary local conversation retains the description, prompt and response/, /Save and enable remain separate/, /Enable its catalogue preference before Connect/, /Expiry or restart requires deliberate reconnect, not automatic refresh/, /File changes, issues, pull requests and merges remain unavailable/, /Automatic context compaction is unfinished/, /do not establish compositor completeness or a flicker cure/]) expect(within(news).getByText(text)).toBeTruthy();
    expect(news.textContent).not.toContain('Branding, caption, send/stop geometry and sky-blue colours are unchanged.');
    await userEvent.click(within(news).getByRole('button', { name: 'Close dialogue' }));
    await userEvent.click(screen.getByRole('button', { name: 'Help' })); await userEvent.click(screen.getByRole('menuitem', { name: 'Documentation' }));
    const documentation = screen.getByRole('dialog', { name: 'Documentation' });
    expect(within(documentation).getByText(/Main-chat\/model tools, local project reads and file writes/).textContent).toContain('separate GitHub popup supports explicit native-reviewed account/repository selection and bounded file reading only');
    expect(documentation.textContent).not.toContain('No project reads, file writes, voice, browser, shell, web or model tools are enabled.');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled(); expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
  });
  it.each([
    ['valid', 'Authentication verified for the configured key; billing and capacity are separate.'],
    ['unverified', 'Validate the configured Meta API key in Settings before starting a conversation.'],
    ['invalid', 'Meta did not accept this key. Replace it in Settings before starting a conversation.'],
    ['missing', 'Configure and validate a Meta API key in Settings before starting a conversation.'],
    ['unavailable', 'Key validation is unavailable. Review Settings; no key has been labelled invalid.'],
  ] as const)('shows %s key proof separately from unverified inference and billing without probing', async (credentialValidation, text) => {
    const fake = fakeBridge({ ...baseline, credentialValidation, configured: credentialValidation !== 'missing', availability: credentialValidation === 'valid' ? 'ready' : 'needs_credential' });
    fake.bridge.verifyCredential = vi.fn(async () => ({ state: 'valid' as const }));
    render(<App bridge={fake.bridge}/>);
    if (credentialValidation === 'valid') await ready(); else await screen.findByRole('button', { name: 'Open Settings' });
    await userEvent.click(screen.getByRole('button', { name: 'Help' })); await userEvent.click(screen.getByRole('menuitem', { name: 'System Status' }));
    const status = screen.getByRole('dialog', { name: 'System Status' });
    expect(within(status).getByText(`Authentication: ${text}`)).toBeTruthy();
    expect(within(status).getByText('Live inference, billing and capacity: unverified. No request or probe is made by this view.')).toBeTruthy();
    expect(within(status).getByText(/GitHub uses a separate reviewed native account\/read route/).textContent).toContain('Mutations and merges remain unavailable.');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled(); expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
  });
  it('has the requested UK layout, no suggestions, top version/status strip or alternative models', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    expect(screen.getByRole('heading', { name: 'Ready when you are.' })).toBeTruthy();
    expect(screen.getByRole('img', { name: 'Aster AI' }).getAttribute('src')).toContain('aster-wordmark');
    expect(screen.getByRole('img', { name: 'English (United Kingdom)' })).toBeTruthy();
    for (const name of ['File','Edit','View','Help']) expect(screen.getByRole('button', { name })).toBeTruthy();
    expect(screen.queryByRole('combobox', { name: 'Model' })).toBeNull();
    expect((screen.getByRole('button', { name: 'Add attachments' }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.queryByRole('button', { name: /Project|proposal/i })).toBeNull();
    expect(document.querySelector('.conversation-heading')).toBeNull(); expect(document.querySelector('.starter-actions')).toBeNull();
    expect(document.querySelector('.titlebar')?.textContent).not.toMatch(/Aster|v0\.|Windows|session/);
    expect(screen.queryByText(/Nemotron|Glimmer|NVIDIA/)).toBeNull();
    expect(screen.queryAllByRole('combobox')).toHaveLength(0);
  });
  it('requires an explicit classification and sends only the closed fixed-model intent', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: '  Exact synthetic text  ' } });
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    await prepare('  Exact synthetic text  '); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    const input = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    expect(input.message).toBe('  Exact synthetic text  '); expect(input.modelId).toBe(DEFAULT_MODEL); expect(input.dataClass).toBe('synthetic'); expect(input.reasoningEffort).toBe('medium');
    expect(input.requestId).toMatch(/^[a-f0-9]{32}$/);
    expect(input.attachmentIds).toEqual([]);
    expect(Object.keys(input).sort()).toEqual(['attachmentIds','conversationId','dataClass','message','modelId','reasoningEffort','requestId']);
    expect(fake.bridge.createConversation).toHaveBeenCalledOnce();
  });
  it('hydrates ordered saved history, navigates and restores it across renderer restarts without network', async () => {
    const fake = fakeBridge(baseline, [saved('older', 'Earlier conversation', 1), saved('newer', 'Latest conversation', 9)]);
    const first = render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for newer');
    const nav = screen.getByRole('navigation', { name: 'Saved conversations' });
    const rows = within(nav).getAllByRole('button'); expect(rows[0].textContent).toContain('Latest conversation'); expect(rows[0].getAttribute('aria-current')).toBe('page');
    await userEvent.click(within(nav).getByRole('button', { name: /Earlier conversation/ })); await screen.findByText('Reply for older');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); first.unmount(); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for newer');
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(localStorage.length).toBe(0);
  });
  it('preserves old records when creating a new conversation and searching titles', async () => {
    const fake = fakeBridge(baseline, [saved('keep', 'Keep this conversation', 3)]); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for keep');
    await userEvent.click(screen.getByRole('button', { name: 'New conversation' })); await screen.findByRole('heading', { name: 'Ready when you are.' });
    expect(fake.records.has('keep')).toBe(true); expect(fake.bridge.deleteConversation).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Search conversations' }));
    await userEvent.type(screen.getByRole('searchbox', { name: 'Search saved conversations' }), 'Keep this');
    const nav = screen.getByRole('navigation', { name: 'Saved conversations' }); expect(within(nav).getAllByRole('button')).toHaveLength(1);
  });
  it('locks history/new-chat during a global request and keeps Stop bound to it', async () => {
    const fake = fakeBridge(baseline, [saved('one', 'First conversation', 5), saved('two', 'Second conversation', 1)]); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for one');
    await prepare(); await userEvent.click(screen.getByRole('button', { name: 'Send message' })); const input = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    for (const row of within(screen.getByRole('navigation', { name: 'Saved conversations' })).getAllByRole('button')) expect((row as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('button', { name: 'New conversation' }) as HTMLButtonElement).disabled).toBe(true);
    await userEvent.keyboard('{Control>}n{/Control}'); expect(fake.bridge.createConversation).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Stop response' })); expect(fake.bridge.cancelMessage).toHaveBeenCalledWith(input.requestId);
    act(() => fake.emit({ requestId: input.requestId, conversationId: input.conversationId, type: 'terminal', status: 'cancelled' }));
    await screen.findByText(/Response stopped/); await waitFor(() => expect(screen.queryByRole('button', { name: 'Stop response' })).toBeNull());
  });
  it('renders hostile stream text inertly and ignores stale, duplicate and late events', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready(); await prepare(); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    const input = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    act(() => fake.emit({ requestId: 'wrong', conversationId: input.conversationId, type: 'content', text: 'STALE' }));
    act(() => fake.emit({ requestId: input.requestId, conversationId: input.conversationId, type: 'content', text: '<img src=x onerror=alert(1)>' }));
    expect(screen.getByText('<img src=x onerror=alert(1)>')).toBeTruthy(); expect(document.querySelector('.message-text img')).toBeNull(); expect(screen.queryByText('STALE')).toBeNull();
    act(() => fake.emit({ requestId: input.requestId, conversationId: input.conversationId, type: 'terminal', status: 'completed' }));
    act(() => fake.emit({ requestId: input.requestId, conversationId: input.conversationId, type: 'terminal', status: 'failed' }));
    act(() => fake.emit({ requestId: input.requestId, conversationId: input.conversationId, type: 'content', text: 'LATE' }));
    await screen.findByText('Response completed'); expect(screen.queryByText('LATE')).toBeNull(); expect(screen.queryByText(/Response failed/)).toBeNull();
  });
  it.each(['truncated','refused','interrupted'] as const)('restores %s display without replaying or regaining authority', async state => {
    const record = saved('restored', 'Saved incomplete request'); record.entries[1].state = state;
    const fake = fakeBridge(baseline, [record]); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for restored');
    expect(screen.queryByRole('button', { name: 'Stop response' })).toBeNull(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(screen.getByText(/not reused as context|Not reused as context/i)).toBeTruthy();
  });
  it('fails closed for missing or incompatible hosts and corrupt persisted metadata', async () => {
    const fake = fakeBridge(); vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...baseline, protocolVersion: 2 } as unknown as typeof baseline);
    render(<App bridge={fake.bridge}/>); await screen.findByText('Native host unavailable or incompatible.');
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true); expect(fake.bridge.listConversations).not.toHaveBeenCalled();
    cleanup(); const corrupted = fakeBridge(baseline, [saved('c', 'Corrupted')]);
    vi.mocked(corrupted.bridge.loadConversation).mockResolvedValue({ ...saved('c', 'Corrupted'), approved: true } as unknown as Conversation);
    render(<App bridge={corrupted.bridge}/>); await screen.findByText('Native host unavailable or incompatible.'); expect(corrupted.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('does not replace an already received terminal outcome when the send receipt fails late', async () => {
    const fake = fakeBridge(); let rejectSend: ((reason: Error) => void) | undefined;
    vi.mocked(fake.bridge.sendMessage).mockImplementation(input => {
      const item = fake.records.get(input.conversationId)!;
      item.entries.push({ id: 'final_user', role: 'user', content: input.message, requestId: input.requestId, state: 'completed' }, { id: 'final_answer', role: 'assistant', content: 'Final reply', requestId: input.requestId, state: 'completed' });
      return new Promise((_, reject) => { rejectSend = reject; });
    });
    render(<App bridge={fake.bridge}/>); await ready(); await prepare(); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    const input = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    act(() => fake.emit({ conversationId: input.conversationId, requestId: input.requestId, type: 'terminal', status: 'completed' }));
    await screen.findByText('Final reply'); act(() => rejectSend!(new Error('private provider error body')));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Stop response' })).toBeNull());
    expect(screen.getByText('Response completed')).toBeTruthy(); expect(screen.queryByText(/private provider|request was not accepted|Response failed/)).toBeNull();
  });
  it('preserves the exact unsent draft and classification when native disclosure is declined', async () => {
    const fake = fakeBridge(); vi.mocked(fake.bridge.sendMessage).mockRejectedValue('disclosure_denied');
    const message = '  A synthetic draft\nwith exact spacing and café 🦊  ';
    render(<App bridge={fake.bridge}/>); await ready(); await prepare(message);
    await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    await screen.findByText('The native disclosure review was declined. Nothing was sent.');
    await waitFor(() => expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(false));
    expect((screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement).value).toBe(message);
    expect(screen.getByLabelText('Content classification').textContent).toBe('Synthetic');
    expect([...fake.records.values()][0].entries).toEqual([]);
    expect(document.querySelector('.message-text')).toBeNull();
    expect(fake.bridge.sendMessage).toHaveBeenCalledOnce();
    expect(fake.bridge.cancelMessage).not.toHaveBeenCalled();
  });
  it.each(['accepted', 'rejected'] as const)('keeps a newer draft after a terminal event precedes a late %s receipt', async receipt => {
    const fake = fakeBridge();
    let resolveSend: ((value: { requestId: string; accepted: boolean }) => void) | undefined;
    let rejectSend: ((reason: Error) => void) | undefined;
    vi.mocked(fake.bridge.sendMessage).mockImplementation(input => {
      const item = fake.records.get(input.conversationId)!;
      item.entries.push({ id: 'completed_user', role: 'user', content: input.message, requestId: input.requestId, state: 'completed' }, { id: 'completed_answer', role: 'assistant', content: 'Completed original reply', requestId: input.requestId, state: 'completed' });
      return new Promise((resolve, reject) => { resolveSend = resolve; rejectSend = reject; });
    });
    render(<App bridge={fake.bridge}/>); await ready(); await prepare('Original synthetic prompt');
    await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    const input = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    act(() => fake.emit({ conversationId: input.conversationId, requestId: input.requestId, type: 'terminal', status: 'completed' }));
    await screen.findByText('Completed original reply');
    await waitFor(() => expect((screen.getByRole('button', { name: 'New conversation' }) as HTMLButtonElement).disabled).toBe(false));
    const newerDraft = '  A different draft after completion  ';
    await prepare(newerDraft);
    await selectClassification('public_eligible');
    await act(async () => {
      if (receipt === 'accepted') resolveSend!({ requestId: input.requestId, accepted: true });
      else rejectSend!(new Error('Private provider body'));
      await Promise.resolve();
    });
    expect((screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement).value).toBe(newerDraft);
    expect(screen.getByLabelText('Content classification').textContent).toBe('Eligible public');
    expect(screen.getByText('Response completed')).toBeTruthy();
    expect(screen.queryByText(/Private provider|request was not accepted|Response failed/)).toBeNull();
    expect(fake.bridge.sendMessage).toHaveBeenCalledOnce();
  });
  it('maps untrusted request errors to local English without provider-body disclosure or retries', async () => {
    const fake = fakeBridge(); vi.mocked(fake.bridge.sendMessage).mockRejectedValue(new Error('Bearer private-key provider body'));
    render(<App bridge={fake.bridge}/>); await ready(); await prepare(); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    await screen.findByText(/The request was not accepted/); expect(screen.queryByText(/private-key|provider body/)).toBeNull();
    expect(fake.bridge.sendMessage).toHaveBeenCalledOnce();
    await waitFor(() => expect((screen.getByRole('button', { name: 'New conversation' }) as HTMLButtonElement).disabled).toBe(false));
  });
  it.each([
    ['meta_authentication_failed', /Meta did not accept the credential\. Check the key through the native Windows settings/],
    ['meta_rate_limited', /Meta rate-limited the request\. No automatic retry occurred/],
    ['data_admission_denied', /The message or prior conversation failed Contributor data admission/],
    ['contributor_terms_required', /Review the current Contributor terms and account eligibility/],
    ['budget_attention_required', /The spending policy needs attention\. Check Settings/],
    ['private_store_capacity', /The protected local store has reached its capacity/],
    ['model_denied', /This model is not supported\. Only Muse Spark/],
    ['reasoning_denied', /Choose Minimal, Low, Medium, High or Extra high reasoning/],
    ['clock_unavailable', /The host cannot validate the current time/],
  ] as const)('displays an actionable local English message for finite %s errors', async (code, message) => {
    const fake = fakeBridge(); vi.mocked(fake.bridge.sendMessage).mockRejectedValue(code);
    render(<App bridge={fake.bridge}/>); await ready(); await prepare(); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    await screen.findByText(message); expect(screen.queryByText(code, { exact: true })).toBeNull(); expect(fake.bridge.sendMessage).toHaveBeenCalledOnce();
  });
  it('explains a protected-store startup failure without exposing raw backend output or resetting state', async () => {
    const fake = fakeBridge(); vi.mocked(fake.bridge.getStatus).mockRejectedValue('private_store_corrupt');
    render(<App bridge={fake.bridge}/>); await screen.findByText(/The protected local store failed its integrity check/);
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(screen.queryByText('private_store_corrupt')).toBeNull();
  });
  it('shows financial attention even when the visible answer completed', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready(); await prepare(); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    const input = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    act(() => fake.emit({ conversationId: input.conversationId, requestId: input.requestId, type: 'content', text: 'Completed visible answer' }));
    act(() => fake.emit({ conversationId: input.conversationId, requestId: input.requestId, type: 'terminal', status: 'completed', errorCode: 'budget_attention_required' }));
    await screen.findByText('Completed visible answer'); await screen.findByText(/The spending policy needs attention/);
    expect(screen.getByText('Response completed')).toBeTruthy();
  });
  it('uses the UTF-8 limit in the actual composer without silently truncating', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready(); await prepare('é'.repeat(16384));
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: 'é'.repeat(16385) } }); expect(area.value).toHaveLength(16385);
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true); expect(screen.getByText(/32,770.*message too long/)).toBeTruthy();
  });
  it('implements local menus, traps modal focus, restores it and blocks behind-dialogue shortcuts', async () => {
    const fake = fakeBridge(); const user = userEvent.setup(); render(<App bridge={fake.bridge}/>); await ready();
    await user.type(screen.getByRole('textbox', { name: 'Message for Aster' }), 'Draft only'); await user.click(screen.getByRole('button', { name: 'Edit' }));
    await user.click(screen.getByRole('menuitem', { name: 'Select All' })); await user.click(screen.getByRole('button', { name: 'Edit' }));
    await user.click(screen.getByRole('menuitem', { name: 'Delete' })); expect((screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement).value).toBe('');
    const opener = screen.getByRole('button', { name: 'Settings' }); await user.click(opener);
    const close = screen.getByRole('button', { name: 'Close dialogue' }); expect(document.activeElement).toBe(close);
    await user.tab({ shift: true }); expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Remove key…' }));
    await user.tab(); expect(document.activeElement).toBe(close); await user.keyboard('{Control>}n{/Control}'); expect(fake.bridge.createConversation).not.toHaveBeenCalled();
    await user.keyboard('{Escape}'); expect(document.activeElement).toBe(opener);
    await user.click(screen.getByRole('button', { name: 'Help' })); await user.click(screen.getByRole('menuitem', { name: 'Documentation' }));
    expect(screen.getByRole('dialog', { name: 'Documentation' })).toBeTruthy(); expect(screen.getByText(/encrypted recovery records remain/)).toBeTruthy();
    expect(screen.getByText(/combined native PDF-original and normalised-image bytes sent inline must fit 16 MiB before base64 encoding; the complete serialised reviewed request must fit 24 MiB/)).toBeTruthy();
    expect(screen.getByText(/16 MiB cumulative processed content per conversation, including inert text and PNG pages/)).toBeTruthy();
  });
  it('removes only the selected history record through the native-confirmed bridge', async () => {
    const fake = fakeBridge(baseline, [saved('first', 'Remove me', 2), saved('second', 'Keep me', 1)]); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for first');
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Local data' })); await userEvent.click(screen.getByRole('button', { name: 'Remove this conversation from history…' }));
    await screen.findByText('Reply for second'); expect(fake.bridge.deleteConversation).toHaveBeenCalledWith({ conversationId: 'first' });
    expect(fake.bridge.removeCredential).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(fake.bridge.reconcileBudget).not.toHaveBeenCalled();
  });
  it('proposes explicit financial integers, preserves account identity and never accepts a key in the renderer', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready(); await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(document.querySelector('input[type="password"]')).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Usage' }));
    for (const [label, value] of [['GBP per USD (integer parts per million)','765432'],['Safety margin (basis points)','100'],['Tax (basis points)','0'],['Payment charges (basis points)','0']]) fireEvent.change(screen.getByLabelText(label), { target: { value } });
    fireEvent.change(screen.getByLabelText(/Valid until/), { target: { value: '2026-10-20T10:00' } }); await userEvent.click(screen.getByRole('button', { name: 'Review financial configuration in Windows' }));
    expect(fake.bridge.configureBudget).toHaveBeenCalledOnce(); const input = vi.mocked(fake.bridge.configureBudget).mock.calls[0][0];
    expect(input).toMatchObject({ accountId: 'a'.repeat(32), gbpPerUsdPpm: 765432, marginBps: 100, taxBps: 0, paymentBps: 0 }); expect(Number.isInteger(input.validUntilUnix)).toBe(true);
    expect(JSON.stringify(input)).not.toMatch(/spent|reserved|reset/);
  });
  it('proposes statement-confirmed pending charges with exact decimal arithmetic, not a reset', async () => {
    const status = { ...baseline, availability: 'attention_required' as const, budget: { ...baseline.budget, requiresReconciliation: true, pending: [{ requestId: 'pending_request', period: '2026-10', reservationNanoGbp: '100000000', dispatched: true }] } };
    const fake = fakeBridge(status); render(<App bridge={fake.bridge}/>); await screen.findByText('Attention required. Check Settings; sending is unavailable.');
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Usage' })); fireEvent.change(screen.getByLabelText('Statement-confirmed charge in GBP for pending_request'), { target: { value: '0.012345678' } });
    await userEvent.click(screen.getByRole('button', { name: 'Review this charge in Windows' })); expect(fake.bridge.reconcileBudget).toHaveBeenCalledWith({ requestId: 'pending_request', confirmedChargeNanoGbp: '12345678' });
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(fake.bridge.deleteConversation).not.toHaveBeenCalled();
  });
});
