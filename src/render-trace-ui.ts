import type { RenderTraceSnapshot } from './types';

const rendererPhases = new Set(['mounted', 'assets_ready', 'hydration_started', 'hydration_finished', 'visible', 'hidden']);
const nativePhases = new Set(['host_created', 'parent_created', 'page_started', 'page_finished', 'parent_visible', 'visibility_denied', 'resized', 'scale_changed', 'focused', 'unfocused', 'destroyed',
  'observer_attached', 'observer_unavailable', 'observer_detached', 'navigation_started', 'content_loading', 'dom_content_loaded', 'navigation_completed', 'navigation_failed', 'process_failed',
  'controller_visible', 'controller_hidden', 'controller_unavailable', 'background_dark', 'background_other', 'background_unavailable', 'zoom_default', 'zoom_changed', 'zoom_unavailable']);
function exact(value: unknown, keys: string[]): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value) && Object.keys(value).sort().join('|') === [...keys].sort().join('|');
}
const tick = (value: unknown): value is number => Number.isSafeInteger(value) && (value as number) >= 0 && (value as number) <= 60_000;

// This validates a fixed, content-free observation DTO only. Renderer markers
// cannot establish native visibility, painted pixels, or any host authority.
export function validRenderTrace(value: unknown): value is RenderTraceSnapshot {
  if (!exact(value, ['schemaVersion', 'classification', 'visibleAtMs', 'elapsedMs', 'dropped', 'events', 'webview2DetailEventsAvailable']) || value.schemaVersion !== 1 || value.classification !== 'bounded-observation-not-a-rendering-cure' || typeof value.webview2DetailEventsAvailable !== 'boolean' || !tick(value.elapsedMs) || !Number.isSafeInteger(value.dropped) || (value.dropped as number) < 0 || (value.dropped as number) > 4_294_967_295 || !Array.isArray(value.events) || value.events.length > 96 || (value.visibleAtMs !== null && !tick(value.visibleAtMs))) return false;
  let previous = 0;
  let firstNativeVisible: number | null = null;
  for (const event of value.events) {
    if (!exact(event, ['elapsedMs', 'source', 'phase']) || !tick(event.elapsedMs) || event.elapsedMs < previous || event.elapsedMs > value.elapsedMs || typeof event.phase !== 'string') return false;
    if (event.source === 'native') { if (!nativePhases.has(event.phase)) return false; }
    else if (event.source === 'renderer') { if (!rendererPhases.has(event.phase)) return false; }
    else return false;
    if (event.source === 'native' && event.phase === 'parent_visible' && firstNativeVisible === null) firstNativeVisible = event.elapsedMs;
    previous = event.elapsedMs;
  }
  return value.visibleAtMs === firstNativeVisible;
}
