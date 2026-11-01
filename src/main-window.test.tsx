import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge } from './test-fixtures';

// Source/DOM evidence for the owner screenshot; this does not measure native
// WebView paint, installed window geometry or the reported startup flashes.
const { readFileSync } = await vi.importActual<{
  readFileSync(path: string, encoding: 'utf8'): string;
}>('node:fs');
const rendererCss = readFileSync('src/styles.css', 'utf8');
let stylesheet: HTMLStyleElement | null = null;

afterEach(() => { cleanup(); stylesheet?.remove(); stylesheet = null; vi.restoreAllMocks(); });
async function ready() { await screen.findByText('Ready for separately reviewed requests.'); }
function applyRendererStyles() {
  stylesheet = document.createElement('style');
  stylesheet.textContent = rendererCss;
  document.head.append(stylesheet);
}
function windowButtons() {
  return ['Minimise window', 'Maximise window', 'Close Aster'].map(name => screen.getByRole('button', { name }) as HTMLButtonElement);
}

describe('v0.2.4 single menu/caption row and transparent attachment glyph (UX-016 / AC-074 / T-084)', () => {
  it('places the four menus, flag and window controls in one row without a separate visible caption', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const rows = document.querySelectorAll('.titlebar, .captionbar');
    expect(rows).toHaveLength(1);
    const row = rows[0] as HTMLElement;
    expect(row.tagName).toBe('HEADER');
    expect(row.getAttribute('aria-label')).toBe('Window title bar');
    for (const name of ['File', 'Edit', 'View', 'Help']) expect(within(row).getByRole('button', { name })).toBeTruthy();
    const flag = within(row).getByRole('img', { name: 'English (United Kingdom)' });
    const controls = row.querySelector('.caption-controls');
    expect(flag.nextElementSibling).toBe(controls);
    for (const button of windowButtons()) expect(button.closest('header')).toBe(row);
    expect(row.querySelector('.caption-emblem')).toBeNull();
    expect(within(row).queryByText('Aster Desktop')).toBeNull();
    expect(fake.bridge.windowAction).not.toHaveBeenCalled();
  });

  it('confines drag and double-click intents to the empty caption space, using the existing closed route', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    const drag = document.querySelector('.caption-drag-region') as HTMLElement;
    expect(drag.textContent).toBe('');
    expect(drag.querySelector('button, img, svg')).toBeNull();
    fireEvent.mouseDown(screen.getByRole('button', { name: 'File' }), { button: 0, detail: 1 });
    fireEvent.doubleClick(screen.getByRole('button', { name: 'File' }));
    fireEvent.mouseDown(drag, { button: 2, detail: 1 });
    fireEvent.mouseDown(drag, { button: 0, detail: 2 });
    expect(fake.bridge.windowAction).not.toHaveBeenCalled();
    fireEvent.mouseDown(drag, { button: 0, detail: 1 });
    await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenCalledExactlyOnceWith({ action: 'start_dragging' }));
    await waitFor(() => expect(windowButtons()[0].disabled).toBe(false));
    fireEvent.doubleClick(drag);
    await waitFor(() => expect(fake.bridge.windowAction).toHaveBeenLastCalledWith({ action: 'toggle_maximise' }));
    expect(fake.bridge.windowAction).toHaveBeenCalledTimes(2);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('retains caption busy and modal guards after combining the rows', async () => {
    const active = fakeBridge({ ...baseline, availability: 'busy', activeRequest: true });
    render(<App bridge={active.bridge}/>); await screen.findByText('A request is in progress.');
    for (const button of windowButtons()) expect(button.disabled).toBe(true);
    fireEvent.mouseDown(document.querySelector('.caption-drag-region')!, { button: 0, detail: 1 });
    fireEvent.doubleClick(document.querySelector('.caption-drag-region')!);
    expect(active.bridge.windowAction).not.toHaveBeenCalled();
    cleanup();
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
    for (const button of windowButtons()) expect(button.disabled).toBe(true);
    fireEvent.mouseDown(document.querySelector('.caption-drag-region')!, { button: 0, detail: 1 });
    fireEvent.doubleClick(document.querySelector('.caption-drag-region')!);
    fireEvent.keyDown(window, { key: 'F11' });
    expect(fake.bridge.windowAction).not.toHaveBeenCalled();
  });

  it('uses one 44 px shell row and a white glowing plus on the transparent composer surface', async () => {
    applyRendererStyles();
    render(<App bridge={fakeBridge().bridge}/>); await ready();
    expect(getComputedStyle(document.querySelector('.app-shell')!).gridTemplateRows).toBe('44px minmax(0,1fr)');
    expect(getComputedStyle(document.querySelector('.titlebar')!).gridRow).toBe('1');
    for (const panel of document.querySelectorAll('.sidebar, .main-panel')) expect(getComputedStyle(panel).gridRow).toBe('2');
    const add = screen.getByRole('button', { name: 'Add attachments' });
    const model = add.nextElementSibling as HTMLElement;
    expect(model.classList.contains('fixed-model')).toBe(true);
    expect(model.textContent).toBe('Muse Spark 1.3 · Contributor');
    const style = getComputedStyle(add);
    expect(style.backgroundColor).toBe('rgba(0, 0, 0, 0)');
    expect(style.color).toBe('rgb(255, 255, 255)');
    expect(getComputedStyle(add.querySelector('svg')!).filter).toMatch(/drop-shadow/);
    expect(getComputedStyle(model).marginLeft).toBe('-4px');
    expect(screen.getByRole('button', { name: 'Reasoning effort' }).textContent).toContain('Medium');
    const send = screen.getByRole('button', { name: 'Send message' });
    expect(getComputedStyle(send).width).toBe('44px');
    expect(getComputedStyle(send).height).toBe('44px');
    expect(getComputedStyle(send).backgroundColor).toBe('rgb(100, 195, 253)');
  });
});
