import { useEffect, useRef, useState } from 'react';
import type { Bridge, CustomizationDraft, DataClass, GenerationBinding, GenerationFinished, PackageKind, StreamEvent } from './types';
import { utf8Bytes, validStatus, validStreamEvent } from './policy';
import { scalarCount } from './personalization-ui';
import { MAX_GENERATED_BYTES, parseGeneratedDraft, validGenerationDescription, validGenerationReceipt } from './generation-ui';

interface Generation {
  requestId: string; kind: PackageKind; conversationId: string | null;
  text: string; queued: StreamEvent[]; queuedBytes: number;
  dispatched: boolean; unsubscribe: (() => void) | null;
}
export function GenerateCustomization({ bridge, kind, locked, onDraft, onClose, onBusyChange, onBound, onFinished }: { bridge: Bridge; kind: PackageKind; locked: boolean; onDraft: (draft: CustomizationDraft) => void; onClose: () => void; onBusyChange: (busy: boolean) => void; onBound?: (binding: GenerationBinding) => void; onFinished?: (result: GenerationFinished) => void }) {
  const [description, setDescription] = useState('');
  const [classification, setClassification] = useState<DataClass | ''>('');
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const mounted = useRef(false);
  const current = useRef<Generation | null>(null);
  function busyState(value: boolean) { if (mounted.current) { setBusy(value); onBusyChange(value); } }
  function finished(session: Generation, terminalObserved: boolean) { onFinished?.({ requestId: session.requestId, conversationId: session.conversationId, terminalObserved }); }
  function finish(session: Generation, message: string, terminalObserved = false, notify = true) {
    if (current.current !== session) return;
    current.current = null; session.unsubscribe?.(); session.unsubscribe = null; session.queued = []; session.text = '';
    if (mounted.current) { setNotice(message); busyState(false); }
    if (notify) finished(session, terminalObserved);
  }
  function invalidate(message: string) {
    const session = current.current; if (!session) return;
    const dispatched = session.dispatched; finish(session, message, false, false);
    if (dispatched) void bridge.cancelMessage(session.requestId).catch(() => {}).finally(() => finished(session, false));
    else finished(session, false);
  }
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; const session = current.current; current.current = null; if (session) { session.unsubscribe?.(); session.queued = []; session.text = ''; if (session.dispatched) void bridge.cancelMessage(session.requestId).catch(() => {}).finally(() => finished(session, false)); else finished(session, false); } };
  }, [bridge]);
  function observe(session: Generation, event: unknown) {
    if (current.current !== session || !mounted.current || typeof event !== 'object' || event === null || (event as Record<string, unknown>).requestId !== session.requestId) return;
    if (session.conversationId !== null && (event as Record<string, unknown>).conversationId !== session.conversationId) return;
    if (!validStreamEvent(event)) { invalidate('The matching generation stream violated its contract. No draft was accepted; stopping was requested and nothing was retried.'); return; }
    if (session.conversationId === null) {
      const bytes = event.type === 'content' ? utf8Bytes(event.text!) : 0;
      if (session.queuedBytes + bytes > MAX_GENERATED_BYTES || session.queued.length >= 1024) { invalidate('The generated response exceeded the local observation limit. It was not turned into a package; stopping was requested and any uncertain spending remains reserved.'); return; }
      session.queued.push(event); session.queuedBytes += bytes; return;
    }
    if (event.conversationId !== session.conversationId) return;
    if (event.type === 'content') {
      if (utf8Bytes(session.text) + utf8Bytes(event.text!) > MAX_GENERATED_BYTES) { invalidate('The generated response exceeded 64 KiB. No partial package was created; stopping was requested and any uncertain spending remains reserved.'); return; }
      session.text += event.text!; return;
    }
    if (event.status !== 'completed' || event.errorCode) { finish(session, 'Generation did not complete successfully. No package draft was created or saved; review any unresolved spending in Settings. Nothing was retried.', true); return; }
    const draft = parseGeneratedDraft(session.text, session.kind);
    if (!draft) { finish(session, 'The completed response was not one valid bounded package of the requested kind. No package was created or saved; nothing was retried.', true); return; }
    finish(session, '', true); onDraft(draft);
  }
  async function start() {
    if (locked || current.current || !bridge.generateCustomization || !validGenerationDescription(description) || !classification) return;
    const session: Generation = { requestId: crypto.randomUUID().replaceAll('-', ''), kind, conversationId: null, text: '', queued: [], queuedBytes: 0, dispatched: false, unsubscribe: null };
    current.current = session; setNotice('Requesting an exact native disclosure review. Generation may incur a separately admitted charge.'); busyState(true);
    try {
      const status: unknown = await bridge.getStatus();
      if (!mounted.current || current.current !== session) return;
      if (!validStatus(status) || status.credentialValidation !== 'valid') { finish(session, 'Configure and validate a Meta API key in Providers before generating a new conversation. No generation or authentication probe was sent.'); return; }
      if (status.availability !== 'ready') { finish(session, 'The native provider admission status is not ready. Review Contributor terms, spending and active requests in Settings; no generation was sent.'); return; }
      const unsubscribe = await bridge.subscribe(event => observe(session, event));
      if (!mounted.current || current.current !== session) { unsubscribe(); return; }
      session.unsubscribe = unsubscribe; session.dispatched = true;
      const receipt: unknown = await bridge.generateCustomization({ requestId: session.requestId, kind: session.kind, description, dataClass: classification });
      if (!mounted.current || current.current !== session) return;
      if (!validGenerationReceipt(receipt, session.requestId)) { invalidate('The generation receipt could not be verified. No draft was accepted; stopping was requested and any uncertain spending remains reserved.'); return; }
      session.conversationId = receipt.conversationId;
      onBound?.({ requestId: session.requestId, conversationId: session.conversationId });
      const queued = session.queued; session.queued = []; session.queuedBytes = 0;
      setNotice('Waiting for the complete request-bound response. Saving and enabling remain separate.');
      for (const event of queued) observe(session, event);
    } catch { if (mounted.current && current.current === session) invalidate('Generation was denied or could not be verified. No draft was accepted; nothing was retried. Check native review and any unresolved spending in Settings.'); }
  }
  return <form className="package-editor generation-editor" onSubmit={event => { event.preventDefault(); void start(); }}>
    <h4>Generate an inert {kind} draft with Muse Spark</h4>
    <p>Use only synthetic or Contributor-eligible public descriptions. The complete native-authored prompt is reviewed separately and uses the existing terms, authentication and spending policy. No Personalization, GitHub content, existing package or previous conversation is added.</p>
    <p>A fresh ordinary local conversation will retain this description, the generation prompt and the response. Generation is inference, not installation or execution. Only a completed valid response opens an unsaved editor; review it and choose Save separately.</p>
    <label>Description<textarea aria-label="Generation description" rows={5} maxLength={8000} disabled={busy || locked} value={description} onChange={event => setDescription(event.target.value)}/></label>
    <p>{scalarCount(description)} / 4,000 characters · {utf8Bytes(description)} / 16,000 UTF-8 bytes</p>
    <label>Content classification<select aria-label="Generation content classification" value={classification} disabled={busy || locked} onChange={event => setClassification(event.target.value as DataClass | '')}><option value="">Choose a classification</option><option value="synthetic">Synthetic</option><option value="public_eligible">Eligible public</option></select></label>
    <div className="settings-actions"><button type="button" className="secondary-action" onClick={() => { invalidate(''); onClose(); }}>Close generation editor</button>{busy && <button type="button" className="secondary-action" onClick={() => invalidate('Generation stopped in this interface; native cancellation was requested. Late results cannot create a draft. Any uncertain spending remains reserved.')}>Stop generation</button>}<button type="submit" className="primary-action" disabled={busy || locked || !bridge.generateCustomization || !validGenerationDescription(description) || !classification}>Review and generate in Windows</button></div>
    {notice && <p role="status">{notice}</p>}
  </form>;
}
