import { utf8Bytes } from './policy';
import type { PersonalizationProfile, PersonalizationSnapshot } from './types';

export const PROFILE_LIMITS = {
  customInstructions: { label: 'Custom instructions', characters: 2000, bytes: 8000 },
  nickname: { label: 'Nickname', characters: 80, bytes: 320 },
  occupation: { label: 'Occupation', characters: 120, bytes: 480 },
  moreAboutYou: { label: 'More about you', characters: 1000, bytes: 4000 },
} as const;
export const emptyProfile = (): PersonalizationProfile => ({ customInstructions: '', nickname: '', occupation: '', moreAboutYou: '' });
export const scalarCount = (text: string) => Array.from(text).length;
export const wellFormedText = (text: string) => !text.includes('\0') && !/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/u.test(text);
export function validProfile(value: unknown): value is PersonalizationProfile {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).sort().join('|') !== Object.keys(PROFILE_LIMITS).sort().join('|')) return false;
  let characters = 0, bytes = 0;
  for (const key of Object.keys(PROFILE_LIMITS) as (keyof PersonalizationProfile)[]) {
    const text = (value as PersonalizationProfile)[key], limit = PROFILE_LIMITS[key];
    const multiline = key === 'customInstructions' || key === 'moreAboutYou';
    if (typeof text !== 'string' || !wellFormedText(text) || scalarCount(text) > limit.characters || utf8Bytes(text) > limit.bytes ||
      (multiline ? /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/u : /[\u0000-\u001f\u007f-\u009f]/u).test(text)) return false;
    characters += scalarCount(text); bytes += utf8Bytes(text);
  }
  return characters <= 3200 && bytes <= 12800;
}
export function validPersonalization(value: unknown): value is PersonalizationSnapshot {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).sort().join('|') !== 'profile|revision') return false;
  const snapshot = value as PersonalizationSnapshot;
  return Number.isSafeInteger(snapshot.revision) && snapshot.revision >= 0 && snapshot.revision <= 512 && validProfile(snapshot.profile);
}
