import { DEFAULT_MODEL, type AppStatus, type Conversation, type ConversationSummary, type StreamEvent } from './types';

export const MODEL_LABEL = 'Muse Spark 1.3 · Contributor';
export const MAX_MESSAGE_BYTES = 32 * 1024;
export const MAX_CONVERSATION_BYTES = 512 * 1024;
export const MAX_CONVERSATION_ENTRIES = 256;
export const REASONING_EFFORTS = ['minimal', 'low', 'medium', 'high', 'xhigh'] as const;
const terminalStates = ['completed', 'failed', 'cancelled', 'truncated', 'refused'] as const;
const record = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);
const exact = (v: Record<string, unknown>, keys: string[]) => Object.keys(v).sort().join('|') === [...keys].sort().join('|');
export const validId = (v: unknown): v is string => typeof v === 'string' && /^[A-Za-z0-9_-]{1,80}$/.test(v);
const safeInteger = (v: unknown): v is number => typeof v === 'number' && Number.isSafeInteger(v) && v >= 0;
const integerString = (v: unknown): v is string => typeof v === 'string' && /^(0|[1-9][0-9]{0,24})$/.test(v);
export const utf8Bytes = (text: string) => new TextEncoder().encode(text).length;
export function validMessage(text: string): boolean {
  // Reject unpaired surrogates rather than changing the bytes that are reviewed.
  return Boolean(text.trim()) && !text.includes('\0') && !/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/u.test(text) && utf8Bytes(text) <= MAX_MESSAGE_BYTES;
}
export function validStatus(v: unknown): v is AppStatus {
  if (!record(v) || !exact(v, ['protocolVersion','policyVersion','configured','credentialAvailable','defaultModel','models','activeRequest','projectSelected','projectLabel','capabilities','availability','contributor','historyAvailable','budget'])) return false;
  if (v.protocolVersion !== 3 || v.policyVersion !== 3 || v.defaultModel !== DEFAULT_MODEL ||
      !['configured','credentialAvailable','activeRequest','historyAvailable'].every(k => typeof v[k] === 'boolean') ||
      v.projectSelected !== false || v.projectLabel !== null || !Array.isArray(v.models) || v.models.length !== 1 ||
      !record(v.models[0]) || !exact(v.models[0], ['id','label']) || v.models[0].id !== DEFAULT_MODEL ||
      typeof v.models[0].label !== 'string' || v.models[0].label.length > 100 ||
      !['needs_terms','needs_credential','needs_budget','attention_required','ready','busy','unsupported'].includes(String(v.availability))) return false;
  const c = v.capabilities, t = v.contributor, b = v.budget;
  if (!record(c) || !exact(c, ['projectRead','proposals','applyNewFile','applyExistingFile']) || Object.values(c).some(x => x !== false) ||
      !record(t) || !exact(t, ['termsRevision','accepted','eligible']) || typeof t.termsRevision !== 'string' || t.termsRevision.length > 100 || typeof t.accepted !== 'boolean' || typeof t.eligible !== 'boolean' ||
      !record(b) || !exact(b, ['accountId','pending','configured','period','limitNanoGbp','spentNanoGbp','reservedNanoGbp','remainingNanoGbp','warning','requiresReconciliation','configurationExpiresAt']) ||
      !(b.accountId === null || (typeof b.accountId === 'string' && /^[a-f0-9]{32}$/.test(b.accountId))) ||
      !['configured','warning','requiresReconciliation'].every(k => typeof b[k] === 'boolean') ||
      !(b.period === null || (typeof b.period === 'string' && /^\d{4}-\d{2}$/.test(b.period))) ||
      !['limitNanoGbp','spentNanoGbp','reservedNanoGbp','remainingNanoGbp'].every(k => integerString(b[k])) ||
      b.limitNanoGbp !== '89000000000' || !(b.configurationExpiresAt === null || safeInteger(b.configurationExpiresAt))) return false;
  if (!Array.isArray(b.pending) || b.pending.length > 1024 || !b.pending.every(item => record(item) && exact(item, ['requestId','period','reservationNanoGbp','dispatched']) && validId(item.requestId) && typeof item.period === 'string' && /^\d{4}-\d{2}$/.test(item.period) && integerString(item.reservationNanoGbp) && typeof item.dispatched === 'boolean') || new Set(b.pending.map(item => item.requestId)).size !== b.pending.length) return false;
  if (v.availability === 'ready' && (!v.configured || !v.credentialAvailable || !v.historyAvailable || !t.accepted || !t.eligible || !b.configured || b.requiresReconciliation || v.activeRequest)) return false;
  return true;
}
export function validStreamEvent(v: unknown): v is StreamEvent {
  if (!record(v) || !validId(v.requestId) || !validId(v.conversationId)) return false;
  if (v.type === 'content') return exact(v, ['requestId','conversationId','type','text']) && typeof v.text === 'string' && utf8Bytes(v.text) <= 256 * 1024;
  return v.type === 'terminal' && exact(v, v.errorCode === undefined ? ['requestId','conversationId','type','status'] : ['requestId','conversationId','type','status','errorCode']) &&
    terminalStates.includes(v.status as typeof terminalStates[number]) && (v.errorCode === undefined || (typeof v.errorCode === 'string' && /^[a-z0-9_]{1,80}$/.test(v.errorCode)));
}
export function validSummary(v: unknown): v is ConversationSummary {
  return record(v) && exact(v, ['id','title','updatedAt','entryCount']) && validId(v.id) && typeof v.title === 'string' && utf8Bytes(v.title) <= 256 && safeInteger(v.updatedAt) && safeInteger(v.entryCount) && v.entryCount <= MAX_CONVERSATION_ENTRIES;
}
export function validSummaries(v: unknown): v is ConversationSummary[] {
  return Array.isArray(v) && v.length <= 128 && v.every(validSummary) && new Set(v.map(x => x.id)).size === v.length;
}
export function validConversation(v: unknown): v is Conversation {
  if (!record(v) || !exact(v, ['id','title','updatedAt','entries']) || !validSummary({ id: v.id, title: v.title, updatedAt: v.updatedAt, entryCount: Array.isArray(v.entries) ? v.entries.length : -1 }) || !Array.isArray(v.entries)) return false;
  let bytes = 0;
  const ids = new Set<string>();
  return v.entries.every(e => {
    if (!record(e) || !exact(e, ['id','role','content','requestId','state']) || !validId(e.id) || ids.has(e.id) || !validId(e.requestId) || !['user','assistant'].includes(String(e.role)) || typeof e.content !== 'string' || ![...terminalStates, 'interrupted'].includes(e.state as 'completed')) return false;
    ids.add(e.id); bytes += utf8Bytes(e.content);
    return bytes <= MAX_CONVERSATION_BYTES;
  });
}
export function formatGbp(nano: string): string {
  if (!integerString(nano)) return 'Unavailable';
  const value = BigInt(nano), pennies = value / 10_000_000n;
  return `£${(pennies / 100n).toLocaleString('en-GB')}.${(pennies % 100n).toString().padStart(2, '0')}`;
}
export function parseGbp(value: string): string | null {
  if (!/^(0|[1-9][0-9]{0,2})(\.[0-9]{1,9})?$/.test(value)) return null;
  const [pounds, fraction = ''] = value.split('.');
  const nanos = BigInt(pounds) * 1_000_000_000n + BigInt(fraction.padEnd(9, '0'));
  return nanos <= 89_000_000_000n ? nanos.toString() : null;
}
