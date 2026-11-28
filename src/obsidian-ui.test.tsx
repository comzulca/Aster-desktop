import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { fakeBridge } from './test-fixtures';
import { VaultSettings } from './VaultSettings';
import { component, validNotes } from './obsidian-ui';

afterEach(cleanup);
function bridge() {
  return {
    getObsidianStatus: vi.fn(async () => ({ selected: false, label: '' })),
    selectObsidianVault: vi.fn(async () => ({ selected: true, label: 'Vault' })),
    clearObsidianVault: vi.fn(async () => ({ selected: false, label: '' })),
    listObsidianNotes: vi.fn(async () => ({ notes: [{ relativePath: 'Python/Atlas.md' }], limited: false })),
    readObsidianNote: vi.fn(async () => null),
    createObsidianNote: vi.fn(async () => ({ relativePath: 'Python/New.md' })),
    createObsidianTopic: vi.fn(async () => ({ relativePath: 'Python' })),
  };
}
const button = (name: string) => screen.getByRole('button', { name }) as HTMLButtonElement;
async function openVault() {
  const host = bridge();
  render(<VaultSettings bridge={host} locked={false}/>);
  await screen.findByText('No vault is connected.');
  fireEvent.click(button('Choose vault in Windows'));
  await screen.findByText('Selected vault: Vault');
  fireEvent.change(screen.getByLabelText('Topic folder'), { target: { value: 'Python' } });
  return host;
}
describe('Aster_obsidian manual vault', () => {
  it('waits for the initial status before allowing selection and recovers after a failed check', async () => {
    const host = bridge();
    let rejectStatus!: (error: Error) => void;
    host.getObsidianStatus.mockImplementationOnce(() => new Promise((_, reject) => { rejectStatus = reject; }));
    render(<VaultSettings bridge={host} locked={false}/>);
    expect(button('Choose vault in Windows').disabled).toBe(true);
    fireEvent.click(button('Choose vault in Windows'));
    expect(host.selectObsidianVault).not.toHaveBeenCalled();
    await act(async () => rejectStatus(new Error('obsidian_io_error')));
    fireEvent.click(button('Choose vault in Windows'));
    await screen.findByText('Selected vault: Vault');
  });

  it('ignores a late selection receipt from a previous host session', async () => {
    const first = bridge();
    const second = bridge();
    let finishSelection!: (value: { selected: boolean; label: string }) => void;
    first.selectObsidianVault.mockImplementationOnce(() => new Promise(resolve => { finishSelection = resolve; }));
    const view = render(<VaultSettings bridge={first} locked={false}/>);
    await screen.findByText('No vault is connected.');
    fireEvent.click(button('Choose vault in Windows'));
    view.rerender(<VaultSettings bridge={second} locked={false}/>);
    await screen.findByText('No vault is connected.');
    fireEvent.click(button('Choose vault in Windows'));
    await screen.findByText('Selected vault: Vault');
    await act(async () => finishSelection({ selected: true, label: 'Previous vault' }));
    expect(screen.queryByText('Selected vault: Previous vault')).toBeNull();
    expect(screen.getByText('Selected vault: Vault')).toBeTruthy();
  });

  it('qualifies an empty result when the host stopped before checking the whole topic', async () => {
    const host = await openVault();
    host.listObsidianNotes.mockResolvedValueOnce({ notes: [], limited: true });
    fireEvent.click(button('Find local notes'));
    await screen.findByText('No matches in the entries checked.');
    expect(screen.queryByText('No matching notes in this topic.')).toBeNull();
  });

  it('does not open the vault or search on entry; reads only a chosen relative note locally', async () => {
    const host = await openVault();
    expect(host.listObsidianNotes).not.toHaveBeenCalled();
    expect(host.readObsidianNote).not.toHaveBeenCalled();
    fireEvent.click(button('Find local notes'));
    await screen.findByRole('button', { name: 'View Python/Atlas.md in Windows' });
    expect(host.listObsidianNotes).toHaveBeenCalledWith({ topic: 'Python', query: '' });
    expect(host.readObsidianNote).not.toHaveBeenCalled();
    fireEvent.click(button('View Python/Atlas.md in Windows'));
    await screen.findByText('Closed the local note viewer.');
    expect(host.readObsidianNote).toHaveBeenCalledWith({ relativePath: 'Python/Atlas.md' });
  });
  it('reports cancellation without claiming a note was saved, then confirms the exact native receipt', async () => {
    const host = await openVault();
    host.createObsidianNote.mockRejectedValueOnce('cancelled');
    fireEvent.change(screen.getByLabelText('New note title (without .md)'), { target: { value: 'New' } });
    fireEvent.change(screen.getByLabelText('New note contents'), { target: { value: 'Owner-approved decision' } });
    fireEvent.click(button('Review new note in Windows'));
    await screen.findByText('No change was made.');
    expect(screen.queryByText(/^Saved locally:/)).toBeNull();
    fireEvent.click(button('Review new note in Windows'));
    await screen.findByText('Saved locally: Python/New.md');
    expect(host.createObsidianNote).toHaveBeenCalledWith({ topic: 'Python', title: 'New', content: 'Owner-approved decision' });
  });
  it('does not consult a vault for chat drafts and keeps note creation out of the conversation', async () => {
    const host = bridge();
    const fake = fakeBridge();
    const appBridge = { ...fake.bridge, ...host };
    render(<App bridge={appBridge}/>);
    const mainDraft = screen.getByRole('textbox', { name: 'Message for Aster' });
    const chat = 'How do Python virtual environments work?';
    fireEvent.change(mainDraft, { target: { value: chat } });
    expect(host.getObsidianStatus).not.toHaveBeenCalled();
    expect(host.listObsidianNotes).not.toHaveBeenCalled();
    fireEvent.click(button('Settings'));
    fireEvent.click(button('Aster_obsidian'));
    await screen.findByText('No vault is connected.');
    fireEvent.click(button('Choose vault in Windows'));
    await screen.findByText('Selected vault: Vault');
    const input = screen.getByLabelText('New note contents');
    fireEvent.change(input, { target: { value: 'SYNTHETIC_LOCAL_CANARY' } });
    expect(host.createObsidianNote).not.toHaveBeenCalled();
    fireEvent.click(button('Close dialogue'));
    expect((screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement).value).toBe(chat);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(host.listObsidianNotes).not.toHaveBeenCalled();
  });
  it('rejects receipt paths outside the requested topic and nonportable names', () => {
    for (const invalid of ['..', 'a/b', 'x\\y', 'COM1.txt', 'Access_Token', 'NUL ', 'hidden.', '.obsidian', 'x:y']) expect(component(invalid)).toBe(false);
    expect(component('Notas de Atlas')).toBe(true);
    expect(validNotes({ notes: [{ relativePath: 'Rust/Atlas.md' }], limited: false }, 'Python')).toBe(false);
    expect(validNotes({ notes: [{ relativePath: 'Python/../Atlas.md' }], limited: false }, 'Python')).toBe(false);
    expect(validNotes({ notes: [{ relativePath: 'Python/Atlas.md', content: 'extra authority' }], limited: false }, 'Python')).toBe(false);
  });
});
