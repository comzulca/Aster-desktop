import { describe, expect, it } from 'vitest';
import { formatGbp, parseGbp, validConversation, validMessage, validStatus, validStreamEvent, validSummaries } from './policy';
import { baseline, saved } from './test-fixtures';

describe('protocol 3 renderer validation (T-055/T-061)', () => {
  it('admits exactly the reviewed fixed model and false capabilities', () => {
    expect(validStatus(baseline)).toBe(true);
    for (const version of [1, 2, 4]) expect(validStatus({ ...baseline, protocolVersion: version })).toBe(false);
    expect(validStatus({ ...baseline, policyVersion: 2 })).toBe(false);
    expect(validStatus({ ...baseline, approved: true })).toBe(false);
    for (const model of ['meta/muse-glimmer-30b','nvidia/nemotron-3.5-lightning-30b-a3b','muse-spark-1.3-standard']) expect(validStatus({ ...baseline, models: [{ id: model, label: 'Unsupported' }] })).toBe(false);
    for (const key of Object.keys(baseline.capabilities)) expect(validStatus({ ...baseline, capabilities: { ...baseline.capabilities, [key]: true } })).toBe(false);
    expect(validStatus({ ...baseline, projectSelected: true })).toBe(false);
    expect(validStatus({ ...baseline, availability: 'ready', configured: false })).toBe(false);
    expect(validStatus({ ...baseline, availability: 'ready', contributor: { ...baseline.contributor, eligible: false } })).toBe(false);
  });
  it('denies malformed financial metadata, unknown fields and duplicate pending identities', () => {
    for (const value of ['-1','1.2','1e3','00',42]) expect(validStatus({ ...baseline, budget: { ...baseline.budget, spentNanoGbp: value } })).toBe(false);
    expect(validStatus({ ...baseline, budget: { ...baseline.budget, limitNanoGbp: '90000000000' } })).toBe(false);
    expect(validStatus({ ...baseline, budget: { ...baseline.budget, accountId: 'personal email' } })).toBe(false);
    const item = { requestId: 'pending_1', period: '2026-10', reservationNanoGbp: '100', dispatched: true };
    expect(validStatus({ ...baseline, availability: 'attention_required', budget: { ...baseline.budget, pending: [item], requiresReconciliation: true } })).toBe(true);
    expect(validStatus({ ...baseline, budget: { ...baseline.budget, pending: [item, item] } })).toBe(false);
    expect(validStatus({ ...baseline, budget: { ...baseline.budget, pending: [{ ...item, text: 'secret' }] } })).toBe(false);
  });
  it('measures UTF-8 rather than UTF-16 without changing user input', () => {
    expect(validMessage('a'.repeat(32768))).toBe(true); expect(validMessage('a'.repeat(32769))).toBe(false);
    expect(validMessage('é'.repeat(16384))).toBe(true); expect(validMessage('é'.repeat(16385))).toBe(false);
    expect(validMessage('😀'.repeat(8192))).toBe(true); expect(validMessage('😀'.repeat(8193))).toBe(false);
    expect(validMessage('\ud800')).toBe(false); expect(validMessage('\udc00')).toBe(false);
    expect(validMessage('   ')).toBe(false); expect(validMessage('x\0y')).toBe(false);
  });
  it('validates request-bound stream envelopes without approval or tools', () => {
    expect(validStreamEvent({ requestId: 'r', conversationId: 'c', type: 'content', text: '<script>inert</script>' })).toBe(true);
    for (const status of ['completed','failed','cancelled','truncated','refused']) expect(validStreamEvent({ requestId: 'r', conversationId: 'c', type: 'terminal', status })).toBe(true);
    expect(validStreamEvent({ requestId: 'r', conversationId: 'c', type: 'terminal', status: 'verified' })).toBe(false);
    expect(validStreamEvent({ requestId: 'r', conversationId: 'c', type: 'content', text: 'ok', approved: true })).toBe(false);
    expect(validStreamEvent({ requestId: '', conversationId: 'c', type: 'content', text: 'ok' })).toBe(false);
    expect(validStreamEvent({ requestId: 'r', conversationId: 'c', type: 'terminal', status: 'failed', errorCode: '<script>private body</script>' })).toBe(false);
  });
  it('treats restored transcripts as bounded display metadata, never authority', () => {
    const item = saved('c', '<img src=x onerror=evil>');
    expect(validConversation(item)).toBe(true);
    expect(validConversation({ ...item, modelId: 'legacy' })).toBe(false);
    expect(validConversation({ ...item, entries: [{ ...item.entries[0], state: 'streaming' }] })).toBe(false);
    expect(validConversation({ ...item, entries: [{ ...item.entries[0], approved: true }] })).toBe(false);
    expect(validConversation({ ...item, entries: [{ ...item.entries[0], content: 'a'.repeat(524289) }] })).toBe(false);
    expect(validConversation({ ...item, entries: [item.entries[0], item.entries[0]] })).toBe(false);
    const summary = { id: 'c', title: 'Example', updatedAt: 1, entryCount: 2 };
    expect(validSummaries([summary])).toBe(true); expect(validSummaries([summary, summary])).toBe(false);
    expect(validSummaries([{ ...summary, entries: item.entries }])).toBe(false);
  });
  it('uses decimal integer arithmetic for display and charge proposals', () => {
    expect(formatGbp('89000000000')).toBe('£89.00'); expect(formatGbp('10999999')).toBe('£0.01');
    expect(formatGbp('1e9')).toBe('Unavailable'); expect(parseGbp('0.000000001')).toBe('1');
    expect(parseGbp('12.345678901')).toBe('12345678901'); expect(parseGbp('89')).toBe('89000000000');
    for (const bad of ['89.000000001','90','-1','01','1e2','0.0000000001','NaN','']) expect(parseGbp(bad)).toBeNull();
  });
  it('matches the host 256-entry display ceiling across interrupted and failed restarts', () => {
    const item = saved('c', 'Bounded failed attempts');
    const entries = Array.from({ length: 257 }, (_, index) => ({ id: `entry_${index}`, role: index % 2 ? 'assistant' : 'user', content: 'Synthetic fixture', requestId: `request_${Math.floor(index / 2)}`, state: index % 2 ? 'failed' : 'completed' }));
    for (const entryCount of [129, 256]) {
      expect(validConversation({ ...item, entries: entries.slice(0, entryCount) })).toBe(true);
      expect(validSummaries([{ id: item.id, title: item.title, updatedAt: item.updatedAt, entryCount }])).toBe(true);
    }
    expect(validConversation({ ...item, entries })).toBe(false);
    expect(validSummaries([{ id: item.id, title: item.title, updatedAt: item.updatedAt, entryCount: 257 }])).toBe(false);
  });
});
