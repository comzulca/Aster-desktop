import { utf8Bytes } from './policy';
import { scalarCount, wellFormedText } from './personalization-ui';
import type { CustomizationDraft, CustomizationPackage, CustomizationSnapshot } from './types';

const record = (value: unknown): value is Record<string, unknown> => Boolean(value) && typeof value === 'object' && !Array.isArray(value);
const exact = (value: Record<string, unknown>, keys: string[]) => Object.keys(value).sort().join('|') === [...keys].sort().join('|');
const revision = (value: unknown) => typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 && value <= 512;
const bounded = (value: unknown, characters: number, bytes: number, multiline = false) => typeof value === 'string' && wellFormedText(value) && scalarCount(value) <= characters && utf8Bytes(value) <= bytes && !(multiline ? /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/u : /[\u0000-\u001f\u007f-\u009f]/u).test(value);
const validEntryId = (value: unknown) => typeof value === 'string' && /^[a-zA-Z0-9:_-]{1,100}$/.test(value);
export function validCustomizationDraft(value: unknown): value is CustomizationDraft {
  return record(value) && exact(value, ['kind','slug','name','description','instructions']) && ['skill','plugin'].includes(String(value.kind)) &&
    typeof value.slug === 'string' && value.slug.length >= 3 && value.slug.length <= 64 && /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(value.slug) &&
    bounded(value.name, 80, 320) && Boolean((value.name as string).trim()) && bounded(value.description, 320, 1280) &&
    bounded(value.instructions, 12000, 48000, true) && Boolean((value.instructions as string).trim());
}
export function validCustomization(value: unknown): value is CustomizationSnapshot {
  if (!record(value) || !exact(value, ['revision','entries']) || !revision(value.revision) || !Array.isArray(value.entries) || value.entries.length > 66) return false;
  return value.entries.every(item => record(item) && exact(item, ['id','kind','name','description','enabled','readOnly','availability','entryRevision']) &&
    validEntryId(item.id) && ['skill','plugin'].includes(String(item.kind)) && bounded(item.name, 80, 320) && bounded(item.description, 320, 1280) &&
    typeof item.enabled === 'boolean' && typeof item.readOnly === 'boolean' && revision(item.entryRevision) &&
    ['instructions-only','built-in-connector','reference-unavailable'].includes(String(item.availability)) &&
    (item.availability !== 'reference-unavailable' || item.enabled === false)) && new Set(value.entries.map(item => item.id)).size === value.entries.length;
}
export function validCustomizationPackage(value: unknown): value is CustomizationPackage {
  if (!record(value) || !exact(value, ['id','kind','slug','name','description','instructions','enabled','entryRevision','readOnly'])) return false;
  return validEntryId(value.id) && typeof value.enabled === 'boolean' && typeof value.readOnly === 'boolean' && revision(value.entryRevision) &&
    validCustomizationDraft({ kind: value.kind, slug: value.slug, name: value.name, description: value.description, instructions: value.instructions });
}
