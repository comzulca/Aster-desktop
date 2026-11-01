import { utf8Bytes, validId } from './policy';
import { scalarCount, wellFormedText } from './personalization-ui';
import { validCustomizationDraft } from './customization-ui';
import type { CustomizationDraft, DataClass, PackageKind } from './types';

export const MAX_GENERATED_BYTES = 64 * 1024;
export function validGenerationDescription(value: string): boolean {
  return Boolean(value.trim()) && wellFormedText(value) && scalarCount(value) <= 4000 && utf8Bytes(value) <= 16000 && !/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/u.test(value);
}
export function validGenerationIntent(value: unknown): value is { requestId: string; kind: PackageKind; description: string; dataClass: DataClass } {
  if (typeof value !== 'object' || value === null || Array.isArray(value) || Object.keys(value).sort().join('|') !== 'dataClass|description|kind|requestId') return false;
  const input = value as Record<string, unknown>;
  return typeof input.requestId === 'string' && /^[a-f0-9]{32}$/.test(input.requestId) && ['skill', 'plugin'].includes(String(input.kind)) && typeof input.description === 'string' && validGenerationDescription(input.description) && ['synthetic', 'public_eligible'].includes(String(input.dataClass));
}
export function validGenerationReceipt(value: unknown, requestId: string): value is { conversationId: string; requestId: string; accepted: true } {
  return typeof value === 'object' && value !== null && !Array.isArray(value) && Object.keys(value).sort().join('|') === 'accepted|conversationId|requestId' &&
    (value as Record<string, unknown>).requestId === requestId && validId((value as Record<string, unknown>).conversationId) && (value as Record<string, unknown>).accepted === true;
}
// One flat JSON object whose five values are strings. Parse token boundaries
// explicitly to reject duplicate/escaped duplicate keys, nesting and fences.
// Generated prose never becomes HTML, a script, a capability or a tool call.
export function parseGeneratedDraft(text: string, kind: PackageKind): CustomizationDraft | null {
  if (utf8Bytes(text) > MAX_GENERATED_BYTES || !wellFormedText(text)) return null;
  let offset = 0;
  const keys = new Set(['kind', 'slug', 'name', 'description', 'instructions']);
  const values: Record<string, string> = {};
  const whitespace = () => { while (offset < text.length && /[ \t\r\n]/.test(text[offset])) offset++; };
  function string(): string {
    if (text[offset] !== '"') throw new Error('invalid_draft');
    const start = offset++; let escaped = false;
    while (offset < text.length) {
      const character = text[offset++];
      if (escaped) { escaped = false; continue; }
      if (character === '\\') { escaped = true; continue; }
      if (character === '"') return JSON.parse(text.slice(start, offset)) as string;
    }
    throw new Error('invalid_draft');
  }
  try {
    whitespace(); if (text[offset++] !== '{') return null; whitespace();
    while (true) {
      const key = string(); if (!keys.has(key)) return null; keys.delete(key);
      whitespace(); if (text[offset++] !== ':') return null; whitespace(); values[key] = string(); whitespace();
      const delimiter = text[offset++]; if (delimiter === '}') break; if (delimiter !== ',') return null; whitespace();
    }
    whitespace();
    return offset === text.length && keys.size === 0 && values.kind === kind && validCustomizationDraft(values) ? values : null;
  } catch { return null; }
}
