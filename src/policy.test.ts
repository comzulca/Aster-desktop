import { describe, expect, it } from 'vitest';
import { formatGbp, parseGbp, validAttachmentSummaries, validConversation, validMessage, validStatus, validStreamEvent, validSummaries } from './policy';
import { baseline, saved } from './test-fixtures';

describe('protocol 4 renderer validation (T-055/T-061/T-080)', () => {
  it('admits exactly the reviewed fixed model and false capabilities', () => {
    expect(validStatus(baseline)).toBe(true);
    for (const version of [1, 2, 3, 5]) expect(validStatus({ ...baseline, protocolVersion: version })).toBe(false);
    expect(validStatus({ ...baseline, policyVersion: 2 })).toBe(false);
    expect(validStatus({ ...baseline, approved: true })).toBe(false);
    for (const model of ['meta/muse-glimmer-30b','nvidia/nemotron-3.5-lightning-30b-a3b','muse-spark-1.3-standard']) expect(validStatus({ ...baseline, models: [{ id: model, label: 'Unsupported' }] })).toBe(false);
    for (const key of ['projectRead','proposals','applyNewFile','applyExistingFile']) expect(validStatus({ ...baseline, capabilities: { ...baseline.capabilities, [key]: true } })).toBe(false);
    expect(validStatus({ ...baseline, capabilities: { ...baseline.capabilities, attachmentImport: true } })).toBe(true);
    expect(validStatus({ ...baseline, capabilities: { ...baseline.capabilities, attachmentImport: 'yes' } })).toBe(false);
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
  it('accepts inert bounded attachment metadata only, with closed kinds and page counts', () => {
    const text = { id: 'a'.repeat(32), kind: 'text', label: 'Eligible example.txt', sourceBytes: 80, processedBytes: 80, pageCount: 0 };
    expect(validAttachmentSummaries([])).toBe(true); expect(validAttachmentSummaries([text])).toBe(true);
    expect(validAttachmentSummaries([{ ...text, kind: 'image', pageCount: 1 }])).toBe(true);
    for (const pageCount of [1, 20]) expect(validAttachmentSummaries([{ ...text, kind: 'pdf', pageCount }])).toBe(true);
    for (const pageCount of [0, 21, 1.5]) expect(validAttachmentSummaries([{ ...text, kind: 'pdf', pageCount }])).toBe(false);
    expect(validAttachmentSummaries([{ ...text, kind: 'image', pageCount: 2 }])).toBe(false);
    for (const key of ['path', 'url', 'bytes', 'approved', 'base64']) expect(validAttachmentSummaries([{ ...text, [key]: 'untrusted' }])).toBe(false);
    for (const label of ['../secret.txt', 'C:\\Users\\Owner\\private.txt', 'x\n.txt', 'x\u202e.txt', 'x\u200b.txt', 'é'.repeat(61), '\ud800', '']) expect(validAttachmentSummaries([{ ...text, label }])).toBe(false);
    for (const id of ['../x', 'A'.repeat(32), 'a'.repeat(33)]) expect(validAttachmentSummaries([{ ...text, id }])).toBe(false);
    for (const sourceBytes of [0, -1, 0.5, Infinity, Number.MAX_SAFE_INTEGER + 1]) expect(validAttachmentSummaries([{ ...text, sourceBytes }])).toBe(false);
    expect(validAttachmentSummaries([text, text])).toBe(false);
    expect(validAttachmentSummaries(Array.from({ length: 5 }, (_, index) => ({ ...text, id: String(index).repeat(32) })))).toBe(false);
  });
  it('rejects excessive per-file, aggregate and total-PDF-page metadata without treating renderer checks as native enforcement', () => {
    const pdf = { id: 'a'.repeat(32), kind: 'pdf', label: 'Eligible example.pdf', sourceBytes: 8 * 1024 * 1024, processedBytes: 4 * 1024 * 1024, pageCount: 10 };
    const second = { ...pdf, id: 'b'.repeat(32) };
    expect(validAttachmentSummaries([pdf, second])).toBe(true);
    expect(validAttachmentSummaries([{ ...pdf, sourceBytes: pdf.sourceBytes + 1 }])).toBe(false);
    expect(validAttachmentSummaries([{ ...pdf, processedBytes: pdf.processedBytes + 1 }])).toBe(false);
    expect(validAttachmentSummaries([pdf, second, { ...pdf, id: 'c'.repeat(32), sourceBytes: 1, processedBytes: 1, pageCount: 1 }])).toBe(false);
    expect(validAttachmentSummaries([{ ...pdf, sourceBytes: 1, processedBytes: 1, pageCount: 20 }, { ...second, sourceBytes: 1, processedBytes: 1, pageCount: 1 }])).toBe(false);
    expect(validAttachmentSummaries([{ ...pdf, kind: 'text', pageCount: 0, processedBytes: 128 * 1024 }])).toBe(true);
    expect(validAttachmentSummaries([{ ...pdf, kind: 'text', pageCount: 0, processedBytes: 128 * 1024 + 1 }])).toBe(false);
  });
  it('admits optional display-only processed references on user entries and rejects originals, grants or assistant attachments', () => {
    const item = saved('c', 'Synthetic document fixture');
    const attachment = { id: 'a'.repeat(32), kind: 'pdf', label: 'Synthetic.pdf', sourceBytes: 80, processedBytes: 60, pageCount: 2 };
    const user = { ...item.entries[0], processedAttachments: [attachment] };
    expect(validConversation({ ...item, entries: [user, item.entries[1]] })).toBe(true);
    expect(validConversation({ ...item, entries: [{ ...user, processedAttachments: [] }, item.entries[1]] })).toBe(true);
    expect(validConversation({ ...item, entries: [item.entries[0], { ...item.entries[1], processedAttachments: [attachment] }] })).toBe(false);
    for (const field of ['originalBytes', 'filePath', 'approved', 'grant', 'fileData']) {
      expect(validConversation({ ...item, entries: [{ ...user, [field]: 'forged' }] })).toBe(false);
      expect(validConversation({ ...item, entries: [{ ...user, processedAttachments: [{ ...attachment, [field]: 'forged' }] }] })).toBe(false);
    }
    for (const value of [null, 'original.pdf', { id: attachment.id }]) expect(validConversation({ ...item, entries: [{ ...user, processedAttachments: value }] })).toBe(false);
  });
});
