import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge } from './test-fixtures';
import { shortcutAction } from './local-ui';

afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });
async function ready() { await screen.findByText('Ready for separately reviewed text requests.'); }
function wheel(init: WheelEventInit = {}) {
  // Web IDL rejects non-finite doubles; deliberately forge them after construction
  // to exercise the handler's malformed-event boundary, not the DOM constructor.
  const malformed = init.deltaY !== undefined && !Number.isFinite(init.deltaY);
  const event = new WheelEvent('wheel', { bubbles: true, cancelable: true, ...init, ...(malformed ? { deltaY: 0 } : {}) });
  if (malformed) Object.defineProperty(event, 'deltaY', { value: init.deltaY });
  act(() => document.body.dispatchEvent(event));
  return event;
}
function zoom() { return screen.getByLabelText('Interface zoom').textContent; }
function status() { return screen.queryByRole('status', { name: 'Zoom level' }); }
function noPresentationAuthority(fake: ReturnType<typeof fakeBridge>) {
  for (const method of ['sendMessage', 'cancelMessage', 'createConversation', 'deleteConversation', 'configureCredential', 'removeCredential', 'configureBudget', 'reconcileBudget', 'acceptContributorTerms', 'windowAction'] as const) expect(fake.bridge[method]).not.toHaveBeenCalled();
}

describe('v0.2.2 single zoom and transient status (T-071/T-072/T-074)', () => {
  it('recognises main-key aliases and keypad plus/minus without modifier-conflicting shortcuts', () => {
    for (const event of [{ key: '+', ctrlKey: true }, { key: '+', ctrlKey: true, shiftKey: true }, { key: '=', ctrlKey: true }, { key: '=', ctrlKey: true, shiftKey: true }, { key: '+', ctrlKey: true, code: 'NumpadAdd' }]) expect(shortcutAction(event)).toBe('zoomIn');
    for (const event of [{ key: '-', ctrlKey: true }, { key: '-', ctrlKey: true, code: 'NumpadSubtract' }]) expect(shortcutAction(event)).toBe('zoomOut');
    expect(shortcutAction({ key: '=', ctrlKey: true, altKey: true })).toBeNull();
    expect(shortcutAction({ key: '=', ctrlKey: true, isComposing: true })).toBeNull();
    expect(shortcutAction({ key: '=' })).toBeNull();
  });
  it('is hidden at default 100%, uses non-passive Ctrl-wheel and leaves ordinary scrolling alone', async () => {
    const listener = vi.spyOn(window, 'addEventListener'); const fake = fakeBridge();
    render(<App bridge={fake.bridge}/>); await ready();
    expect(zoom()).toBe('100%'); expect(status()).toBeNull();
    expect(listener.mock.calls.some(([type, , options]) => type === 'wheel' && typeof options === 'object' && options.passive === false)).toBe(true);
    const ordinary = wheel({ deltaY: -120 }); expect(ordinary.defaultPrevented).toBe(false); expect(zoom()).toBe('100%');
    const upwards = wheel({ ctrlKey: true, deltaY: -120 }); expect(upwards.defaultPrevented).toBe(true); expect(zoom()).toBe('110%');
    expect(status()?.textContent).toBe('110%');
    expect(status()?.closest('.app-shell')).toBeNull(); expect(status()?.getAttribute('aria-live')).toBe('polite');
    const downwards = wheel({ ctrlKey: true, deltaY: 3, deltaMode: WheelEvent.DOM_DELTA_LINE }); expect(downwards.defaultPrevented).toBe(true); expect(zoom()).toBe('100%');
    noPresentationAuthority(fake);
  });
  it('reuses keyboard/menu state, bounds each step and provides feedback even for a bound action', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready(); vi.useFakeTimers();
    fireEvent.keyDown(window, { key: '=', ctrlKey: true }); expect(zoom()).toBe('110%');
    fireEvent.keyDown(window, { key: '+', code: 'NumpadAdd', ctrlKey: true }); expect(zoom()).toBe('120%');
    fireEvent.click(screen.getByRole('button', { name: 'View' })); fireEvent.click(screen.getByRole('menuitem', { name: 'Zoom In' })); expect(zoom()).toBe('130%');
    for (let index = 0; index < 12; index++) wheel({ ctrlKey: true, deltaY: -1 }); expect(zoom()).toBe('150%');
    act(() => vi.advanceTimersByTime(1000)); expect(status()).toBeNull();
    wheel({ ctrlKey: true, deltaY: -1 }); expect(zoom()).toBe('150%'); expect(status()?.textContent).toBe('150%');
    act(() => vi.advanceTimersByTime(1000)); expect(status()).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'View' })); fireEvent.click(screen.getByRole('menuitem', { name: 'Zoom In' })); expect(zoom()).toBe('150%'); expect(status()?.textContent).toBe('150%');
    for (let index = 0; index < 12; index++) fireEvent.keyDown(window, { key: '-', code: 'NumpadSubtract', ctrlKey: true }); expect(zoom()).toBe('80%');
    fireEvent.keyDown(window, { key: '0', ctrlKey: true }); expect(zoom()).toBe('100%'); expect(status()?.textContent).toBe('100%');
    noPresentationAuthority(fake);
  });
  it('shows actual percent for 1000 ms from the latest action, keeps focus and cancels its sole timer on unmount', async () => {
    const fake = fakeBridge(); const view = render(<App bridge={fake.bridge}/>); await ready(); vi.useFakeTimers();
    const input = screen.getByRole('textbox', { name: 'Message for Aster' }); input.focus();
    wheel({ ctrlKey: true, deltaY: -100 }); expect(status()?.textContent).toBe('110%'); expect(document.activeElement).toBe(input);
    act(() => vi.advanceTimersByTime(999)); expect(status()).toBeTruthy();
    wheel({ ctrlKey: true, deltaY: -100 }); expect(status()?.textContent).toBe('120%');
    act(() => vi.advanceTimersByTime(1)); expect(status()).toBeTruthy();
    act(() => vi.advanceTimersByTime(998)); expect(status()).toBeTruthy();
    act(() => vi.advanceTimersByTime(1)); expect(status()).toBeNull();
    fireEvent.keyDown(window, { key: '0', ctrlKey: true }); expect(status()?.textContent).toBe('100%');
    act(() => vi.advanceTimersByTime(999)); fireEvent.keyDown(window, { key: '0', ctrlKey: true });
    act(() => vi.advanceTimersByTime(1)); expect(status()?.textContent).toBe('100%');
    expect(vi.getTimerCount()).toBe(1); view.unmount(); expect(vi.getTimerCount()).toBe(0);
    expect(wheel({ ctrlKey: true, deltaY: -10 }).defaultPrevented).toBe(false);
    noPresentationAuthority(fake);
  });
  it('cancels every Ctrl-wheel default but ignores malformed, horizontal and conflicting-modifier input', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    for (const init of [{ deltaY: 0 }, { deltaY: Number.NaN }, { deltaY: Number.POSITIVE_INFINITY }, { deltaY: 0, deltaX: 120 }, { deltaY: -1, shiftKey: true }, { deltaY: -1, altKey: true }]) {
      expect(wheel({ ctrlKey: true, ...init }).defaultPrevented).toBe(true); expect(zoom()).toBe('100%'); expect(status()).toBeNull();
    }
    noPresentationAuthority(fake);
  });
  it('blocks wheel/keyboard feedback behind a local dialogue while cancelling native zoom', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>); await ready();
    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    const focused = document.activeElement;
    expect(wheel({ ctrlKey: true, deltaY: -20 }).defaultPrevented).toBe(true);
    fireEvent.keyDown(window, { key: '=', ctrlKey: true });
    expect(zoom()).toBe('100%'); expect(status()).toBeNull(); expect(document.activeElement).toBe(focused);
    noPresentationAuthority(fake);
  });
  it('blocks active-request and in-flight native-review zoom without extending a previously visible notice', async () => {
    const active = fakeBridge({ ...baseline, activeRequest: true, availability: 'busy' }); render(<App bridge={active.bridge}/>); await screen.findByText('A request is in progress.');
    expect(wheel({ ctrlKey: true, deltaY: -20 }).defaultPrevented).toBe(true); expect(zoom()).toBe('100%'); expect(status()).toBeNull(); noPresentationAuthority(active); cleanup();
    const fake = fakeBridge(); let complete: (() => void) | undefined;
    vi.mocked(fake.bridge.windowAction!).mockImplementation(() => new Promise(resolve => { complete = () => resolve({ maximised: false, fullscreen: false }); }));
    render(<App bridge={fake.bridge}/>); await ready(); vi.useFakeTimers();
    wheel({ ctrlKey: true, deltaY: -20 }); act(() => vi.advanceTimersByTime(600));
    fireEvent.click(screen.getByRole('button', { name: 'Minimise window' }));
    expect(wheel({ ctrlKey: true, deltaY: -20 }).defaultPrevented).toBe(true); expect(zoom()).toBe('110%');
    act(() => vi.advanceTimersByTime(400)); expect(status()).toBeNull();
    await act(async () => { complete!(); await Promise.resolve(); });
    expect(fake.bridge.windowAction).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
  });
});
