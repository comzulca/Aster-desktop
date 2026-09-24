import { DEFAULT_MODEL, MUSE_MODEL, type AppStatus, type ModelId, type StreamEvent } from './types';

export const MODELS: readonly { id: ModelId; label: string }[] = [
  { id: DEFAULT_MODEL, label: 'Nemotron 3.5 Lightning 30B A3B' },
  { id: MUSE_MODEL, label: 'Muse Glimmer 30B' },
];

// The UI is never the source of provider or file authority. Fail closed on a
// host registry incompatible with the protocol the UI knows how to display.
export function validStatus(value: unknown): value is AppStatus {
  if (typeof value !== 'object' || value === null) return false;
  const s = value as Partial<AppStatus>;
  if (s.policyVersion !== 1 || typeof s.configured !== 'boolean' ||
      s.defaultModel !== DEFAULT_MODEL || !Array.isArray(s.models) || s.models.length !== 2 ||
      typeof s.activeRequest !== 'boolean' || typeof s.projectSelected !== 'boolean' ||
      !(s.projectLabel === null || typeof s.projectLabel === 'string') ||
      !s.capabilities || typeof s.capabilities !== 'object') return false;
  const c = s.capabilities;
  if (c.projectRead !== false || c.proposals !== false || c.applyNewFile !== false || c.applyExistingFile !== false) return false;
  const actual = s.models.map((m) => m && typeof m === 'object' ? m.id : '').sort();
  return actual.join('|') === [DEFAULT_MODEL, MUSE_MODEL].sort().join('|') &&
    s.models.every((m) => typeof m.label === 'string' && m.label.length <= 100);
}

export function validStreamEvent(value: unknown): value is StreamEvent {
  if (typeof value !== 'object' || value === null) return false;
  const e = value as Partial<StreamEvent>;
  if (typeof e.requestId !== 'string' || typeof e.conversationId !== 'string' ||
      e.requestId.length > 100 || e.conversationId.length > 100) return false;
  if (e.type === 'content') return typeof e.text === 'string' && e.text.length <= 262_144;
  if (e.type === 'terminal') return e.status === 'completed' || e.status === 'failed' || e.status === 'cancelled';
  return false;
}

export function safeRelativePath(path: string): boolean {
  if (!path || path.length > 260 || path.startsWith('/') || path.startsWith('\\') ||
      /[\\:\u0000-\u001f]/.test(path)) return false;
  const parts = path.split('/');
  return parts.every((part) => part.length > 0 && part !== '.' && part !== '..' &&
    part.trim() === part && !/[. ]$/.test(part));
}
