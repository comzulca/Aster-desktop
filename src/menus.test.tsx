import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { fakeBridge, saved } from './test-fixtures';
afterEach(cleanup);
async function ready() { await screen.findByText('Ready for separately reviewed text requests.'); }
async function menu(name: string, item: string) { await userEvent.click(screen.getByRole('button', { name })); await userEvent.click(screen.getByRole('menuitem', { name: item })); }
describe('v0.2.1 menus/caption/quit integration (T-064/T-066/T-067/T-068/T-070)', () => {
  it('closes a mouse-opened top menu with Escape and preserves trigger focus', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const trigger = screen.getByRole('button', { name: 'Help' }); await userEvent.click(trigger);
    expect(screen.getByRole('menu', { name: 'Help' })).toBeTruthy(); expect(document.activeElement).toBe(trigger);
    await userEvent.keyboard('{Escape}'); expect(screen.queryByRole('menu', { name: 'Help' })).toBeNull(); expect(document.activeElement).toBe(trigger);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.windowAction).not.toHaveBeenCalled();
  });
  it('keeps zoom viewport percentages unmodified instead of compensating them twice', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    for (let i = 0; i < 5; i++) fireEvent.keyDown(window, { key: '+', ctrlKey: true, shiftKey: true });
    const shell = document.querySelector('.app-shell') as HTMLElement;
    expect(shell.style.zoom).toBe('1.5'); expect(shell.style.width).toBe('100%'); expect(shell.style.height).toBe('100%');
    for (let i = 0; i < 7; i++) fireEvent.keyDown(window, { key: '-', ctrlKey: true });
    expect(shell.style.zoom).toBe('0.8'); expect(shell.style.width).toBe('100%'); expect(shell.style.height).toBe('100%');
  });
  it('preserves unsent drafts across saved chat navigation and clears content classification', async () => {
    const fake = fakeBridge(undefined, [saved('first', 'First', 2), saved('second', 'Second', 1)]);
    render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for first');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'First unsent' } });
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Content classification' }), 'synthetic');
    await menu('View', 'Next Chat'); await screen.findByText('Reply for second');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Second unsent' } });
    await menu('View', 'Previous Chat'); await screen.findByText('Reply for first');
    expect((screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement).value).toBe('First unsent');
    expect((screen.getByRole('combobox', { name: 'Content classification' }) as HTMLSelectElement).value).toBe('');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('uses default-cancel quit review for drafts and routes native close to it', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Keep this draft' } });
    act(() => fake.close());
    const cancel = await screen.findByRole('button', { name: 'Keep working' }); expect(document.activeElement).toBe(cancel);
    await userEvent.keyboard('{Escape}'); expect(fake.bridge.windowAction).not.toHaveBeenCalled();
    await menu('File', 'Quit Aster'); await userEvent.click(screen.getByRole('button', { name: 'Discard drafts and quit' }));
    await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenCalledWith({ action: 'close' }));
    expect(fake.bridge.deleteConversation).not.toHaveBeenCalled(); expect(fake.bridge.removeCredential).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
  });
  it('edits the preserved composer selection with undo/redo and rejects failed paste honestly', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: 'Alpha Beta' } }); area.focus(); area.setSelectionRange(6, 10); fireEvent.select(area);
    await menu('Edit', 'Delete'); expect(area.value).toBe('Alpha ');
    await menu('Edit', 'Undo'); expect(area.value).toBe('Alpha Beta');
    await menu('Edit', 'Redo'); expect(area.value).toBe('Alpha ');
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { readText: vi.fn(async () => { throw new Error('private clipboard payload'); }) } });
    await menu('Edit', 'Paste'); await screen.findByText(/Clipboard access was refused/);
    expect(area.value).toBe('Alpha '); expect(screen.queryByText(/private clipboard payload/)).toBeNull(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('finds only visible text, bounds zoom, restores sidebar and shows honest offline help/status', async () => {
    const fake = fakeBridge(undefined, [saved('first', 'First')]); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for first');
    await menu('View', 'Find…'); fireEvent.change(screen.getByRole('searchbox', { name: 'Find in this conversation' }), { target: { value: 'reply' } });
    expect(screen.getByText('1 match')).toBeTruthy(); expect(document.querySelector('mark')?.textContent).toBe('Reply');
    await userEvent.keyboard('{Control>}{Shift>}s{/Shift}{/Control}'); expect(screen.getByRole('complementary', { hidden: true }).getAttribute('aria-hidden')).toBe('true');
    await userEvent.keyboard('{Control>}{Shift>}s{/Shift}{/Control}'); expect(screen.getByRole('complementary').getAttribute('aria-hidden')).toBe('false');
    await userEvent.keyboard('{Control>}+{/Control}'); expect(screen.getByLabelText('Interface zoom').textContent).toBe('110%');
    await menu('Help', 'System Status'); expect(screen.getByRole('dialog', { name: 'System Status' })).toBeTruthy(); expect(screen.getByText(/Live connection: unverified/)).toBeTruthy();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled();
  });
  it('uses guarded caption actions and refuses unavailable or failed native window operations', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    expect(screen.getByRole('img', { name: 'English (United Kingdom)' }).closest('.captionbar')).toBeTruthy();
    await userEvent.click(screen.getByRole('button', { name: 'Minimise window' }));
    await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenCalledWith({ action: 'minimise' }));
    await userEvent.click(screen.getByRole('button', { name: 'Maximise window' })); await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenCalledWith({ action: 'toggle_maximise' }));
    await menu('View', 'Toggle Full Screen'); await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenCalledWith({ action: 'toggle_fullscreen' }));
    fireEvent.mouseDown(document.querySelector('.caption-drag-region')!, { button: 0, detail: 1 }); await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenCalledWith({ action: 'start_dragging' }));
    vi.mocked(fake.bridge.windowAction!).mockRejectedValue(new Error('private host output'));
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Retain on failure' } });
    await menu('File', 'Quit Aster'); await userEvent.click(screen.getByRole('button', { name: 'Discard drafts and quit' }));
    await screen.findByText(/The window action could not be confirmed/);
    expect((screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement).value).toBe('Retain on failure'); expect(screen.queryByText('private host output')).toBeNull();
    cleanup(); delete fake.bridge.windowAction; delete fake.bridge.subscribeClose; render(<App bridge={fake.bridge}/>); await ready();
    expect((screen.getByRole('button', { name: 'Minimise window' }) as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(screen.getByRole('button', { name: 'File' })); expect((screen.getByRole('menuitem', { name: 'Quit Aster' }) as HTMLButtonElement).disabled).toBe(true);
  });
  it('keeps clipboard reads deliberate, preserves the range and declines stale asynchronous paste', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    const clipboard = { readText: vi.fn(async () => 'Gamma'), writeText: vi.fn(async (_text: string) => {}) };
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: clipboard });
    expect(clipboard.readText).not.toHaveBeenCalled();
    fireEvent.change(area, { target: { value: 'Alpha Beta' } }); area.focus(); area.setSelectionRange(6, 10); fireEvent.select(area);
    await menu('Edit', 'Paste'); await waitFor(() => expect(area.value).toBe('Alpha Gamma'));
    expect(clipboard.readText).toHaveBeenCalledOnce(); expect(area.selectionStart).toBe(11);
    area.focus(); area.setSelectionRange(6, 11); fireEvent.select(area); await userEvent.click(screen.getByRole('button', { name: 'Edit' }));
    expect((screen.getByRole('menuitem', { name: 'Copy' }) as HTMLButtonElement).disabled).toBe(false);
    await userEvent.click(screen.getByRole('menuitem', { name: 'Copy' })); expect(clipboard.writeText).toHaveBeenLastCalledWith('Gamma'); expect(area.value).toBe('Alpha Gamma');
    let finish: ((text: string) => void) | undefined; clipboard.readText.mockImplementation(() => new Promise(resolve => { finish = resolve; }));
    await menu('Edit', 'Paste'); fireEvent.change(area, { target: { value: 'Newer unsent draft' } });
    await act(async () => finish!('Stale replacement')); expect(area.value).toBe('Newer unsent draft');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled();
  });
  it('blocks every local action and keyboard shortcut during native review, without clipboard access', async () => {
    const fake = fakeBridge(); let finish: (() => void) | undefined;
    vi.mocked(fake.bridge.configureCredential).mockImplementation(() => new Promise(resolve => { finish = () => resolve({ configured: true }); }));
    const clipboard = { readText: vi.fn(async () => 'Unexpected'), writeText: vi.fn(async (_text: string) => {}) };
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: clipboard });
    render(<App bridge={fake.bridge}/>); await ready(); await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await userEvent.click(screen.getByRole('button', { name: 'Change key in Windows' }));
    await userEvent.keyboard('{Control>}nqv{/Control}{F11}');
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.windowAction).not.toHaveBeenCalled(); expect(clipboard.readText).not.toHaveBeenCalled();
    await act(async () => finish!());
  });
  it('copies deliberately selected rendered text rather than an older composer range', async () => {
    const fake = fakeBridge(undefined, [saved('first', 'First')]); render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for first');
    const clipboard = { readText: vi.fn(async () => ''), writeText: vi.fn(async (_text: string) => {}) }; Object.defineProperty(navigator, 'clipboard', { configurable: true, value: clipboard });
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement; fireEvent.change(area, { target: { value: 'Unrelated draft' } }); area.focus(); area.select(); fireEvent.select(area);
    const reply = screen.getByText('Reply for first'); const range = document.createRange(); range.selectNodeContents(reply); window.getSelection()!.removeAllRanges(); window.getSelection()!.addRange(range);
    await userEvent.click(screen.getByRole('button', { name: 'Edit' }));
    expect((screen.getByRole('menuitem', { name: 'Cut' }) as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(screen.getByRole('menuitem', { name: 'Copy' })); expect(clipboard.writeText).toHaveBeenCalledWith('Reply for first'); expect(area.value).toBe('Unrelated draft');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); window.getSelection()!.removeAllRanges();
  });
  it('bounds zoom and menu keyboard navigation and bundles every help page without egress', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    for (let i = 0; i < 12; i++) fireEvent.keyDown(window, { key: '+', ctrlKey: true, shiftKey: true });
    expect(screen.getByLabelText('Interface zoom').textContent).toBe('150%');
    for (let i = 0; i < 12; i++) fireEvent.keyDown(window, { key: '-', ctrlKey: true });
    expect(screen.getByLabelText('Interface zoom').textContent).toBe('80%');
    fireEvent.keyDown(window, { key: '0', ctrlKey: true }); expect(screen.getByLabelText('Interface zoom').textContent).toBe('100%');
    const view = screen.getByRole('button', { name: 'View' }); view.focus(); await userEvent.keyboard('{ArrowDown}'); await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('menuitem', { name: 'Toggle Sidebar' })));
    await userEvent.keyboard('{End}'); expect(document.activeElement).toBe(screen.getByRole('menuitem', { name: 'Toggle Full Screen' }));
    await userEvent.keyboard('{ArrowRight}'); await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('menuitem', { name: 'Documentation' })));
    await userEvent.keyboard('{Escape}'); expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Help' }));
    for (const page of ['Documentation','Keyboard Shortcuts',"What's New",'Troubleshooting','System Status','About Aster']) {
      await menu('Help', page); expect(screen.getByRole('dialog', { name: page })).toBeTruthy(); await userEvent.keyboard('{Escape}');
    }
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureCredential).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(fake.bridge.deleteConversation).not.toHaveBeenCalled();
  });
  it('materialises an initial unsent draft before creating a fresh chat and never truncates over-limit navigation', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const area = screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
    fireEvent.change(area, { target: { value: 'Initial unsent draft' } }); await menu('File', 'New Chat');
    await waitFor(() => expect(fake.bridge.createConversation).toHaveBeenCalledTimes(2)); expect(area.value).toBe('');
    await menu('View', 'Next Chat'); await waitFor(() => expect(area.value).toBe('Initial unsent draft'));
    fireEvent.change(area, { target: { value: 'é'.repeat(16385) } }); await menu('View', 'Previous Chat');
    await screen.findByText(/Reduce the draft to 32,768 UTF-8 bytes before navigating/); expect(area.value.length).toBe(16385);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
});
