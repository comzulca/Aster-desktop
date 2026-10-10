import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, saved, selectClassification } from './test-fixtures';
import type { CustomizationSnapshot, GitHubStatus } from './types';
import { emptyProfile } from './personalization-ui';
// Vitest deliberately stubs CSS imports in this headless configuration.
// Inspect authored source, not jsdom dimensions or an installed-paint claim.
const { readFileSync } = await vi.importActual<{ readFileSync(path: string, encoding: 'utf8'): string }>('node:fs');
const styles = readFileSync('src/styles.css', 'utf8');
const appSource = readFileSync('src/App.tsx', 'utf8');

afterEach(() => cleanup());
function fixture() {
  const fake = fakeBridge({ ...baseline, capabilities: { ...baseline.capabilities, attachmentImport: true } }, [saved('main_chat', 'Main conversation preserved')]);
  let catalogue: CustomizationSnapshot = { revision: 0, entries: [
    { id: 'skill:aster-builder', kind: 'skill', name: 'Aster Builder', description: 'Owner-supplied inert instructions', enabled: false, readOnly: true, availability: 'instructions-only', entryRevision: 0 },
    { id: 'skill:synthetic-fixture', kind: 'skill', name: 'Synthetic fixture skill', description: 'Imported inert test entry', enabled: false, readOnly: false, availability: 'instructions-only', entryRevision: 0 },
    { id: 'builtin:github', kind: 'plugin', name: 'GitHub', description: 'Reviewed built-in connector', enabled: false, readOnly: true, availability: 'built-in-connector', entryRevision: 0 },
  ] };
  fake.bridge.getCustomization = vi.fn(async () => structuredClone(catalogue));
  fake.bridge.getCustomizationPackage = vi.fn(async ({ id }) => ({ id, kind: 'skill' as const, slug: 'aster-builder', name: 'Aster Builder', description: 'Owner-supplied inert instructions', instructions: 'Complete inert fixture instructions.', enabled: false, readOnly: true, entryRevision: 0 }));
  fake.bridge.saveCustomization = vi.fn(); fake.bridge.importCustomization = vi.fn(async () => structuredClone(catalogue));
  fake.bridge.setCustomizationEnabled = vi.fn(async input => { catalogue = { revision: catalogue.revision + 1, entries: catalogue.entries.map(entry => entry.id === input.id ? { ...entry, enabled: input.enabled } : entry) }; return structuredClone(catalogue); });
  fake.bridge.selectAttachments = vi.fn(async () => [{ id: 'a'.repeat(32), kind: 'text' as const, label: 'Synthetic retained.txt', sourceBytes: 80, processedBytes: 80, pageCount: 0 }]);
  fake.bridge.removeAttachment = vi.fn(async () => ({ removed: true }));
  return fake;
}
async function openWorkspace() {
  await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
  await userEvent.click(screen.getByRole('button', { name: 'Customize' }));
  expect(screen.queryByRole('tab', { name: 'Skills' })).toBeNull();
  await userEvent.click(screen.getByRole('button', { name: 'Open Customize workspace' }));
  await screen.findByText('Aster Builder');
}
describe('owner-reference local workspace conformance (T-084/T-088/T-089)', () => {
  it('uses coordinate-aware source proportions and responsive two-column cards without claiming rendered dimensions', () => {
    expect(styles).toContain('--reference-rail:clamp(240px,20.9vw,440px)');
    expect(styles).toContain('--reference-composer:clamp(600px,63vw,1320px)');
    expect(styles).toContain('--reference-orb:clamp(110px,8.7vw,184px)');
    expect(styles).toContain('.customization-catalogue ul{display:grid;grid-template-columns:repeat(2,minmax(0,1fr))');
    expect(styles).toContain('@media(max-width:1200px)'); expect(styles).toContain('@media(max-width:530px)');
    expect(styles).toContain('.customization-page{grid-template-columns:1fr;grid-template-rows:auto minmax(0,1fr)}');
  });
  it('excludes profile fields from global capture before any value or selection text is read', () => {
    const capture = appSource.slice(appSource.indexOf('  function captureEdit('), appSource.indexOf('  function captureVisibleSelection('));
    const profileGuard = capture.indexOf("element.closest('.personalization-page')");
    expect(profileGuard).toBeGreaterThanOrEqual(0); expect(profileGuard).toBeLessThan(capture.indexOf('text: element.value'));
    expect(capture.slice(profileGuard, capture.indexOf('text: element.value'))).toContain("editTarget.current = null; copiedSelection.current = ''; return;");
    const selection = appSource.slice(appSource.indexOf('  function captureVisibleSelection('), appSource.indexOf('  function editable('));
    const selectionGuard = selection.indexOf("anchorElement?.closest('.personalization-page')");
    expect(selectionGuard).toBeGreaterThanOrEqual(0); expect(selectionGuard).toBeLessThan(selection.indexOf('selection?.toString()'));
    expect(selection.slice(selectionGuard, selection.indexOf('selection?.toString()'))).toContain("editTarget.current = null; copiedSelection.current = ''; return;");
  });
  it('opens a separate full-page workspace and preserves the chat, draft, classification and exact temporary attachment IDs', async () => {
    const fake = fixture(); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'SYNTHETIC_MAIN_DRAFT' } });
    await userEvent.click(screen.getByRole('button', { name: 'Add attachments' })); await screen.findByText('Synthetic retained.txt'); await selectClassification('synthetic');
    await openWorkspace();
    const workspace = screen.getByRole('main', { name: 'Customize workspace' });
    expect(workspace.className).toBe('customize-workspace'); expect(screen.queryByRole('dialog')).toBeNull();
    expect(workspace.querySelector('.customization-subrail h2')?.textContent).toBe('Customize');
    expect(screen.getByRole('tablist', { name: 'Customization categories' }).getAttribute('aria-orientation')).toBe('vertical');
    expect(within(screen.getByRole('list', { name: 'Local skills' })).getAllByRole('listitem')).toHaveLength(2);
    expect(document.getElementById('main')?.hidden).toBe(true); expect(document.getElementById('main')?.hasAttribute('inert')).toBe(true);
    expect(screen.queryByRole('textbox', { name: 'Message for Aster' })).toBeNull();
    expect(fake.bridge.loadConversation).toHaveBeenCalledExactlyOnceWith({ conversationId: 'main_chat' });
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.removeAttachment).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Return to chat' }));
    expect(screen.getByDisplayValue('SYNTHETIC_MAIN_DRAFT')).toBeTruthy(); expect(screen.getByText('Synthetic retained.txt')).toBeTruthy(); expect(screen.getByText('Reply for main_chat')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(false);
    await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    expect(fake.bridge.sendMessage).toHaveBeenCalledWith(expect.objectContaining({ conversationId: 'main_chat', message: 'SYNTHETIC_MAIN_DRAFT', dataClass: 'synthetic', attachmentIds: ['a'.repeat(32)] }));
  });
  it('denies hidden chat callbacks and shortcuts, while search/category navigation shows only real entries', async () => {
    const fake = fixture(); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat');
    const mainInput = screen.getByRole('textbox', { name: 'Message for Aster' });
    fireEvent.change(mainInput, { target: { value: 'SYNTHETIC_HIDDEN_DRAFT' } }); await selectClassification('synthetic');
    await openWorkspace();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Return to chat' })));
    fireEvent.keyDown(mainInput, { key: 'Enter' });
    fireEvent.click(document.querySelector('.sidebar-tools button')!);
    await userEvent.keyboard('{Control>}n{/Control}{Control>}k{/Control}{Control>}f{/Control}{Control>}{PageDown}{/Control}');
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.selectAttachments).not.toHaveBeenCalled(); expect(fake.bridge.loadConversation).toHaveBeenCalledOnce();
    fireEvent.change(screen.getByRole('searchbox', { name: 'Search skills' }), { target: { value: 'synthetic fixture' } });
    expect(screen.queryByText('Aster Builder')).toBeNull(); expect(screen.getByText('Synthetic fixture skill')).toBeTruthy();
    screen.getByRole('tab', { name: 'Skills' }).focus(); await userEvent.keyboard('{ArrowUp}');
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('tab', { name: 'Plugins' })));
    expect(screen.getByRole('tab', { name: 'Plugins' }).getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('searchbox', { name: 'Search plugins' }).getAttribute('value')).toBe('');
    expect(screen.getByText('GitHub')).toBeTruthy();
    for (const label of ['Gmail','Cloud Environment','Image Gen','Public','System','Install']) expect(screen.queryByText(label, { exact: true })).toBeNull();
    expect((screen.getByRole('button', { name: 'Manage account' }) as HTMLButtonElement).disabled).toBe(true);
  });
  it('requires explicit cancellation of an unsaved editor before workspace navigation or hidden chat actions', async () => {
    const fake = fixture(); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat'); await openWorkspace();
    await userEvent.click(screen.getByRole('button', { name: 'Add skill' }));
    fireEvent.change(screen.getByLabelText('Name'), { target: { value: 'Unsaved synthetic draft' } });
    expect((screen.getByRole('button', { name: 'Return to chat' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('button', { name: 'Open workspace Settings' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('tab', { name: 'Plugins' }) as HTMLButtonElement).disabled).toBe(true);
    await userEvent.keyboard('{Control>}n{/Control}{Control>},{/Control}');
    expect(screen.getByDisplayValue('Unsaved synthetic draft')).toBeTruthy(); expect(screen.queryByRole('dialog')).toBeNull(); expect(fake.bridge.saveCustomization).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Cancel editor' }));
    await userEvent.click(screen.getByRole('button', { name: 'Return to chat' })); await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('textbox', { name: 'Message for Aster' })));
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('denies the new transition for unsaved or in-flight local Personalization without passing any profile content to Customize', async () => {
    const fake = fixture(); let profile = { revision: 0, profile: emptyProfile() }; let finish: (() => void) | undefined;
    fake.bridge.getPersonalization = vi.fn(async () => structuredClone(profile));
    fake.bridge.clearPersonalization = vi.fn();
    fake.bridge.savePersonalization = vi.fn(async input => { await new Promise<void>(resolve => { finish = resolve; }); profile = { revision: profile.revision + 1, profile: structuredClone(input.profile) }; return structuredClone(profile); });
    render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat');
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Personalization' }));
    await waitFor(() => expect((screen.getByLabelText('Nickname') as HTMLInputElement).disabled).toBe(false));
    fireEvent.change(screen.getByLabelText('Nickname'), { target: { value: 'PRIVATE_UNSAVED_PROFILE_CANARY' } });
    await userEvent.click(screen.getByRole('button', { name: 'Customize' }));
    expect((screen.getByRole('button', { name: 'Open Customize workspace' }) as HTMLButtonElement).disabled).toBe(true);
    expect(fake.bridge.getCustomization).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Personalization' })); expect(screen.getByDisplayValue('PRIVATE_UNSAVED_PROFILE_CANARY')).toBeTruthy();
    await userEvent.click(screen.getByRole('button', { name: 'Save Personalization' }));
    await userEvent.click(screen.getByRole('button', { name: 'Customize' }));
    expect((screen.getByRole('button', { name: 'Open Customize workspace' }) as HTMLButtonElement).disabled).toBe(true); expect(fake.bridge.getCustomization).not.toHaveBeenCalled();
    finish!(); await waitFor(() => expect((screen.getByRole('button', { name: 'Open Customize workspace' }) as HTMLButtonElement).disabled).toBe(false));
    await userEvent.click(screen.getByRole('button', { name: 'Open Customize workspace' })); await screen.findByText('Aster Builder');
    expect(screen.queryByDisplayValue('PRIVATE_UNSAVED_PROFILE_CANARY')).toBeNull(); expect(screen.queryByText('PRIVATE_UNSAVED_PROFILE_CANARY')).toBeNull();
    expect(JSON.stringify(vi.mocked(fake.bridge.getCustomization!).mock.calls)).not.toContain('PRIVATE_UNSAVED_PROFILE_CANARY');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.saveCustomization).not.toHaveBeenCalled();
  });
  it('keeps ordinary profile editing local and denies a forged profile edit-key from poisoning global undo or clipboard buffers', async () => {
    const fake = fixture(); const profile = { revision: 0, profile: emptyProfile() };
    fake.bridge.getPersonalization = vi.fn(async () => structuredClone(profile)); fake.bridge.savePersonalization = vi.fn(); fake.bridge.clearPersonalization = vi.fn();
    const clipboard = { readText: vi.fn(async () => ''), writeText: vi.fn(async () => {}) };
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: clipboard });
    render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'SYNTHETIC_MAIN_DRAFT_NOT_PROFILE' } });
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Personalization' }));
    const nickname = screen.getByLabelText('Nickname') as HTMLInputElement; await waitFor(() => expect(nickname.disabled).toBe(false));
    // A hostile local edit-key attribute must not cross the component boundary.
    nickname.dataset.editKey = 'search';
    nickname.focus(); fireEvent.change(nickname, { target: { value: 'PRIVATE_GLOBAL_CAPTURE_CANARY' } }); nickname.setSelectionRange(0, nickname.value.length); fireEvent.select(nickname);
    const ordinarySelect = new KeyboardEvent('keydown', { key: 'a', ctrlKey: true, bubbles: true, cancelable: true }); nickname.dispatchEvent(ordinarySelect);
    expect(ordinarySelect.defaultPrevented).toBe(false); expect(nickname.value).toBe('PRIVATE_GLOBAL_CAPTURE_CANARY');
    await userEvent.click(screen.getByRole('button', { name: 'Close dialogue' }));
    await userEvent.click(screen.getByRole('button', { name: 'Search conversations' }));
    const search = await screen.findByRole('searchbox', { name: 'Search saved conversations' }); search.focus();
    await userEvent.click(screen.getByRole('button', { name: 'Edit' }));
    expect((screen.getByRole('menuitem', { name: 'Undo' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('menuitem', { name: 'Copy' }) as HTMLButtonElement).disabled).toBe(true);
    expect((search as HTMLInputElement).value).toBe(''); expect(screen.getByDisplayValue('SYNTHETIC_MAIN_DRAFT_NOT_PROFILE')).toBeTruthy();
    expect(screen.queryByDisplayValue('PRIVATE_GLOBAL_CAPTURE_CANARY')).toBeNull(); expect(clipboard.writeText).not.toHaveBeenCalled(); expect(clipboard.readText).not.toHaveBeenCalled();
    expect(fake.bridge.savePersonalization).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.getCustomization).not.toHaveBeenCalled();
  });
  it('keeps an owned account popup modal, cancels its pending operation and ignores a late callback after returning', async () => {
    const fake = fixture(); let finish: (() => void) | undefined;
    const status: GitHubStatus = { enabled: true, state: 'disconnected', account: null, repository: null, userCode: null, verificationUri: 'https://github.com/login/device', nextPollSeconds: 0, inFlight: false, mutationsAvailable: false };
    fake.bridge.getGitHubStatus = vi.fn(async () => structuredClone(status));
    fake.bridge.beginGitHubConnection = vi.fn(async () => { await new Promise<void>(resolve => { finish = resolve; }); });
    fake.bridge.pollGitHubConnection = vi.fn(async () => {}); fake.bridge.cancelGitHubConnection = vi.fn(async () => {}); fake.bridge.disconnectGitHub = vi.fn(async () => {});
    fake.bridge.listGitHubRepositories = vi.fn(async () => []); fake.bridge.selectGitHubRepository = vi.fn(async () => {}); fake.bridge.readGitHubFile = vi.fn();
    render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for main_chat'); await openWorkspace();
    await userEvent.click(screen.getByRole('tab', { name: 'Plugins' })); await userEvent.click(screen.getByRole('switch', { name: 'Enable GitHub' }));
    await screen.findByText(/catalogue preference was enabled/); await userEvent.click(screen.getByRole('button', { name: 'Manage account' })); await screen.findByText(/Connection: disconnected/);
    expect((screen.getByRole('button', { name: 'Return to chat' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('switch', { name: 'Enable GitHub' }) as HTMLInputElement).disabled).toBe(true);
    await userEvent.click(screen.getByRole('button', { name: 'Connect or change account in Windows' }));
    await userEvent.click(screen.getByRole('button', { name: 'Close account popup' })); await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(fake.bridge.cancelGitHubConnection).toHaveBeenCalledOnce(); await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Manage account' })));
    await userEvent.click(screen.getByRole('button', { name: 'Return to chat' }));
    await act(async () => { finish!(); }); expect(fake.bridge.pollGitHubConnection).not.toHaveBeenCalled(); expect(screen.queryByRole('dialog')).toBeNull();
    expect(screen.getByText('Reply for main_chat')).toBeTruthy(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.createConversation).not.toHaveBeenCalled();
  });
});
