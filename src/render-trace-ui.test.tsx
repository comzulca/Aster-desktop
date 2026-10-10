import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { RenderingDiagnostics } from './RenderingDiagnostics';
import { validRenderTrace } from './render-trace-ui';
import { fakeBridge } from './test-fixtures';
import type { NativeTracePhase, RenderTraceSnapshot } from './types';

const snapshot: RenderTraceSnapshot = { schemaVersion: 1, classification: 'bounded-observation-not-a-rendering-cure', visibleAtMs: 4, elapsedMs: 10, dropped: 0, events: [{ elapsedMs: 0, source: 'native', phase: 'host_created' }, { elapsedMs: 4, source: 'native', phase: 'parent_visible' }, { elapsedMs: 5, source: 'renderer', phase: 'visible' }], webview2DetailEventsAvailable: false };
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
describe('bounded rendering observation, not a painted-pixel or cure claim (T-086)', () => {
  it('rejects private fields, unknown phases, overflow and manufactured native visibility', () => {
    expect(validRenderTrace(snapshot)).toBe(true);
    for (const value of [{ ...snapshot, url: 'PRIVATE_DIAGNOSTIC_CANARY' }, { ...snapshot, webview2DetailEventsAvailable: 'true' }, { ...snapshot, webview2DetailEventsAvailable: 1 }, { ...snapshot, elapsedMs: 60_001 }, { ...snapshot, dropped: 4_294_967_296 }, { ...snapshot, events: Array(97).fill(snapshot.events[0]) }, { ...snapshot, visibleAtMs: 5 }, { ...snapshot, events: [{ elapsedMs: 1, source: 'renderer', phase: 'parent_visible' }] }, { ...snapshot, events: [{ elapsedMs: 1, source: 'renderer', phase: 'observer_attached' }] }, { ...snapshot, events: [{ elapsedMs: 1, source: 'renderer', phase: 'navigation_completed' }] }, { ...snapshot, events: [{ elapsedMs: 1, source: 'native', phase: 'private text' }] }, { ...snapshot, events: [{ elapsedMs: 1, source: 'native', phase: 'host_created', content: 'private' }] }, { ...snapshot, events: [...snapshot.events, { elapsedMs: 2, source: 'renderer', phase: 'hidden' }] }]) expect(validRenderTrace(value)).toBe(false);
    expect(validRenderTrace({ ...snapshot, visibleAtMs: null, events: [{ elapsedMs: 1, source: 'renderer', phase: 'visible' }] })).toBe(true);
  });
  it('admits only the fixed native WebView2 categories and does not equate controller visibility with parent visibility', () => {
    const phases: NativeTracePhase[] = ['observer_attached', 'observer_unavailable', 'observer_detached', 'navigation_started', 'content_loading', 'dom_content_loaded', 'navigation_completed', 'navigation_failed', 'process_failed', 'controller_visible', 'controller_hidden', 'controller_unavailable', 'background_dark', 'background_other', 'background_unavailable', 'zoom_default', 'zoom_changed', 'zoom_unavailable'];
    const receipt = { ...snapshot, visibleAtMs: null, elapsedMs: phases.length, webview2DetailEventsAvailable: true, events: phases.map((phase, index) => ({ elapsedMs: index, source: 'native', phase })) };
    expect(validRenderTrace(receipt)).toBe(true);
    expect(validRenderTrace({ ...receipt, webview2DetailEventsAvailable: false })).toBe(true);
    expect(validRenderTrace({ ...receipt, visibleAtMs: 9 })).toBe(false);
    for (const phase of phases) expect(validRenderTrace({ ...receipt, events: [{ elapsedMs: 0, source: 'renderer', phase }] })).toBe(false);
    for (const phase of ['compositor_ready', 'painted', 'navigation_uri', 'zoom:1.0']) expect(validRenderTrace({ ...receipt, events: [{ elapsedMs: 0, source: 'native', phase }] })).toBe(false);
  });
  it('reads an exact inert observation only after an explicit click and never sends or exports it', async () => {
    const fake = fakeBridge(); fake.bridge.getRenderTrace = vi.fn(async () => structuredClone(snapshot));
    render(<RenderingDiagnostics bridge={fake.bridge}/>);
    expect(fake.bridge.getRenderTrace).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Read rendering diagnostics' }));
    const trace = await screen.findByLabelText('Local rendering observation'); expect(trace.textContent).toContain('bounded-observation-not-a-rendering-cure');
    expect(screen.getByRole('status').textContent).toContain('detailed WebView2 observation unavailable');
    expect(fake.bridge.getRenderTrace).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(screen.queryByRole('link')).toBeNull();
  });
  it('reports observer availability from each fresh captured receipt without claiming painted pixels or sending it', async () => {
    const fake = fakeBridge(); fake.bridge.getRenderTrace = vi.fn().mockResolvedValueOnce({ ...snapshot, webview2DetailEventsAvailable: true }).mockResolvedValueOnce(structuredClone(snapshot));
    render(<RenderingDiagnostics bridge={fake.bridge}/>);
    expect(screen.queryByRole('status')).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Read rendering diagnostics' }));
    await screen.findByText(/captured host receipt reports the full fixed WebView2 observer set registered/);
    expect(screen.getByText(/Initial navigation may predate observer attachment/)).toBeTruthy();
    await userEvent.click(screen.getByRole('button', { name: 'Read rendering diagnostics' }));
    await screen.findByText(/captured host receipt reports detailed WebView2 observation unavailable/);
    expect(screen.queryByText(/full fixed WebView2 observer set registered/)).toBeNull();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(screen.queryByRole('link')).toBeNull();
  });
  it('does not display unexpected private payloads or native errors', async () => {
    const fake = fakeBridge(); fake.bridge.getRenderTrace = vi.fn(async () => ({ ...snapshot, content: 'PRIVATE_DIAGNOSTIC_CANARY' }));
    render(<RenderingDiagnostics bridge={fake.bridge}/>); await userEvent.click(screen.getByRole('button', { name: 'Read rendering diagnostics' }));
    await screen.findByText(/local rendering observation is unavailable/); expect(document.body.textContent).not.toContain('PRIVATE_DIAGNOSTIC_CANARY'); expect(screen.queryByLabelText('Local rendering observation')).toBeNull();
  });
  it('submits fixed local markers with a churn cap and removes listeners on unmount', async () => {
    const fake = fakeBridge(); fake.bridge.recordRenderEvent = vi.fn(async () => true);
    const view = render(<App bridge={fake.bridge}/>); await screen.findByText('Ready for separately reviewed requests.');
    for (const image of document.querySelectorAll('img')) fireEvent.load(image);
    await waitFor(() => expect(fake.bridge.recordRenderEvent).toHaveBeenCalledWith({ protocolVersion: 1, phase: 'assets_ready' }));
    const calls = vi.mocked(fake.bridge.recordRenderEvent).mock.calls;
    expect(calls.some(([input]) => input.phase === 'mounted')).toBe(true); expect(calls.some(([input]) => input.phase === 'hydration_started')).toBe(true); expect(calls.some(([input]) => input.phase === 'hydration_finished')).toBe(true);
    for (let index = 0; index < 150; index++) document.dispatchEvent(new Event('visibilitychange'));
    expect(fake.bridge.recordRenderEvent).toHaveBeenCalledTimes(32);
    expect(calls.every(([input]) => Object.keys(input).sort().join('|') === 'phase|protocolVersion')).toBe(true); expect(JSON.stringify(calls)).not.toMatch(/visibleAtMs|url|path|content|PRIVATE/);
    view.unmount(); document.dispatchEvent(new Event('visibilitychange')); expect(fake.bridge.recordRenderEvent).toHaveBeenCalledTimes(32); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('does not call a failed asset ready or introduce a new visibility wait', async () => {
    const fake = fakeBridge(); fake.bridge.recordRenderEvent = vi.fn(async () => true); fake.bridge.shellReady = vi.fn(async () => ({ accepted: true, visible: true }));
    render(<App bridge={fake.bridge}/>); await screen.findByText('Ready for separately reviewed requests.');
    const images = document.querySelectorAll('img'); fireEvent.error(images[0]); for (const image of [...images].slice(1)) fireEvent.load(image);
    await waitFor(() => expect(fake.bridge.shellReady).toHaveBeenCalledOnce());
    expect(vi.mocked(fake.bridge.recordRenderEvent).mock.calls.some(([input]) => input.phase === 'assets_ready')).toBe(false);
  });
});
