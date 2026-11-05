import { vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DEFAULT_MODEL, type AppStatus, type Bridge, type Conversation, type StreamEvent } from './types';

export const baseline: AppStatus = {
  protocolVersion: 4, policyVersion: 4, configured: true, credentialAvailable: true,
  credentialValidation: 'valid',
  defaultModel: DEFAULT_MODEL, models: [{ id: DEFAULT_MODEL, label: 'Muse Spark 1.3 · Contributor' }],
  activeRequest: false, projectSelected: false, projectLabel: null,
  capabilities: { projectRead: false, proposals: false, applyNewFile: false, applyExistingFile: false, attachmentImport: false },
  availability: 'ready', contributor: { termsRevision: '2026-10-09', accepted: true, eligible: true }, historyAvailable: true,
  budget: { accountId: 'a'.repeat(32), pending: [], configured: true, period: '2026-10', limitNanoGbp: '89000000000', spentNanoGbp: '0', reservedNanoGbp: '0', remainingNanoGbp: '89000000000', warning: false, requiresReconciliation: false, configurationExpiresAt: 1790964000 },
};
export async function selectClassification(value: 'synthetic' | 'public_eligible') {
  await userEvent.click(screen.getByRole('button', { name: 'Reasoning effort' }));
  await userEvent.click(screen.getByRole('menuitem', { name: 'Classify content' }));
  await userEvent.click(screen.getByRole('menuitemradio', { name: value === 'synthetic' ? 'Synthetic' : 'Eligible public' }));
}
export function saved(id: string, title: string, updatedAt = 1): Conversation {
  return { id, title, updatedAt, entries: [{ id: `${id}_u`, role: 'user', content: title, requestId: `${id}_r`, state: 'completed' }, { id: `${id}_a`, role: 'assistant', content: `Reply for ${id}`, requestId: `${id}_r`, state: 'completed' }] };
}
export function fakeBridge(status: AppStatus = baseline, initial: Conversation[] = []) {
  const records = new Map(initial.map(item => [item.id, structuredClone(item)]));
  let onEvent: (event: StreamEvent) => void = () => {};
  let onClose: () => void = () => {};
  let sequence = 0;
  const finished = new Set<string>();
  const bridge: Bridge = {
    windowAction: vi.fn(async () => ({ maximised: false, fullscreen: false })),
    subscribeClose: vi.fn(async callback => { onClose = callback; return () => { onClose = () => {}; }; }),
    getStatus: vi.fn(async () => structuredClone(status)),
    subscribe: vi.fn(async callback => { onEvent = callback; return () => {}; }),
    configureCredential: vi.fn(async () => ({ configured: true })),
    removeCredential: vi.fn(async () => ({ configured: false })),
    verifyCredential: vi.fn(async () => ({ state: 'valid' as const })),
    acceptContributorTerms: vi.fn(async () => ({ accepted: true })),
    configureBudget: vi.fn(async () => ({ configured: true })),
    reconcileBudget: vi.fn(async () => ({ reconciled: true })),
    listConversations: vi.fn(async () => [...records.values()].map(item => ({ id: item.id, title: item.title, updatedAt: item.updatedAt, entryCount: item.entries.length }))),
    createConversation: vi.fn(async () => { const item: Conversation = { id: `new_${++sequence}`, title: 'Untitled conversation', updatedAt: sequence, entries: [] }; records.set(item.id, item); return structuredClone(item); }),
    loadConversation: vi.fn(async ({ conversationId }) => { const item = records.get(conversationId); if (!item) throw new Error('history_unavailable'); return structuredClone(item); }),
    deleteConversation: vi.fn(async ({ conversationId }) => ({ deleted: records.delete(conversationId) })),
    sendMessage: vi.fn(async input => {
      const item = records.get(input.conversationId)!; item.title = input.message.slice(0, 60); item.updatedAt++;
      item.entries.push({ id: `${input.requestId}_u`, role: 'user', content: input.message, requestId: input.requestId, state: 'completed' }, { id: `${input.requestId}_a`, role: 'assistant', content: '', requestId: input.requestId, state: 'interrupted' });
      return { requestId: input.requestId, accepted: true };
    }),
    cancelMessage: vi.fn(async () => ({ cancelled: true })),
  };
  return { bridge, records, close: () => onClose(), emit(event: StreamEvent) {
    const item = records.get(event.conversationId);
    const entry = item?.entries.find(e => e.role === 'assistant' && e.requestId === event.requestId);
    if (entry && !finished.has(event.requestId)) {
      if (event.type === 'content') entry.content += event.text ?? '';
      else { entry.state = event.status!; finished.add(event.requestId); }
    }
    onEvent(event);
  } };
}
