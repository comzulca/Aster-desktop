import { useCallback, useEffect, useId, useRef, useState, type CSSProperties, type KeyboardEvent, type ReactNode } from 'react';
import { nativeBridge } from './bridge';
import { formatGbp, parseGbp, MAX_CONVERSATION_BYTES, MAX_MESSAGE_BYTES, MODEL_LABEL, REASONING_EFFORTS, utf8Bytes, validAttachmentSummaries, validConversation, validMessage, validStatus, validStreamEvent, validSummaries } from './policy';
import { DEFAULT_MODEL, type AppStatus, type AttachmentSummary, type Bridge, type BudgetIntent, type Conversation, type ConversationEntry, type ConversationSummary, type DataClass, type GenerationBinding, type GenerationFinished, type ReasoningEffort, type RendererTracePhase, type WindowAction } from './types';
import wordmark from './assets/aster-wordmark.png';
import readyArtwork from './assets/aster-ready.png';
import { DraftMemory, editSelection, findMatches, helpTitles, menuGroups, shortcutAction, type HelpPage, type LocalAction, type MenuName } from './local-ui';
import packageInfo from '../package.json';
import { Personalization } from './Personalization';
import { Customization } from './Customization';
import { RenderingDiagnostics } from './RenderingDiagnostics';

type DisplayEntry = Omit<ConversationEntry, 'state'> & { state: ConversationEntry['state'] | 'streaming' };
const uuid = () => crypto.randomUUID();
const availabilityText: Record<AppStatus['availability'], string> = {
  needs_terms: 'Review Contributor terms in Settings before sending.',
  needs_credential: 'Set up Meta in Settings before sending.',
  needs_budget: 'Configure the spending policy in Settings before sending.',
  attention_required: 'Attention required. Check Settings; sending is unavailable.',
  ready: 'Ready for separately reviewed requests.',
  busy: 'A request is in progress.',
  unsupported: 'The native Windows host is unavailable.',
};
const credentialText: Record<AppStatus['credentialValidation'], string> = {
  missing: 'Configure and validate a Meta API key in Settings before starting a conversation.',
  unverified: 'Validate the configured Meta API key in Settings before starting a conversation.',
  valid: 'Authentication verified for the configured key; billing and capacity are separate.',
  invalid: 'Meta did not accept this key. Replace it in Settings before starting a conversation.',
  unavailable: 'Key validation is unavailable. Review Settings; no key has been labelled invalid.',
};
const errorText: Record<string, string> = {
  invalid_input: 'The request is invalid. Check the message and its classification.',
  prohibited_content: 'This content cannot be sent under the Contributor policy.',
  data_denied: 'This content cannot be sent under the Contributor policy.',
  admission_denied: 'The content was not admitted. Nothing was retried.',
  terms_required: 'Review the current Contributor terms in Settings.',
  account_ineligible: 'Account eligibility has not been confirmed.',
  budget_unconfigured: 'Configure the spending policy in Settings.',
  budget_expired: 'The spending configuration has expired. Review it in Settings.',
  budget_exceeded: 'The request would exceed the £89 admission ceiling.',
  reconciliation_required: 'Spending is unresolved. Sending remains blocked.',
  busy: 'Another request or native review is in progress.',
  history_unavailable: 'Saved conversations are unavailable. Nothing was overwritten.',
  history_limit: 'The saved-history capacity has been reached. No conversation was discarded.',
  session_limit: 'This process has reached its request limit. Saved history and spending are retained.',
  context_limit: 'This conversation has reached its context limit. Start a new conversation.',
  credential_missing: 'Set up the Meta credential through the native Windows dialogue.',
  credential_unverified: 'Validate the configured Meta API key in Settings before continuing.',
  credential_invalid: 'Meta did not accept this key. Replace it through the native Windows dialogue.',
  review_denied: 'The native disclosure review was declined. Nothing was sent.',
  cancelled: 'The request was stopped. Any unresolved charge remains reserved.',
  refused: 'The response was refused and will not be reused as context.',
  truncated: 'The response is incomplete and will not be reused as context.',
  http_401: 'Meta did not accept the credential. Check the native configuration.',
  http_403: 'Meta denied access to this account or model.',
  http_429: 'Meta rate-limited this request. It was not retried.',
  host_unavailable: 'Open the native Windows app to use this feature.',
  incompatible_host: 'The app and native host versions are incompatible. Use matching components; sending is disabled.',
  unsupported_platform: 'This feature requires the native Windows app. Sending is unavailable on this platform.',
  windows_only: 'This feature requires the native Windows app.',
  caller_denied: 'The native host refused this caller. No privilege or permission was granted.',
  state_unavailable: 'The native request state is unavailable. Sending is blocked; saved records are not reset.',
  clock_unavailable: 'The host cannot validate the current time. Check the Windows date and time; financial records will not be reset.',
  model_denied: 'This model is not supported. Only Muse Spark 1.3 Contributor is admitted.',
  reasoning_denied: 'Choose Minimal, Low, Medium, High or Extra high reasoning. Other levels are not admitted.',
  request_replayed: 'This request identifier has already been used. Aster will not replay it.',
  contributor_terms_required: 'Review the current Contributor terms and account eligibility in Settings before sending.',
  data_class_denied: 'Classify the complete message and attachments as synthetic or eligible public content. Other classifications are not admitted.',
  data_admission_denied: 'The message or prior conversation failed Contributor data admission. Remove prohibited content or start a new eligible conversation; confirmation cannot override this denial.',
  budget_attention_required: 'The spending policy needs attention. Check Settings for expiry, pending charges, capacity or reconciliation; no financial record was reset.',
  disclosure_denied: 'The native disclosure review was declined. Nothing was sent.',
  native_dialog_busy: 'Another native review is open. Finish or close it before starting this action.',
  native_owner_unavailable: 'The native app window could not be authenticated for review. This action remains unavailable.',
  native_review_unavailable: 'The complete request could not be displayed in the trusted native review. It was not approved.',
  native_review_expired: 'The native review expired. The old approval cannot be reused.',
  credential_unavailable: 'The native Windows credential dialogue is unavailable. No key is accepted in this interface.',
  credential_rejected: 'The Meta credential format was not accepted. Check it only in the native Windows dialogue; do not paste it here.',
  credential_error: 'Windows could not validate the credential operation. Check the native dialogue; no key is exposed here.',
  invalid_reconciliation: 'The reconciliation proposal is invalid. Enter the final statement-confirmed GBP charge for one pending request.',
  request_not_pending: 'This request no longer has a pending reservation. Refresh Settings; no balance reset is available.',
  private_store_unavailable: 'The protected local store is unavailable. Saved history and spending records will not be reset or repaired automatically.',
  private_store_denied: 'A local validation rule denied this operation. Review Settings; no financial record was reset.',
  private_store_conflict: 'The protected local store is busy or changed. Finish the other Aster instance or seek review; existing records remain untouched.',
  private_store_corrupt: 'The protected local store failed its integrity check. Sending is blocked; existing records will not be overwritten or reset.',
  private_store_capacity: 'The protected local store has reached its capacity. Sending or saving is unavailable; removing history does not reclaim encrypted recovery records.',
  private_store_io: 'The protected local record could not be verified on disk. Check available storage; uncertain spending remains reserved.',
  history_attention_required: 'The final conversation record could not be verified. Saved history needs review before continuing; spending is handled independently.',
  invalid_request: 'The request does not match the fixed provider contract. Check message size and supported settings; nothing was retried.',
  provider_contract_mismatch: 'Meta returned an unexpected response format. The response may be incomplete; nothing was retried and spending may require reconciliation.',
  provider_usage_invalid: 'Meta usage could not be validated. Check the pending charge in Settings; no zero-cost result is assumed.',
  meta_authentication_failed: 'Meta did not accept the credential. Check the key through the native Windows settings; no retry occurred.',
  meta_billing_unavailable: 'Meta billing is unavailable for this account. Check the account separately; any pending charge remains reserved.',
  meta_access_denied: 'Meta denied access. Check the account, territory and direct-API eligibility; no alternative model was selected.',
  meta_model_unavailable: 'Meta did not make Muse Spark 1.3 Contributor available to this account. No fallback or retry occurred.',
  meta_rate_limited: 'Meta rate-limited the request. No automatic retry occurred; check any pending charge in Settings.',
  meta_unavailable: 'Meta is temporarily unavailable. No automatic retry occurred; uncertain spending remains reserved.',
  network_error: 'The network request failed. The response may be incomplete; check pending spending before trying again.',
  request_timeout: 'The request timed out. It was not retried; any uncertain charge remains reserved.',
  response_limit: 'The response exceeded a configured limit. It may be incomplete and is not reused as context; check any pending charge.',
  incomplete_answer: 'The response ended without valid completion. It is not reused as context; spending may require reconciliation.',
  provider_tools_denied: 'The provider attempted an unsupported tool operation. No tool was executed; the request was stopped without retry.',
  attachments_unavailable: 'Attachment import is unavailable in this host. No file was read or sent.',
  attachment_unavailable: 'Attachment processing is unavailable. Temporary attachment identifiers were discarded; nothing was sent.',
  attachment_capacity: 'The selected files or processed history exceed an Aster limit. No content was silently truncated; nothing was sent.',
  attachment_convert_to_pdf: 'DOCX, XLSX and PPTX require safe external conversion to PDF. Convert the complete document with your own trusted tool, check every page, sheet, slide and print range, then attach the PDF. Aster does not run or require an Office converter.',
  attachment_denied: 'The selected file was not admitted. Check its format, content and the attachment limits; nothing was sent.',
  attachment_limit: 'The attachment exceeds an Aster limit. Use a smaller complete file; no content was silently truncated.',
  attachment_expired: 'The temporary attachment has expired or was removed. Select the file again before sending.',
  attachment_conflict: 'The selected file changed or could not be read safely. No file was sent.',
  attachment_invalid: 'The host returned invalid attachment metadata. Temporary attachment identifiers were discarded; nothing was sent.',
  processed_attachment_denied: 'The processed attachment is unavailable or no longer belongs to this saved request. Nothing was sent and no original file was reconstructed.',
  office_conversion_required: 'DOCX, XLSX and PPTX are not imported directly. Convert the complete document to PDF with your own trusted tool, check every page or sheet, then attach the PDF. Aster does not run an Office converter.',
};
function safeError(error: unknown, fallback: string): string {
  const code = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  return errorText[code] ?? fallback; // Never render provider bodies or arbitrary host error text.
}
function Icon({ name, size = 20 }: { name: 'new' | 'search' | 'settings' | 'send' | 'stop' | 'close' | 'menu' | 'trash' | 'plus'; size?: number }) {
  const paths: Record<typeof name, ReactNode> = {
    new: <><path d="M13 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-8"/><path d="m10 14 1-4L20 1l3 3-9 9-4 1"/></>,
    search: <><circle cx="10.5" cy="10.5" r="7"/><path d="m16 16 5 5"/></>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="m9 3 6 0 1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z"/></>,
    send: <path d="M12 20V4m-7 7 7-7 7 7"/>,
    stop: <rect x="6" y="6" width="12" height="12" rx="1" fill="currentColor" stroke="none"/>,
    close: <path d="m5 5 14 14M19 5 5 19"/>,
    menu: <path d="M3 6h18M3 12h18M3 18h18"/>,
    trash: <><path d="M4 6h16M9 3h6M6 6l1 15h10l1-15M10 10v7M14 10v7"/></>,
    plus: <path d="M12 4v16M4 12h16"/>,
  };
  return <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">{paths[name]}</svg>;
}
function UnionJack() {
  return <svg aria-hidden="true" width="32" height="20" viewBox="0 0 60 36"><defs><clipPath id="flag-boundary"><rect width="60" height="36"/></clipPath></defs><g clipPath="url(#flag-boundary)"><rect width="60" height="36" fill="#012169"/><path d="m0 0 60 36m0-36L0 36" stroke="#fff" strokeWidth="8"/><path d="m0 0 60 36m0-36L0 36" stroke="#c8102e" strokeWidth="3"/><path d="M30 0v36M0 18h60" stroke="#fff" strokeWidth="12"/><path d="M30 0v36M0 18h60" stroke="#c8102e" strokeWidth="7"/></g></svg>;
}

const effortLabel = (effort: ReasoningEffort) => effort === 'xhigh' ? 'Extra high' : effort[0].toUpperCase() + effort.slice(1);
const classLabel = (classification: DataClass | '') => classification === 'synthetic' ? 'Synthetic' : classification === 'public_eligible' ? 'Eligible public' : 'Not classified';

// One presentation popup, two independent policy values. This never approves
// a disclosure and never changes classification when an effort is selected.
function EffortControl({ effort, classification, disabled, onEffort, onClassify }: {
  effort: ReasoningEffort; classification: DataClass | ''; disabled: boolean;
  onEffort(value: ReasoningEffort): void; onClassify(value: DataClass): void;
}) {
  const [open, setOpen] = useState(false);
  const [child, setChild] = useState(false);
  const region = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const classify = useRef<HTMLButtonElement>(null);
  const focusAfterRender = useRef<'first' | 'last' | 'selected' | 'child' | 'classify' | null>(null);
  const identifier = useId();
  function dismiss(returnFocus = false) {
    setOpen(false); setChild(false); focusAfterRender.current = null;
    if (returnFocus && !disabled) trigger.current?.focus();
  }
  function reveal(focus: 'first' | 'last' | 'selected' = 'selected') {
    if (disabled) return;
    focusAfterRender.current = focus; setOpen(true); setChild(false);
  }
  function revealChild() {
    if (disabled) return;
    focusAfterRender.current = 'child'; setChild(true);
  }
  useEffect(() => {
    if (disabled) { setOpen(false); setChild(false); focusAfterRender.current = null; }
  }, [disabled]);
  useEffect(() => {
    const pending = focusAfterRender.current; focusAfterRender.current = null;
    if (!open || disabled || !pending) return;
    if (pending === 'classify') { classify.current?.focus(); return; }
    const items = [...(region.current?.querySelectorAll<HTMLButtonElement>(pending === 'child' ? '[data-class-item]' : '[data-effort-item]') ?? [])];
    (pending === 'last' ? items.at(-1) : pending === 'selected' ? items.find(item => item.getAttribute('aria-checked') === 'true') ?? items[0] : items[0])?.focus();
  }, [open, child, disabled]);
  useEffect(() => {
    if (!open) return;
    function outside(event: MouseEvent) { if (!region.current?.contains(event.target as Node)) { setOpen(false); setChild(false); } }
    window.addEventListener('mousedown', outside);
    return () => window.removeEventListener('mousedown', outside);
  }, [open]);
  function keys(event: KeyboardEvent<HTMLElement>, inChild: boolean) {
    if (disabled) { event.preventDefault(); dismiss(); return; }
    if (event.key === 'Tab') { dismiss(); return; }
    if (event.key === 'Escape') {
      event.preventDefault(); event.stopPropagation();
      if (inChild) { focusAfterRender.current = 'classify'; setChild(false); } else dismiss(true);
      return;
    }
    if (inChild && event.key === 'ArrowLeft') { event.preventDefault(); event.stopPropagation(); focusAfterRender.current = 'classify'; setChild(false); return; }
    if (!inChild && event.key === 'ArrowRight' && document.activeElement === classify.current) { event.preventDefault(); revealChild(); return; }
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault(); event.stopPropagation();
    const items = [...(region.current?.querySelectorAll<HTMLButtonElement>(inChild ? '[data-class-item]' : '[data-effort-item]') ?? [])];
    const index = items.indexOf(document.activeElement as HTMLButtonElement);
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length;
    items[next]?.focus(); items[next]?.scrollIntoView?.({ block: 'nearest' });
  }
  return <div className="effort-control" ref={region}>
    <button ref={trigger} className="effort-trigger" aria-label="Reasoning effort" aria-describedby={`${identifier}-classification`} aria-haspopup="menu" aria-expanded={open} aria-controls={`${identifier}-effort`} disabled={disabled}
      onClick={() => open ? dismiss(true) : reveal()} onKeyDown={event => {
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') { event.preventDefault(); reveal(event.key === 'ArrowDown' ? 'first' : 'last'); }
        else if (event.key === 'Escape' && open) { event.preventDefault(); dismiss(true); }
      }}><span>{effortLabel(effort)}</span><span aria-hidden="true">⌄</span></button>
    <span id={`${identifier}-classification`} className="visually-hidden" aria-label="Content classification" role="status">{classLabel(classification)}</span>
    {open && <div className="effort-popup-shell"><div className="effort-popup" id={`${identifier}-effort`} role="menu" aria-label="Reasoning effort options" onKeyDown={event => keys(event, false)}>
      {REASONING_EFFORTS.map(value => <button key={value} type="button" data-effort-item role="menuitemradio" aria-checked={effort === value} disabled={disabled} onClick={() => { if (!disabled) { onEffort(value); dismiss(true); } }}><span>{effortLabel(value)}</span></button>)}
      <div className="menu-separator" role="separator"/>
      <button ref={classify} data-effort-item type="button" role="menuitem" aria-label="Classify content" aria-haspopup="menu" aria-expanded={child} aria-controls={`${identifier}-classification-menu`} disabled={disabled} onClick={revealChild}><span>Classify content</span><span aria-hidden="true">›</span></button>
    </div>
      {child && <div className="classification-popup" id={`${identifier}-classification-menu`} role="menu" aria-label="Content classification options" onKeyDown={event => { event.stopPropagation(); keys(event, true); }}>
        {(['synthetic', 'public_eligible'] as const).map(value => <button key={value} type="button" data-class-item role="menuitemradio" aria-checked={classification === value} disabled={disabled} onClick={() => { if (!disabled) { onClassify(value); dismiss(true); } }}><span>{classLabel(value)}</span></button>)}
      </div>}
    </div>}
  </div>;
}

export function App({ bridge = nativeBridge }: { bridge?: Bridge }) {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [hostState, setHostState] = useState<'checking' | 'ready' | 'unavailable'>('checking');
  const [summaries, setSummaries] = useState<ConversationSummary[]>([]);
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [entries, setEntries] = useState<DisplayEntry[]>([]);
  const [draft, setDraft] = useState('');
  const [dataClass, setDataClass] = useState<DataClass | ''>('');
  const [reasoningEffort, setReasoningEffort] = useState<ReasoningEffort>('medium');
  const [attachments, setAttachments] = useState<AttachmentSummary[]>([]);
  const attachmentState = useRef<{ conversationId: string | null; items: AttachmentSummary[] }>({ conversationId: null, items: [] });
  const shellRef = useRef<HTMLDivElement>(null);
  const shellReadySent = useRef(false);
  const assetsReadyMarked = useRef(false);
  const renderMarkerCount = useRef(0);
  const [activeRequest, setActiveRequest] = useState<string | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [compact, setCompact] = useState(false);
  const [zoom, setZoom] = useState(100);
  const [zoomFeedback, setZoomFeedback] = useState(false);
  const zoomRef = useRef(100);
  const zoomTimeout = useRef<ReturnType<typeof window.setTimeout> | null>(null);
  const [windowState, setWindowState] = useState({ maximised: false, fullscreen: false });
  const [findOpen, setFindOpen] = useState(false);
  const [find, setFind] = useState('');
  const [helpPage, setHelpPage] = useState<HelpPage>('documentation');
  const [searchOpen, setSearchOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [menu, setMenu] = useState<MenuName | null>(null);
  const [dialog, setDialog] = useState<'settings' | 'help' | 'quit' | null>(null);
  const [settingsTab, setSettingsTab] = useState<'providers' | 'personalization' | 'usage' | 'local' | 'customize' | 'about'>('providers');
  const [workspace, setWorkspace] = useState<'chat' | 'customize'>('chat');
  const workspaceRef = useRef<'chat' | 'customize'>('chat');
  const [customizationBlocked, setCustomizationBlocked] = useState(false);
  const customizationBlockedRef = useRef(false);
  const customizationNavigation = useCallback((blocked: boolean) => { customizationBlockedRef.current = blocked; setCustomizationBlocked(blocked); }, []);
  const [profileNavigationBlocked, setProfileNavigationBlocked] = useState(false);
  const profileNavigationBlockedRef = useRef(false);
  const profileNavigation = useCallback((blocked: boolean) => { profileNavigationBlockedRef.current = blocked; setProfileNavigationBlocked(blocked); }, []);
  const workspaceReturn = useRef<HTMLButtonElement>(null);
  const [budgetFields, setBudgetFields] = useState({ rate: '', margin: '', tax: '', payment: '', validUntil: '' });
  const [reconciliationCharges, setReconciliationCharges] = useState<Record<string, string>>({});
  const newAccountId = useRef(uuid().replaceAll('-', ''));
  const activeRef = useRef<string | null>(null);
  const busyRef = useRef(false);
  const statusRef = useRef<AppStatus | null>(null);
  const conversationRef = useRef<string | null>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const dialogStateRef = useRef<'settings' | 'help' | 'quit' | null>(null);
  const dialogOpener = useRef<HTMLElement | null>(null);
  const settingsButton = useRef<HTMLButtonElement>(null);
  const draftRef = useRef<HTMLTextAreaElement>(null);
  const draftValue = useRef('');
  const drafts = useRef(new DraftMemory());
  const localEdits = useRef(new DraftMemory());
  const editTarget = useRef<{ element: HTMLInputElement | HTMLTextAreaElement; start: number; end: number; text: string } | null>(null);
  const copiedSelection = useRef('');
  const findRef = useRef<HTMLInputElement>(null);
  const localActionRef = useRef<(action: LocalAction, opener?: HTMLElement) => void>(() => {});
  const quitRef = useRef<() => void>(() => {});
  const searchRef = useRef<HTMLInputElement>(null);
  const historyRef = useRef<HTMLElement>(null);
  const menuRegion = useRef<HTMLDivElement>(null);
  const terminal = useRef(new Set<string>());
  const eventTerminals = useRef(new Set<string>());
  const generationBindings = useRef(new Map<string, string>());
  const generationTerminals = useRef(new Set<string>());
  const endRef = useRef<HTMLDivElement>(null);
  const locked = busy || Boolean(activeRequest) || Boolean(status?.activeRequest);

  function draftKey() { return conversationRef.current ?? '$unsaved'; }
  function changeDraft(value: string) { drafts.current.write(draftKey(), value); draftValue.current = value; setDraft(value); }
  function preserveDraft() {
    if (!drafts.current.write(draftKey(), draftValue.current)) { setNotice('Reduce the draft to 32,768 UTF-8 bytes before navigating. No text was discarded.'); return false; }
    return true;
  }

  function discardAttachments(revoke = true) {
    const previous = attachmentState.current;
    attachmentState.current = { conversationId: null, items: [] }; setAttachments([]);
    // Revocation is best-effort cleanup only; a failure never restores IDs or
    // turns those IDs into renderer filesystem/disclosure authority.
    if (revoke && previous.conversationId && bridge.removeAttachment) {
      for (const item of previous.items) void bridge.removeAttachment({ conversationId: previous.conversationId, attachmentId: item.id }).catch(() => {});
    }
  }

  function adopt(value: Conversation | null) {
    if (conversationRef.current !== (value?.id ?? null)) discardAttachments();
    conversationRef.current = value?.id ?? null;
    setConversation(value); setEntries(value?.entries ?? []);
    const restored = drafts.current.read(value?.id ?? '$unsaved'); draftValue.current = restored;
    setDraft(restored); setDataClass(''); setNotice(null); setFind('');
  }
  async function refreshStatus() {
    const value: unknown = await bridge.getStatus();
    if (!validStatus(value)) throw new Error('incompatible_host');
    statusRef.current = value; setStatus(value); setHostState('ready');
    return value;
  }
  async function refreshList() {
    const value: unknown = await bridge.listConversations();
    if (!validSummaries(value)) throw new Error('history_unavailable');
    const ordered = [...value].sort((a, b) => b.updatedAt - a.updatedAt || a.id.localeCompare(b.id, 'en-GB'));
    setSummaries(ordered); return ordered;
  }
  async function readConversation(id: string) {
    const value: unknown = await bridge.loadConversation({ conversationId: id });
    if (!validConversation(value) || value.id !== id) throw new Error('history_unavailable');
    return value;
  }
  function isLocked() { return busyRef.current || Boolean(activeRef.current) || Boolean(statusRef.current?.activeRequest); }
  function markBusy(value: boolean) { busyRef.current = value; setBusy(value); }
  function generationBound(binding: GenerationBinding) {
    if (generationBindings.current.size >= 32) generationBindings.current.delete(generationBindings.current.keys().next().value!);
    generationBindings.current.set(binding.requestId, binding.conversationId);
  }
  function generationFinished(result: GenerationFinished) {
    if (result.terminalObserved) {
      generationBindings.current.delete(result.requestId);
      if (generationTerminals.current.has(result.requestId)) return;
      if (generationTerminals.current.size >= 32) generationTerminals.current.delete(generationTerminals.current.values().next().value!);
      generationTerminals.current.add(result.requestId);
    }
    if (activeRef.current) return;
    // Only current local status/history metadata is refreshed. This never
    // adopts generation entries, changes the main draft, or restores imports.
    void Promise.all([refreshStatus(), refreshList()]).catch(() => setNotice('Generation status or local history metadata could not be refreshed. Existing conversation content and drafts were kept.'));
  }
  function markRender(phase: RendererTracePhase) {
    // Lower-trust fixed markers only. Bounded locally and again by the native
    // trace; errors neither delay startup nor change visibility or authority.
    if (!bridge.recordRenderEvent || renderMarkerCount.current >= 32) return;
    renderMarkerCount.current++;
    void bridge.recordRenderEvent({ protocolVersion: 1, phase }).catch(() => {});
  }

  useEffect(() => {
    markRender('mounted');
    const visibility = () => markRender(document.visibilityState === 'hidden' ? 'hidden' : 'visible');
    visibility(); document.addEventListener('visibilitychange', visibility);
    return () => document.removeEventListener('visibilitychange', visibility);
  }, [bridge]);

  useEffect(() => {
    if ((!bridge.shellReady && !bridge.recordRenderEvent) || shellReadySent.current) return;
    let disposed = false; let firstFrame: number | null = null; let secondFrame: number | null = null;
    const cleanups: (() => void)[] = [];
    const images = [...(shellRef.current?.querySelectorAll<HTMLImageElement>('img') ?? [])];
    let outstanding = images.length;
    let assetFailed = false;
    function schedule() {
      if (disposed || outstanding) return;
      if (!assetsReadyMarked.current && !assetFailed) { assetsReadyMarked.current = true; markRender('assets_ready'); }
      if (!bridge.shellReady || firstFrame !== null || shellReadySent.current) return;
      firstFrame = window.requestAnimationFrame(() => {
        firstFrame = null;
        if (disposed) return;
        secondFrame = window.requestAnimationFrame(() => {
          secondFrame = null;
          if (disposed || shellReadySent.current) return;
          // This is a scheduling signal only; host timing/native fallback is
          // independent, and neither frame demonstrates painted pixels.
          shellReadySent.current = true;
          void bridge.shellReady!({ protocolVersion: 1 }).then(receipt => {
            if (!receipt || receipt.accepted !== true || typeof receipt.visible !== 'boolean' || Object.keys(receipt).sort().join('|') !== 'accepted|visible') throw new Error('startup_signal_unavailable');
          }).catch(() => { if (!disposed) setNotice('The startup scheduling signal could not be confirmed. The native fallback remains independent; no request was sent.'); });
        });
      });
    }
    for (const image of images) {
      if (image.complete) { outstanding--; if (image.naturalWidth === 0) assetFailed = true; continue; }
      let settled = false;
      function finish(event?: Event) {
        if (settled || disposed) return; settled = true; outstanding--;
        if (event?.type === 'error') assetFailed = true;
        image.removeEventListener('load', finish); image.removeEventListener('error', finish); schedule();
      }
      image.addEventListener('load', finish); image.addEventListener('error', finish);
      cleanups.push(() => { image.removeEventListener('load', finish); image.removeEventListener('error', finish); });
      // Avoid missing a cached asset that settled while listeners were added.
      if (image.complete) { if (image.naturalWidth === 0) assetFailed = true; finish(); }
    }
    schedule();
    return () => {
      disposed = true; cleanups.forEach(cleanup => cleanup());
      if (firstFrame !== null) window.cancelAnimationFrame(firstFrame);
      if (secondFrame !== null) window.cancelAnimationFrame(secondFrame);
    };
  }, [bridge]);

  useEffect(() => {
    const query = window.matchMedia?.('(max-width: 750px)');
    if (!query) return;
    const changed = () => setCompact(query.matches); changed(); query.addEventListener('change', changed);
    return () => query.removeEventListener('change', changed);
  }, []);
  useEffect(() => {
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    async function initialize() {
      markRender('hydration_started');
      markBusy(true);
      try {
        unsubscribe = await bridge.subscribe(event => {
          if (!validStreamEvent(event)) return;
          if (event.type === 'terminal' && !activeRef.current && generationBindings.current.get(event.requestId) === event.conversationId) {
            generationFinished({ requestId: event.requestId, conversationId: event.conversationId, terminalObserved: true }); return;
          }
          if (event.conversationId !== conversationRef.current || event.requestId !== activeRef.current || terminal.current.has(event.requestId)) return;
          if (event.type === 'content') {
            setEntries(current => current.map(entry => {
              if (entry.role !== 'assistant' || entry.requestId !== event.requestId) return entry;
              const content = entry.content + event.text!;
              if (utf8Bytes(content) > MAX_CONVERSATION_BYTES) {
                setNotice('The response exceeded the display limit. Stop the request; no content was truncated silently.');
                return entry;
              }
              return { ...entry, content };
            }));
            return;
          }
          terminal.current.add(event.requestId);
          eventTerminals.current.add(event.requestId);
          discardAttachments(false);
          activeRef.current = null; setActiveRequest(null); setCancelling(false);
          setEntries(current => current.map(entry => entry.role === 'assistant' && entry.requestId === event.requestId ? { ...entry, state: event.status! } : entry));
          if (event.status !== 'completed' || event.errorCode) setNotice(errorText[event.errorCode ?? event.status!] ?? 'The response did not complete. Nothing was retried; spending may require reconciliation.');
          markBusy(true);
          void Promise.all([refreshStatus(), refreshList(), readConversation(event.conversationId)])
            .then(([, , saved]) => { if (!disposed && conversationRef.current === saved.id) { setConversation(saved); setEntries(saved.entries); } })
            .catch(error => { if (!disposed) setNotice(safeError(error, 'The saved result could not be verified. Check Settings before continuing.')); })
            .finally(() => { if (!disposed) markBusy(false); });
        });
        if (disposed) { unsubscribe(); return; }
        const value = await refreshStatus();
        if (disposed) return;
        if (value.historyAvailable) {
          const list = await refreshList();
          if (list.length) { const saved = await readConversation(list[0].id); if (!disposed) adopt(saved); }
        }
      } catch (error) {
        if (!disposed) { discardAttachments(); statusRef.current = null; setStatus(null); setHostState('unavailable'); setNotice(safeError(error, 'The native host or its saved-history contract is unavailable or incompatible. Privileged actions are disabled.')); }
      }
      finally { if (!disposed) { markRender('hydration_finished'); markBusy(false); } }
    }
    void initialize();
    return () => { disposed = true; unsubscribe?.(); };
  }, [bridge]);
  useEffect(() => {
    let disposed = false; let unsubscribe: (() => void) | undefined;
    void bridge.subscribeClose?.(() => quitRef.current()).then(value => { if (disposed) value(); else unsubscribe = value; }).catch(() => setNotice('Native close notifications are unavailable. Use Quit Aster; no draft was discarded.'));
    return () => { disposed = true; unsubscribe?.(); };
  }, [bridge]);

  useEffect(() => {
    const area = draftRef.current;
    if (area) { area.style.height = 'auto'; area.style.height = `${Math.min(Math.max(area.scrollHeight, 48), 220)}px`; }
  }, [draft]);
  useEffect(() => { endRef.current?.scrollIntoView?.({ block: 'end', behavior: 'auto' }); }, [entries]);
  useEffect(() => {
    dialogStateRef.current = dialog;
    if (dialog) (dialogRef.current?.querySelector<HTMLElement>('[data-default-cancel]') ?? dialogRef.current?.querySelector<HTMLElement>('button:not(:disabled), input:not(:disabled)'))?.focus();
    else if (dialogOpener.current) {
      const previous = dialogOpener.current; dialogOpener.current = null;
      (previous.isConnected && !previous.closest('[hidden]') && !(previous as HTMLButtonElement).disabled ? previous : workspaceRef.current === 'customize' ? workspaceReturn.current : settingsButton.current)?.focus();
    }
  }, [dialog]);
  useEffect(() => {
    function shortcut(event: globalThis.KeyboardEvent) {
      const action = shortcutAction(event);
      if (!action) return;
      // Inputs in the local Settings dialogue keep their own ordinary editing;
      // shortcuts never reach native credential/disclosure dialogue controls.
      if (dialogStateRef.current) {
        const target = event.target;
        const ordinaryEdit = dialogStateRef.current === 'settings' && (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) && !target.disabled && ['undo','redo','cut','copy','paste','select'].includes(action);
        if (!ordinaryEdit) event.preventDefault();
        return;
      }
      if (workspaceRef.current === 'customize') {
        const target = event.target;
        // Ordinary editing remains owned by the visible package fields. Main
        // draft/history shortcuts must never act on mounted-but-hidden chat.
        if ((target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) && !target.disabled && ['undo','redo','cut','copy','paste','select'].includes(action)) return;
      }
      captureEdit(document.activeElement); captureVisibleSelection();
      event.preventDefault();
      localActionRef.current(action);
    }
    function outside(event: MouseEvent) { if (menuRegion.current && !menuRegion.current.contains(event.target as Node)) setMenu(null); }
    function wheel(event: WheelEvent) {
      if (!event.ctrlKey) return;
      // Native/page zoom is never a second owner, even when local zoom is denied.
      event.preventDefault();
      if (event.altKey || event.shiftKey || !Number.isFinite(event.deltaY) || event.deltaY === 0) return;
      localActionRef.current(event.deltaY < 0 ? 'zoomIn' : 'zoomOut');
    }
    window.addEventListener('keydown', shortcut); window.addEventListener('mousedown', outside); window.addEventListener('wheel', wheel, { passive: false });
    return () => {
      window.removeEventListener('keydown', shortcut); window.removeEventListener('mousedown', outside); window.removeEventListener('wheel', wheel);
      if (zoomTimeout.current !== null) { window.clearTimeout(zoomTimeout.current); zoomTimeout.current = null; }
    };
  }, []);

  async function newConversation() {
    if (workspaceRef.current !== 'chat' || dialogStateRef.current || isLocked() || statusRef.current?.credentialValidation !== 'valid' || !statusRef.current.historyAvailable || !preserveDraft()) return;
    markBusy(true); setMenu(null);
    try {
      await materialiseDraft();
      const saved: unknown = await bridge.createConversation();
      if (!validConversation(saved)) throw new Error('history_unavailable');
      adopt(saved); await refreshList(); setSidebarOpen(false); draftRef.current?.focus();
    } catch (error) { setNotice(safeError(error, 'A new conversation could not be created. Existing history is unchanged.')); }
    finally { markBusy(false); requestAnimationFrame(() => { if (!dialogStateRef.current) draftRef.current?.focus(); }); }
  }
  async function chooseConversation(id: string) {
    if (workspaceRef.current !== 'chat' || dialogStateRef.current || isLocked() || !statusRef.current?.historyAvailable || !preserveDraft()) return;
    markBusy(true);
    try { await materialiseDraft(); adopt(await readConversation(id)); setSidebarOpen(false); draftRef.current?.focus(); }
    catch (error) { setNotice(safeError(error, 'The conversation could not be loaded. Existing history is unchanged.')); }
    finally { markBusy(false); requestAnimationFrame(() => { if (!dialogStateRef.current) draftRef.current?.focus(); }); }
  }
  async function materialiseDraft() {
    if (conversationRef.current || !draftValue.current) return;
    // A missing proof must not manufacture a persisted conversation just to
    // switch display history. The unsaved draft remains renderer memory.
    if (statusRef.current?.credentialValidation !== 'valid') return;
    const saved: unknown = await bridge.createConversation();
    if (!validConversation(saved)) throw new Error('history_unavailable');
    if (!drafts.current.write(saved.id, draftValue.current)) throw new Error('history_limit');
    drafts.current.forget('$unsaved'); adopt(saved); await refreshList();
  }
  async function removeConversation() {
    const id = conversationRef.current;
    if (!id || isLocked()) return;
    if (draftValue.current.length) { setNotice('Keep or clear the current unsent draft before removing this conversation. No draft was discarded.'); return; }
    markBusy(true); setMenu(null);
    try {
      const result = await bridge.deleteConversation({ conversationId: id });
      if (result.deleted) {
        drafts.current.forget(id);
        const list = await refreshList(); adopt(list.length ? await readConversation(list[0].id) : null);
        setNotice('Removed from visible history. Encrypted recovery records remain; spending and credentials are unchanged.');
      }
    } catch (error) { setNotice(safeError(error, 'The conversation was not removed. Check the native confirmation.')); }
    finally { markBusy(false); }
  }
  async function nativeAction(action: () => Promise<unknown>, fallback: string) {
    if (isLocked() || !statusRef.current) return;
    markBusy(true); setNotice(null);
    try { await action(); await refreshStatus(); }
    catch (error) { setNotice(safeError(error, fallback)); }
    finally { markBusy(false); }
  }
  async function addAttachments() {
    if (workspaceRef.current !== 'chat' || isLocked() || dialogStateRef.current || statusRef.current?.credentialValidation !== 'valid' || !statusRef.current.historyAvailable || !statusRef.current.capabilities.attachmentImport || !bridge.selectAttachments || !bridge.removeAttachment || !preserveDraft()) return;
    markBusy(true); setMenu(null); setNotice(null);
    try {
      if (!conversationRef.current) {
        const value: unknown = await bridge.createConversation();
        if (!validConversation(value)) throw new Error('history_unavailable');
        if (!drafts.current.write(value.id, draftValue.current)) throw new Error('history_limit');
        drafts.current.forget('$unsaved'); adopt(value); await refreshList();
      }
      const conversationId = conversationRef.current!;
      const replacing = attachmentState.current.items.length > 0;
      // The host replaces the complete batch before opening its selector.
      // Pending/cancelled replacement cannot leave old IDs shown as current.
      discardAttachments(false); setDataClass('');
      const selected: unknown = await bridge.selectAttachments({ conversationId });
      if (!validAttachmentSummaries(selected) || conversationRef.current !== conversationId) throw new Error('attachment_invalid');
      // The native receipt describes the complete current batch, not a
      // renderer-created merge or a grant copied from a different chat.
      attachmentState.current = { conversationId, items: selected };
      setAttachments(selected); setDataClass('');
      if (!selected.length && replacing) setNotice('No replacement attachments were selected. The previous temporary attachments were discarded; your draft is unchanged and nothing was sent.');
    } catch (error) {
      discardAttachments();
      setNotice(safeError(error, 'The files could not be imported safely. Temporary attachment identifiers were discarded; your draft is unchanged and nothing was sent.'));
    } finally { markBusy(false); if (!dialogStateRef.current) draftRef.current?.focus(); }
  }
  async function removeAttachment(id: string) {
    const current = attachmentState.current;
    if (workspaceRef.current !== 'chat' || isLocked() || dialogStateRef.current || !bridge.removeAttachment || current.conversationId !== conversationRef.current || !current.conversationId || !current.items.some(item => item.id === id)) return;
    markBusy(true);
    try {
      const receipt = await bridge.removeAttachment({ conversationId: current.conversationId, attachmentId: id });
      if (!receipt || typeof receipt.removed !== 'boolean' || Object.keys(receipt).join('|') !== 'removed') throw new Error('attachment_invalid');
      if (!receipt.removed) throw new Error('attachment_expired');
      if (attachmentState.current !== current || conversationRef.current !== current.conversationId) return;
      const items = current.items.filter(item => item.id !== id);
      attachmentState.current = { conversationId: current.conversationId, items }; setAttachments(items); setDataClass('');
    } catch (error) { discardAttachments(); setNotice(safeError(error, 'The attachment removal could not be confirmed. Temporary identifiers were discarded; your draft and saved history are unchanged.')); }
    finally { markBusy(false); if (!dialogStateRef.current) draftRef.current?.focus(); }
  }
  async function viewProcessedAttachment(entry: DisplayEntry, attachment: AttachmentSummary, opener: HTMLButtonElement) {
    const conversationId = conversationRef.current;
    if (workspaceRef.current !== 'chat' || !conversationId || isLocked() || dialogStateRef.current || !statusRef.current?.historyAvailable || !bridge.viewProcessedAttachment || entry.role !== 'user' || attachment.kind === 'text' || !entry.processedAttachments?.some(item => item.id === attachment.id)) return;
    markBusy(true); setMenu(null);
    try {
      // Saved metadata asks for a local native view only. It is never copied
      // into attachmentState, sent as current import IDs or used as approval.
      const result = await bridge.viewProcessedAttachment({ conversationId, requestId: entry.requestId, attachmentId: attachment.id });
      if (!result || typeof result.viewed !== 'boolean' || Object.keys(result).join('|') !== 'viewed') throw new Error('processed_attachment_denied');
    } catch (error) { setNotice(safeError(error, 'The processed content could not be displayed locally. Nothing was sent and the original was not restored.')); }
    finally {
      markBusy(false);
      requestAnimationFrame(() => { if (opener.isConnected && !opener.disabled && !dialogStateRef.current) opener.focus(); });
    }
  }
  async function send() {
    const currentStatus = statusRef.current;
    if (workspaceRef.current !== 'chat' || dialogStateRef.current || isLocked() || !currentStatus || currentStatus.credentialValidation !== 'valid' || currentStatus.availability !== 'ready' || !validMessage(draft) || !dataClass || !['synthetic', 'public_eligible'].includes(dataClass) || !REASONING_EFFORTS.includes(reasoningEffort)) return;
    const message = draft;
    const classification = dataClass;
    const selectedAttachments = attachmentState.current;
    const attachmentIds = attachmentState.current.conversationId === conversationRef.current ? attachmentState.current.items.map(item => item.id) : [];
    markBusy(true); setNotice(null);
    let id = conversationRef.current;
    let issuedRequestId: string | null = null;
    try {
      if (!id) {
        const value: unknown = await bridge.createConversation();
        if (!validConversation(value)) throw new Error('history_unavailable');
        id = value.id; conversationRef.current = id; setConversation(value); setEntries(value.entries); await refreshList();
        drafts.current.write(id, message); drafts.current.forget('$unsaved');
      }
      const requestId = uuid().replaceAll('-', '');
      issuedRequestId = requestId;
      activeRef.current = requestId; setActiveRequest(requestId); terminal.current.delete(requestId);
      setEntries(current => [...current, { id: uuid(), role: 'user', content: message, requestId, state: 'completed' }, { id: uuid(), role: 'assistant', content: '', requestId, state: 'streaming' }]);
      changeDraft(''); drafts.current.forget(draftKey());
      const response = await bridge.sendMessage({ conversationId: id, requestId, modelId: DEFAULT_MODEL, message, dataClass: classification, reasoningEffort, attachmentIds });
      if (response.requestId !== requestId || response.accepted !== true) throw new Error('request_denied');
      if (attachmentState.current === selectedAttachments) discardAttachments(false);
    } catch (error) {
      if (issuedRequestId && eventTerminals.current.has(issuedRequestId)) return; // The terminal event won this race.
      discardAttachments();
      // A declined native review has no saved intent. Keep the exact unsent
      // draft, but never replace a newer draft after a terminal-winning race.
      if (conversationRef.current === id && (!issuedRequestId || activeRef.current === issuedRequestId)) {
        changeDraft(message); setDataClass(classification);
      }
      const requestId = activeRef.current;
      if (requestId) {
        terminal.current.add(requestId); activeRef.current = null; setActiveRequest(null);
        setEntries(current => current.map(entry => entry.requestId === requestId && entry.role === 'assistant' ? { ...entry, state: 'failed' } : entry));
      }
      setNotice(safeError(error, 'The request was not accepted. No retry or model change occurred; check saved state and spending in Settings.'));
      try { await refreshStatus(); await refreshList(); if (id) { const saved = await readConversation(id); setConversation(saved); setEntries(saved.entries); } } catch { /* Fail closed; keep the explicit notice. */ }
    } finally { if (!issuedRequestId || !eventTerminals.current.has(issuedRequestId)) markBusy(false); }
  }
  async function cancel() {
    const id = activeRef.current;
    if (!id || cancelling) return;
    setCancelling(true);
    try {
      const result = await bridge.cancelMessage(id);
      if (!result.cancelled && activeRef.current === id) setNotice('The host could not stop this request yet. Spending is not assumed to be zero.');
    } catch { if (activeRef.current === id) setNotice('The stop request could not be confirmed. Spending remains unresolved until the host verifies usage.'); }
    finally { setCancelling(false); }
  }
  function openDialog(next: 'settings' | 'help' | 'quit', opener: HTMLElement) {
    if (workspaceRef.current === 'customize' && customizationBlockedRef.current) return;
    dialogOpener.current = opener.closest('.menu-group')?.querySelector<HTMLElement>('[data-menu]') ?? opener;
    dialogStateRef.current = next; setMenu(null); setDialog(next);
  }
  function openCustomize() {
    if (isLocked() || profileNavigationBlockedRef.current || workspaceRef.current !== 'chat' || dialogStateRef.current !== 'settings' || !preserveDraft()) return;
    // Presentation only: no adopt, history load, create or attachment cleanup.
    workspaceRef.current = 'customize'; setWorkspace('customize'); setMenu(null); setSidebarOpen(false);
    editTarget.current = null; copiedSelection.current = '';
    dialogStateRef.current = null; setDialog(null);
    requestAnimationFrame(() => workspaceReturn.current?.focus());
  }
  function returnToChat() {
    if (isLocked() || dialogStateRef.current || customizationBlockedRef.current || workspaceRef.current !== 'customize') return;
    workspaceRef.current = 'chat'; setWorkspace('chat'); setMenu(null);
    editTarget.current = null; copiedSelection.current = '';
    requestAnimationFrame(() => draftRef.current?.focus());
  }
  function dialogKeys(event: KeyboardEvent<HTMLElement>) {
    if (event.key === 'Escape') { event.preventDefault(); setDialog(null); }
    if (event.key !== 'Tab') return;
    const controls = Array.from(dialogRef.current?.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea:not(:disabled)') ?? []).filter(control => !control.closest('[hidden]'));
    if (!controls.length) return;
    if (event.shiftKey && document.activeElement === controls[0]) { event.preventDefault(); controls.at(-1)?.focus(); }
    else if (!event.shiftKey && document.activeElement === controls.at(-1)) { event.preventDefault(); controls[0].focus(); }
  }
  function menuKeys(event: KeyboardEvent<HTMLElement>) {
    if (event.key === 'Escape') { setMenu(null); menuRegion.current?.querySelector<HTMLButtonElement>(`[data-menu="${menu}"]`)?.focus(); return; }
    if (event.key === 'Tab') { setMenu(null); return; }
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      event.preventDefault(); const names: MenuName[] = ['File', 'Edit', 'View', 'Help'];
      const next = names[(names.indexOf(menu!) + (event.key === 'ArrowRight' ? 1 : -1) + names.length) % names.length];
      setMenu(next); requestAnimationFrame(() => menuRegion.current?.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')?.focus()); return;
    }
    if (!['ArrowDown','ArrowUp','Home','End'].includes(event.key)) return;
    const items = Array.from(event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)'));
    if (!items.length) return;
    event.preventDefault(); const index = items.indexOf(document.activeElement as HTMLButtonElement);
    const next = items[event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length];
    next.focus(); next.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }
  function captureEdit(element: EventTarget | null) {
    if (element instanceof HTMLElement && element.closest('.personalization-page')) {
      // Profile editing belongs exclusively to its local component. Never
      // read its value into global draft/edit-history/clipboard buffers.
      editTarget.current = null; copiedSelection.current = ''; return;
    }
    if (!(element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) || element.type === 'password' || element.selectionStart === null || element.selectionEnd === null) return;
    editTarget.current = { element, start: element.selectionStart, end: element.selectionEnd, text: element.value };
    const key = element.dataset.editKey;
    if (key) localEdits.current.write(key, element.value);
  }
  function captureVisibleSelection() {
    const selection = window.getSelection();
    const anchor = selection?.anchorNode;
    const anchorElement = anchor instanceof Element ? anchor : anchor?.parentElement;
    if (anchorElement?.closest('.personalization-page')) {
      editTarget.current = null; copiedSelection.current = ''; return;
    }
    const text = selection?.toString() ?? '';
    const withinVisibleContent = Boolean(selection?.anchorNode && document.getElementById('main')?.contains(selection.anchorNode));
    copiedSelection.current = withinVisibleContent && utf8Bytes(text) <= MAX_MESSAGE_BYTES ? text : '';
    if (withinVisibleContent && text) editTarget.current = null;
  }
  function editable() {
    const target = editTarget.current;
    return target && target.element.isConnected && !target.element.closest('[hidden]') && !target.element.disabled && !target.element.readOnly && target.element.value === target.text ? target : null;
  }
  function applyEdit(value: string, caret: number) {
    const target = editable(); if (!target) return;
    const key = target.element.dataset.editKey;
    if (target.element === draftRef.current) changeDraft(value);
    else if (key === 'search') setSearch(value);
    else if (key === 'find') setFind(value);
    else if (key?.startsWith('budget:')) setBudgetFields(current => ({ ...current, [key.slice(7)]: value }));
    else if (key?.startsWith('charge:')) setReconciliationCharges(current => ({ ...current, [key.slice(7)]: value }));
    else return;
    if (key) localEdits.current.write(key, value);
    const pending = { ...target, text: value, start: caret, end: caret }; editTarget.current = pending;
    requestAnimationFrame(() => { if (editTarget.current === pending && target.element.value === value && !dialogStateRef.current && target.element.isConnected) { target.element.focus(); target.element.setSelectionRange(caret, caret); captureEdit(target.element); } });
  }
  async function editAction(action: LocalAction) {
    if (isLocked() || dialogStateRef.current) return;
    const target = editable();
    const current = target?.element === draftRef.current;
    const history = current ? drafts.current : localEdits.current;
    const key = current ? draftKey() : target?.element.dataset.editKey;
    if (action === 'undo' || action === 'redo') {
      if (!target || !key || !(action === 'undo' ? history.canUndo(key) : history.canRedo(key))) return;
      const text = action === 'undo' ? history.undo(key) : history.redo(key);
      // applyEdit must not append the undo result as a fresh edit.
      if (current) { draftValue.current = text; setDraft(text); }
      else {
        applyEdit(text, text.length);
      }
      const pending = { ...target, text, start: text.length, end: text.length }; editTarget.current = pending;
      requestAnimationFrame(() => { if (editTarget.current === pending && target.element.value === text && !dialogStateRef.current && target.element.isConnected) { target.element.focus(); target.element.setSelectionRange(text.length, text.length); captureEdit(target.element); } }); return;
    }
    if (action === 'select') { if (target) { target.element.focus(); target.element.select(); captureEdit(target.element); } return; }
    const selected = target ? target.text.slice(target.start, target.end) : copiedSelection.current;
    if ((action === 'copy' || action === 'cut') && !selected) return;
    if ((action === 'copy' || action === 'cut') && utf8Bytes(selected) > MAX_MESSAGE_BYTES) { setNotice('Select no more than 32,768 UTF-8 bytes for clipboard editing. No text was copied or changed.'); return; }
    if (action === 'delete') { if (target && target.start !== target.end) { const result = editSelection(target.text, target.start, target.end, ''); if (result) applyEdit(result.text, result.caret); } return; }
    try {
      if (action === 'copy' || action === 'cut') await navigator.clipboard.writeText(selected);
      const pasted = action === 'paste' ? await navigator.clipboard.readText() : '';
      if (action === 'copy') return;
      if (!target || editable() !== target || isLocked() || dialogStateRef.current) return;
      const result = editSelection(target.text, target.start, target.end, pasted);
      if (!result) { setNotice('The edit exceeds 32,768 UTF-8 bytes or the selected range is invalid. No text was changed.'); return; }
      applyEdit(result.text, result.caret);
    } catch { setNotice('Clipboard access was refused or is unavailable. No text was changed or sent.'); }
  }
  async function windowAction(action: WindowAction) {
    if (!bridge.windowAction || isLocked() || (workspaceRef.current === 'customize' && customizationBlockedRef.current)) return;
    markBusy(true);
    try {
      const result = await bridge.windowAction({ action });
      if (!result || typeof result.maximised !== 'boolean' || typeof result.fullscreen !== 'boolean') throw new Error('invalid_window_state');
      setWindowState(result);
    } catch (error) { setNotice(safeError(error, 'The window action could not be confirmed. Drafts, saved history and spending were not reset.')); }
    finally { markBusy(false); }
  }
  function requestQuit() {
    if (isLocked() || dialogStateRef.current || !bridge.windowAction || (workspaceRef.current === 'customize' && customizationBlockedRef.current)) return;
    setMenu(null);
    if (draftValue.current.length || drafts.current.hasDrafts || attachmentState.current.items.length) openDialog('quit', document.activeElement instanceof HTMLElement ? document.activeElement : document.body);
    else void windowAction('close');
  }
  quitRef.current = requestQuit;
  function actionDisabled(action: LocalAction) {
    if (locked || (workspace === 'customize' && customizationBlocked)) return true;
    if (workspace === 'customize' && ['new','previous','next','sidebar','search','find','undo','redo','cut','copy','paste','delete','select'].includes(action)) return true;
    if (action === 'new') return !status?.historyAvailable || status.credentialValidation !== 'valid';
    if (action === 'quit' || action === 'fullscreen') return !bridge.windowAction;
    if (action === 'previous' || action === 'next') { const position = summaries.findIndex(item => item.id === conversation?.id); return !status?.historyAvailable || position < 0 || (action === 'previous' ? position === 0 : position === summaries.length - 1); }
    if (['undo','redo','cut','paste','delete','select'].includes(action)) {
      const target = editable(); if (!target) return true;
      const current = target.element === draftRef.current; const key = current ? draftKey() : target.element.dataset.editKey;
      if (action === 'undo' || action === 'redo') return !key || !(action === 'undo' ? (current ? drafts.current : localEdits.current).canUndo(key) : (current ? drafts.current : localEdits.current).canRedo(key));
      if (action === 'cut' || action === 'delete') return target.start === target.end;
    }
    if (action === 'copy') { const target = editable(); return !(target && target.start !== target.end) && !copiedSelection.current; }
    return false;
  }
  function performAction(action: LocalAction, opener?: HTMLElement) {
    if (isLocked() || dialogStateRef.current || actionDisabled(action)) return;
    setMenu(null);
    if (['undo','redo','cut','copy','paste','delete','select'].includes(action)) { void editAction(action); return; }
    if (action === 'new') { void newConversation(); return; }
    if (action === 'quit') { requestQuit(); return; }
    if (action === 'settings') { openDialog('settings', opener ?? document.activeElement as HTMLElement); return; }
    if (action === 'sidebar') { if (compact) { setSidebarVisible(true); setSidebarOpen(current => !current); } else setSidebarVisible(current => !current); return; }
    if (action === 'search') { setSidebarVisible(true); setSidebarOpen(true); setSearchOpen(true); requestAnimationFrame(() => searchRef.current?.focus()); return; }
    if (action === 'find') { setFindOpen(true); requestAnimationFrame(() => findRef.current?.focus()); return; }
    if (action === 'previous' || action === 'next') { const position = summaries.findIndex(item => item.id === conversationRef.current); const next = summaries[position + (action === 'previous' ? -1 : 1)]; if (next) void chooseConversation(next.id); return; }
    if (action === 'zoomIn' || action === 'zoomOut' || action === 'zoomReset') {
      const next = action === 'zoomReset' ? 100 : Math.max(80, Math.min(150, zoomRef.current + (action === 'zoomIn' ? 10 : -10)));
      zoomRef.current = next; setZoom(next); setZoomFeedback(true);
      if (zoomTimeout.current !== null) window.clearTimeout(zoomTimeout.current);
      zoomTimeout.current = window.setTimeout(() => { zoomTimeout.current = null; setZoomFeedback(false); }, 1000);
      return;
    }
    if (action === 'fullscreen') { void windowAction('toggle_fullscreen'); return; }
    if (action in helpTitles) { setHelpPage(action as HelpPage); openDialog('help', opener ?? document.activeElement as HTMLElement); }
  }
  localActionRef.current = performAction;
  function highlighted(text: string): ReactNode {
    if (!find) return text;
    const chunks: ReactNode[] = []; let cursor = 0;
    for (const span of findMatches(text, find).spans) { chunks.push(text.slice(cursor, span.start), <mark key={span.start}>{text.slice(span.start, span.end)}</mark>); cursor = span.end; }
    chunks.push(text.slice(cursor)); return chunks;
  }
  function submitBudget() {
    const values = [budgetFields.rate, budgetFields.margin, budgetFields.tax, budgetFields.payment];
    const expires = Math.floor(new Date(budgetFields.validUntil).getTime() / 1000);
    if (!values.every(value => /^(0|[1-9][0-9]*)$/.test(value) && Number.isSafeInteger(Number(value))) || !budgetFields.validUntil || !Number.isSafeInteger(expires)) {
      setNotice('Enter each financial value explicitly as a non-negative integer and choose a valid expiry.'); return;
    }
    const input: BudgetIntent = { accountId: status?.budget.accountId ?? newAccountId.current, gbpPerUsdPpm: Number(values[0]), marginBps: Number(values[1]), taxBps: Number(values[2]), paymentBps: Number(values[3]), validUntilUnix: expires };
    void nativeAction(() => bridge.configureBudget(input), 'The spending configuration was not accepted. Existing reservations remain unchanged.');
  }
  function reconcile(requestId: string) {
    const confirmedChargeNanoGbp = parseGbp(reconciliationCharges[requestId] ?? '');
    if (confirmedChargeNanoGbp === null) { setNotice('Enter the statement-confirmed GBP charge from £0 to £89, with no more than nine decimal places.'); return; }
    void nativeAction(() => bridge.reconcileBudget({ requestId, confirmedChargeNanoGbp }), 'The pending charge was not reconciled. Existing reservations remain unchanged.');
  }

  const canSend = workspace === 'chat' && !locked && status?.credentialValidation === 'valid' && status.availability === 'ready' && validMessage(draft) && ['synthetic', 'public_eligible'].includes(dataClass) && REASONING_EFFORTS.includes(reasoningEffort) && (!attachments.length || Boolean(status.capabilities.attachmentImport && attachmentState.current.conversationId === conversation?.id));
  const filteredSummaries = summaries.filter(item => item.title.toLocaleLowerCase('en-GB').includes(search.toLocaleLowerCase('en-GB')));
  const bytes = utf8Bytes(draft);
  const availability = hostState === 'checking' ? 'Checking the native host…' : hostState === 'unavailable' ? 'Native host unavailable or incompatible.' : status ? availabilityText[status.availability] : 'Unavailable';
  const matchCount = find ? entries.reduce((sum, entry) => sum + findMatches(entry.content, find).count, 0) : 0;
  const navigationVisible = workspace === 'chat' && sidebarVisible && (!compact || sidebarOpen);
  const canImport = workspace === 'chat' && !locked && !dialog && Boolean(status?.credentialValidation === 'valid' && status.historyAvailable && status.capabilities.attachmentImport && bridge.selectAttachments && bridge.removeAttachment);
  return <><div ref={shellRef} className={`app-shell ${sidebarVisible ? '' : 'sidebar-hidden'}`} style={{ zoom: zoom / 100, width: '100%', height: '100%', '--interface-scale': zoom / 100 } as CSSProperties} onFocusCapture={event => captureEdit(event.target)} onSelectCapture={event => captureEdit(event.target)} onChangeCapture={event => captureEdit(event.target)}>
    <a href={workspace === 'chat' ? '#main' : '#customize-main'} className="skip-link">Skip to content</a>
    <header className="titlebar captionbar" aria-label="Window title bar">
      <button className="mobile-menu icon-button" aria-label="Open navigation" hidden={workspace !== 'chat'} disabled={locked} onClick={() => { if (workspaceRef.current === 'chat') { setSidebarVisible(true); setSidebarOpen(true); } }}><Icon name="menu"/></button>
      <div className="titlebar-menu" ref={menuRegion}>
        {(['File','Edit','View','Help'] as MenuName[]).map(name => <div className="menu-group" key={name}>
          <button data-menu={name} aria-haspopup="menu" aria-expanded={menu === name} disabled={Boolean(dialog) || customizationBlocked} onMouseDown={captureVisibleSelection} onClick={() => setMenu(menu === name ? null : name)} onKeyDown={event => { if (event.key === 'Escape' && menu === name) { event.preventDefault(); event.stopPropagation(); setMenu(null); event.currentTarget.focus(); return; } if (event.key === 'ArrowDown') { event.preventDefault(); captureVisibleSelection(); setMenu(name); requestAnimationFrame(() => menuRegion.current?.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')?.focus()); } }}>{name}</button>
          {menu === name && <div role="menu" aria-label={name} className="menu-popup" onKeyDown={menuKeys}>
            {menuGroups[name].map((group, index) => <div key={index} role="none">{index > 0 && <div role="separator" className="menu-separator"/>}{group.map(item => <button key={item.action} role="menuitem" aria-label={item.label} disabled={actionDisabled(item.action)} onClick={event => performAction(item.action, event.currentTarget)}><span>{item.label}</span>{item.shortcut && <kbd>{item.shortcut}</kbd>}</button>)}</div>)}
          </div>}
        </div>)}
      </div>
      <div className="caption-drag-region" aria-hidden="true" onMouseDown={event => { if (event.button === 0 && event.detail === 1 && !dialogStateRef.current) void windowAction('start_dragging'); }} onDoubleClick={() => { if (!dialogStateRef.current) void windowAction('toggle_maximise'); }}/>
      <span className="language-indicator" role="img" aria-label="English (United Kingdom)" title="English (United Kingdom)"><UnionJack/></span>
      <div className="caption-controls"><button aria-label="Minimise window" disabled={locked || !bridge.windowAction || Boolean(dialog) || customizationBlocked} onClick={() => void windowAction('minimise')}><svg aria-hidden="true" width="12" height="12" viewBox="0 0 12 12"><path d="M1 6h10" stroke="currentColor"/></svg></button><button aria-label={windowState.maximised ? 'Restore window' : 'Maximise window'} disabled={locked || !bridge.windowAction || Boolean(dialog) || customizationBlocked} onClick={() => void windowAction('toggle_maximise')}><svg aria-hidden="true" width="12" height="12" viewBox="0 0 12 12">{windowState.maximised ? <path d="M3 1h8v8M1 3h8v8H1z" fill="none" stroke="currentColor"/> : <rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor"/>}</svg></button><button className="caption-close" aria-label="Close Aster" disabled={locked || !bridge.windowAction || Boolean(dialog) || customizationBlocked} onClick={requestQuit}><Icon name="close" size={14}/></button></div>
      <span className="visually-hidden" aria-label="Interface zoom">{zoom}%</span>
    </header>
    {workspace === 'chat' && sidebarOpen && <button className="sidebar-backdrop" aria-label="Close navigation" onClick={() => setSidebarOpen(false)}/>}
    <aside className={`sidebar ${sidebarOpen ? 'sidebar-open' : ''}`} hidden={workspace !== 'chat'} aria-label="Conversation navigation" aria-hidden={!navigationVisible} inert={!navigationVisible}>
      <div className="brand-slot"><img src={wordmark} alt="Aster AI" className="brand-image"/></div>
      <div className="sidebar-tools"><button className="icon-button" aria-label="New conversation" title="New conversation (Ctrl+N)" onClick={() => void newConversation()} disabled={locked || !status?.historyAvailable || status.credentialValidation !== 'valid'}><Icon name="new"/></button><button className="icon-button" aria-label="Search conversations" title="Search conversations (Ctrl+K)" disabled={locked} onClick={() => { setSearchOpen(current => !current); requestAnimationFrame(() => searchRef.current?.focus()); }}><Icon name="search"/></button></div>
      {searchOpen && <label className="search-field"><span className="visually-hidden">Search saved conversations</span><input ref={searchRef} data-edit-key="search" type="search" value={search} maxLength={256} disabled={locked} onChange={event => setSearch(event.target.value.slice(0,256))} placeholder="Search conversations"/></label>}
      <div className="section-label">Conversations <span>{summaries.length}</span></div>
      <nav ref={historyRef} className="history-list" aria-label="Saved conversations" tabIndex={0}>
        {filteredSummaries.length ? filteredSummaries.map(item => <button key={item.id} className={`history-row ${conversation?.id === item.id ? 'selected' : ''}`} aria-current={conversation?.id === item.id ? 'page' : undefined} onClick={() => void chooseConversation(item.id)} disabled={locked} title={item.title || 'Untitled conversation'}><span>{item.title || 'Untitled conversation'}</span><small>{item.entryCount} {item.entryCount === 1 ? 'message' : 'messages'}</small></button>) : <p className="muted-line">{search ? 'No matching conversations.' : status?.historyAvailable ? 'No saved conversations yet.' : 'Saved history unavailable.'}</p>}
      </nav>
      <footer className="sidebar-footer"><p>{status?.historyAvailable ? 'History is saved on this Windows account.' : 'Saved history is currently unavailable.'}<br/>Responses are generated by Meta.</p><button ref={settingsButton} className="settings-button" onClick={event => openDialog('settings', event.currentTarget)}><Icon name="settings"/><span>Settings</span></button></footer>
    </aside>
    <main className="main-panel" id="main" tabIndex={-1} hidden={workspace !== 'chat'} inert={workspace !== 'chat'}>
      {findOpen && <div className="find-bar"><label><span className="visually-hidden">Find in this conversation</span><input ref={findRef} data-edit-key="find" type="search" aria-label="Find in this conversation" maxLength={256} value={find} disabled={locked} onChange={event => setFind(event.target.value.slice(0,256))} onKeyDown={event => { if (event.key === 'Escape') { setFindOpen(false); setFind(''); draftRef.current?.focus(); } }}/></label><span role="status">{matchCount} {matchCount === 1 ? 'match' : 'matches'}</span><button className="icon-button" aria-label="Close find" onClick={() => { setFindOpen(false); setFind(''); draftRef.current?.focus(); }}><Icon name="close" size={16}/></button></div>}
      <div className="content-area">
        {!entries.length ? <section className="welcome" aria-labelledby="welcome-title"><div className="ready-orb" aria-hidden="true"><img className="ready-artwork" src={readyArtwork} alt=""/></div><h1 id="welcome-title">Ready when you are.</h1></section> : <section className="messages" aria-label="Conversation messages">
          {entries.map(entry => <article key={entry.id} className={`message message-${entry.role}`}>
            <div className="message-avatar" aria-hidden="true">{entry.role === 'assistant' ? 'A' : 'Y'}</div>
            <div className="message-body">
              <div className="message-meta"><strong>{entry.role === 'user' ? 'You' : 'Aster'}</strong>{entry.role === 'assistant' && <span>{MODEL_LABEL}</span>}</div>
              <div className="message-text">{highlighted(entry.content || (entry.state === 'streaming' ? 'Waiting for a response…' : ''))}</div>
              {entry.role === 'user' && Boolean(entry.processedAttachments?.length) && <section className="processed-attachments" aria-label="Saved processed attachments">
                <ul>{entry.processedAttachments!.map(item => <li key={item.id}>
                  <span><strong>{item.label}</strong><small>{item.kind === 'text' ? 'Processed text' : item.kind === 'image' ? 'Processed image' : `${item.pageCount} processed ${item.pageCount === 1 ? 'page' : 'pages'}`} · local history only</small></span>
                  {item.kind === 'text' ? <span className="attachment-details">Processed text is included in the message above.</span> : <button type="button" className="secondary-action" disabled={locked || Boolean(dialog) || !bridge.viewProcessedAttachment} aria-label={`View processed pages for ${item.label}`} onClick={event => void viewProcessedAttachment(entry, item, event.currentTarget)}>View processed pages</button>}
                </li>)}</ul>
                <p>Original files and disclosure permissions are not retained. {entry.processedAttachments!.some(item => item.kind === 'pdf') ? 'Reattach the original PDF for another complete native reading. ' : ''}{entry.processedAttachments!.some(item => item.kind !== 'text') ? 'Opening this local view sends nothing to Meta.' : 'Processed text is already included in the message; no original is restored.'}</p>
              </section>}
              {entry.role === 'assistant' && <p className={entry.state === 'completed' || entry.state === 'streaming' ? 'visually-hidden' : 'inline-state'} role="status">{entry.state === 'streaming' ? 'Receiving response' : entry.state === 'completed' ? 'Response completed' : entry.state === 'interrupted' ? 'Interrupted before restart. This incomplete response is not reused as context.' : entry.state === 'refused' ? 'Response refused. Not reused as context.' : entry.state === 'truncated' ? 'Response truncated. Not reused as context.' : entry.state === 'cancelled' ? 'Response stopped. Displayed text may be incomplete; spending may remain unresolved.' : 'Response failed. Displayed text may be incomplete; spending may remain unresolved.'}</p>}
            </div>
          </article>)}<div ref={endRef}/>
        </section>}
      </div>
      <div className="composer-zone">
        {status && status.credentialValidation !== 'valid' && <div className="credential-notice" role="status"><span>{credentialText[status.credentialValidation]}</span><button onClick={event => { setSettingsTab('providers'); openDialog('settings', event.currentTarget); }}>Open Settings</button></div>}
        {notice && <div className="notice" role="status"><span>{notice}</span><button className="icon-button" aria-label="Dismiss notice" onClick={() => setNotice(null)}><Icon name="close" size={16}/></button></div>}
        <div className="composer"><label className="visually-hidden" htmlFor="message-input">Message for Aster</label><textarea ref={draftRef} id="message-input" placeholder="Ask Aster" value={draft} onChange={event => changeDraft(event.target.value)} disabled={locked || Boolean(dialog)} onKeyDown={event => { if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) { event.preventDefault(); void send(); } }} rows={2} aria-describedby="composer-guidance message-limit"/>
          {Boolean(attachments.length) && <ul className="attachment-list" aria-label="Temporary attachments">{attachments.map(item => <li key={item.id}><span className="attachment-label">{item.label}</span><span className="attachment-details">{item.kind === 'pdf' ? `${item.pageCount} ${item.pageCount === 1 ? 'page' : 'pages'}` : item.kind === 'image' ? 'Image' : 'Text'} · {item.processedBytes.toLocaleString('en-GB')} bytes</span><button className="icon-button" aria-label={`Remove attachment ${item.label}`} disabled={locked || Boolean(dialog)} onClick={() => void removeAttachment(item.id)}><Icon name="close" size={14}/></button></li>)}</ul>}
          <div className="composer-controls"><button className="attachment-button" aria-label="Add attachments" disabled={!canImport} title={status?.capabilities.attachmentImport ? 'Choose a complete replacement batch of eligible public or synthetic files in Windows. Cancelling discards previous temporary attachments. Convert DOCX, XLSX or PPTX to a complete PDF externally first.' : 'Attachment import is unavailable in this host.'} onClick={() => void addAttachments()}><Icon name="plus" size={22}/></button><span className="fixed-model">{MODEL_LABEL}</span><div className="composer-options"><EffortControl effort={reasoningEffort} classification={dataClass} disabled={locked || Boolean(dialog)} onEffort={setReasoningEffort} onClassify={setDataClass}/></div>{activeRequest || status?.activeRequest ? <button className="send-button" aria-label="Stop response" title="Stop does not erase unresolved charges" onClick={() => void cancel()} disabled={!activeRequest || cancelling}><Icon name="stop"/></button> : <button className="send-button" aria-label="Send message" onClick={() => void send()} disabled={!canSend || Boolean(dialog)}><Icon name="send"/></button>}</div>
        </div>
        <p id="message-limit" className={`message-limit ${bytes > MAX_MESSAGE_BYTES ? 'over-limit' : ''}`}>{bytes.toLocaleString('en-GB')} / {MAX_MESSAGE_BYTES.toLocaleString('en-GB')} UTF-8 bytes{bytes > MAX_MESSAGE_BYTES ? ' — message too long' : ''}</p>
        <p id="composer-guidance" className="composer-hint">Enter sends · Shift+Enter adds a line · Classify the complete message and attachments. Only eligible public or synthetic content; Meta may use inputs and responses for training.</p>
        <p className="availability" role="status">{availability}</p>
      </div>
    </main>
    {workspace === 'customize' && <main id="customize-main" className="customize-workspace" tabIndex={-1} aria-label="Customize workspace">
      <nav className="workspace-rail" aria-label="Local workspace navigation"><button ref={workspaceReturn} className="icon-button" aria-label="Return to chat" title={customizationBlocked ? 'Finish or cancel the open editor or account popup before returning.' : 'Return without changing the conversation, draft or attachments.'} disabled={locked || Boolean(dialog) || customizationBlocked} onClick={returnToChat}><span aria-hidden="true">←</span></button><button className="icon-button" aria-label="Open workspace Settings" disabled={locked || Boolean(dialog) || customizationBlocked} onClick={event => openDialog('settings', event.currentTarget)}><Icon name="settings"/></button></nav>
      <Customization bridge={bridge} locked={locked || Boolean(dialog)} credentialValidated={status?.credentialValidation === 'valid'} onGenerationBound={generationBound} onGenerationFinished={generationFinished} onNavigationBlockedChange={customizationNavigation}/>
    </main>}
    {dialog && <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) setDialog(null); }}><section ref={dialogRef} className="settings-panel" role="dialog" aria-modal="true" aria-labelledby="dialog-title" onKeyDown={dialogKeys}><header className="settings-header"><h2 id="dialog-title">{dialog === 'settings' ? 'Settings and privacy' : dialog === 'quit' ? 'Unsent drafts' : helpTitles[helpPage]}</h2><button className="icon-button" aria-label="Close dialogue" onClick={() => setDialog(null)}><Icon name="close"/></button></header>
      {dialog === 'quit' ? <div className="help-copy"><p>Quitting will discard unsent drafts and temporary attachments held in this process. Saved conversations, credentials and spending records will not be removed or reset.</p><div className="settings-actions"><button className="secondary-action" data-default-cancel onClick={() => setDialog(null)}>Keep working</button><button className="primary-action" disabled={locked || !bridge.windowAction} onClick={() => { dialogStateRef.current = null; setDialog(null); void windowAction('close'); }}>Discard drafts and quit</button></div></div> : dialog === 'help' ? <div className="help-copy">
        {helpPage === 'documentation' && <p>Choosing files with + replaces the complete temporary attachment batch before the native selector opens. Cancelling that selector discards previous temporary attachments; your text draft is retained. Processed text is already included in the saved user message. View processed pages is a local native viewer for image/PDF derivatives, not an original-file reader or disclosure approval.</p>}
        {helpPage === 'documentation' && <p>The combined native PDF-original and normalised-image bytes sent inline must fit 16 MiB before base64 encoding; the complete serialised reviewed request must fit 24 MiB. These additional mixed-egress limits can refuse a batch that fits the separate source and processed-image limits; nothing is silently truncated. Encrypted processed history is limited to 16 MiB cumulative processed content per conversation, including inert text and PNG pages, without automatic eviction or quota reset.</p>}
        {helpPage === 'documentation' && <><p>Aster uses one model: {MODEL_LABEL}. Responses are generated externally, not on this laptop.</p><p>Use only synthetic examples or public material whose rights and eligibility are established. Personal, confidential, sensitive and unknown content is prohibited. A public source can still contain personal information. Each full outgoing conversation needs fresh native disclosure review.</p><p>Ctrl+N creates a conversation; Ctrl+K searches saved conversation titles. Enter sends and Shift+Enter adds a line. Stop ends the visible request, not a possible charge. Failed, stopped, refused, truncated and interrupted responses are not automatically reused as context.</p><p>Saved history is encrypted for this Windows account. Other processes under the same account remain a risk. Removing a conversation hides it from history; encrypted recovery records remain and disk capacity is not reclaimed. Credentials, spending and file-recovery records are not removed.</p><p>Use + for a native Windows file selection only when the host admits attachment import. Direct formats are TXT, MD, JSON, XML, YAML, CSV, TSV, PNG, JPG, JPEG, static GIF and PDF; no more than four temporary attachments are admitted. Adding a file does not send it, approve it or grant project access.</p><p>Each original is limited to 8 MiB and the current batch to 16 MiB. PDFs are limited to 20 pages per batch; every admitted page is processed for local preview at 150 DPI. Images are limited to 4,096 pixels per dimension and 4,194,304 decoded pixels; GIF must have one frame. A file's processed image output must fit 4 MiB. The complete prompt and text attachment context must fit the 32,768-byte message limit.</p><p>DOCX, XLSX and PPTX are not imported directly. Convert the complete document to PDF externally with your own trusted tool, such as your own LibreOffice installation, and check every page, sheet, print range and visual before attaching it. Aster does not invoke or require an Office converter. Unsupported, excessive or unsafe files are refused rather than silently truncated.</p><p>A PDF is sent natively to Meta from its immutable temporary copy, only after the complete native disclosure review. Processed page-images are a local preview/history representation, not a replacement for that native PDF request. The full original can contain metadata and hidden content that the page preview does not show; confirm eligibility of the complete source, not just its visible pages.</p><p>Originals are temporary and are not restored after restart. Removing an attachment or switching chats discards its temporary identifier. Encrypted history retains only processed content, all processed PDF pages and responses. View processed pages opens a local native view without a Meta request. These pages are not automatically resent, and no original, file-read permission or disclosure approval is reconstructed. Reattach the original PDF for another complete native reading.</p><p>Main-chat/model tools, local project reads and file writes, voice, browser, shell and general web access remain disabled. The separate GitHub popup supports explicit native-reviewed account/repository selection and bounded file reading only; it grants no write, merge or Meta disclosure permission. The £89 monthly admission ceiling is not a provider billing balance and does not cover activity outside Aster.</p><p>Unsent text drafts stay in memory while switching chats. They are not saved between restarts. Quit reviews unsent draft and temporary attachment loss with Keep working selected by default.</p><p>Find counts literal case-insensitive matches in the visible conversation. At most the first 512 matches in each message are highlighted; no protected-store or network search occurs.</p></>}
        {helpPage === 'shortcuts' && <><p>Shortcuts apply to the active Aster field or local view. Busy actions and behind-dialogue shortcuts remain unavailable. Clipboard access can be refused by the WebView; no background clipboard reader is used.</p><p>Ctrl + mouse wheel zooms the local interface. Ctrl++ (including Ctrl+= or keypad plus), Ctrl+- and Ctrl+0 use the same 80–150% range. The current percentage appears for one second after each admitted zoom action.</p><dl className="shortcut-list">{Object.values(menuGroups).flat(2).filter(item => item.shortcut).map(item => <div key={item.action}><dt>{item.label}</dt><dd>{item.shortcut}</dd></div>)}<div><dt>Send / add a line</dt><dd>Enter / Shift+Enter</dd></div><div><dt>Menu navigation</dt><dd>Arrow keys, Home, End, Escape</dd></div></dl></>}
        {helpPage === 'news' && <>
          <p>v{packageInfo.version}: the owner-supplied interface governs the adaptive conversation rail, composer and welcome artwork, with one caption/menu row and a full-page Skills and Plugins workspace. Installed layout and accessibility acceptance remain separate.</p>
          <p>New conversations, attachment import and sends require current-key authentication proof. A configured key alone is not proof; successful validation does not establish inference, billing, quota or capacity. Saved history remains locally viewable.</p>
          <p>Personalization is exclusively local and never becomes provider, summary, package, GitHub or diagnostic context. The local catalogue supports bounded inert editors, JSON import and enable preferences, including the read-only Aster Builder instructions; scripts, hooks and third-party MCP execution remain unavailable.</p>
          <p>Muse Spark generation uses only an explicitly classified eligible description and a separate native review. Its fresh ordinary local conversation retains the description, prompt and response. Only a completed valid result opens an unsaved draft; Save and enable remain separate.</p>
          <p>The dedicated Aster GitHub App has separate native account and bounded file-read intents. Enable its catalogue preference before Connect; enabling alone does not connect. Expiry or restart requires deliberate reconnect, not automatic refresh. File changes, issues, pull requests and merges remain unavailable; private repository content must not be sent to Meta. Native/live connector acceptance is not established by this view.</p>
          <p>Automatic context compaction is unfinished. The existing context limits remain in force; no silent truncation or successful compaction is claimed.</p>
          <p>Inherited attachment behaviour remains: + requests native file selection and Classify content stays below Extra high. DOCX, XLSX and PPTX require complete external PDF conversion. Native PDF bytes remain temporary; encrypted history retains processed pages and responses, and another complete native PDF reading requires reattachment. Local processed-page viewing makes no Meta request and restores no approval.</p>
          <p>The dark window starts hidden and has a one-shot native reveal after a shell scheduling signal and at least six seconds. An independent ten-second reveal attempt does not wait for status, history or a missing signal, but requires an available UI loop and operating system. Native show or scoped-hook failure remains possible and is recorded in closed diagnostics; visibility and completed paint are not guaranteed. Bounded content-free rendering observations do not establish compositor completeness or a flicker cure. The owner-reported flashes remain under investigation; installed Windows acceptance remains separate.</p>
          <p>Ctrl + mouse wheel, main-key and keypad shortcuts retain bounded interface zoom and one-second percentage feedback. Existing branding assets and blue send/stop treatment are retained; the owner-reference layout, not predecessor geometry, governs this version.</p>
        </>}
        {helpPage === 'troubleshooting' && <><p>If the host, history or spending needs attention, read the local notice and Settings. Do not reset protected records, remove unresolved charges or repeatedly retry a failed request.</p><p>Enter a key only in the native Windows dialogue. Do not paste keys into chat or diagnostic reports. A configured key does not prove a live connection.</p><p>If Windows or security software blocks the app, retain the exact alert and seek review. Do not disable protection or add exclusions. Opening help does not submit files, logs or a Meta request.</p><p>Use Keep working to cancel quitting. If clipboard access fails, ordinary editing remains local; no alternative global reader is attempted.</p></>}
        {helpPage === 'status' && <><p>Native host: {hostState === 'ready' ? 'Available; protocol 4.' : 'Unavailable or not yet verified.'}</p><p>Live inference, billing and capacity: unverified. No request or probe is made by this view.</p><p>Credential: {status?.configured ? 'Configured; bytes are not available here.' : 'Not configured or unavailable.'}</p><p>Authentication: {status ? credentialText[status.credentialValidation] : 'Current key validation is unavailable.'}</p><p>Saved history: {status?.historyAvailable ? 'Available for this Windows account.' : 'Unavailable or unverified.'}</p><p>Contributor eligibility: {status?.contributor.eligible ? 'Recorded locally; not a live entitlement check.' : 'Unconfirmed.'}</p><p>Spending policy: {status?.budget.configured ? 'Configured locally; not a provider billing balance.' : 'Not configured or unavailable.'} {status?.budget.requiresReconciliation ? 'Pending reconciliation blocks sending.' : 'No current reconciliation flag; no zero-charge claim is made.'}</p><p>Native-selected attachment import: {status?.capabilities.attachmentImport && bridge.selectAttachments && bridge.removeAttachment ? 'Available; every disclosure still needs native review.' : 'Unavailable or unverified.'}</p><p>Main-chat/model tools, local project reads and file writes, shell, browser and general web access: disabled. GitHub uses a separate reviewed native account/read route; its live acceptance is not established by this view. Mutations and merges remain unavailable.</p><p>Window controls: {bridge.windowAction ? 'Native bounded actions available; installed behaviour requires acceptance.' : 'Unavailable in this host.'}</p></>}
        {helpPage === 'about' && <><p>Aster Desktop v{packageInfo.version}</p><p>Unsigned engineering build for Windows 11 x64. This is not a production or live-provider acceptance claim.</p><p>{MODEL_LABEL} is the sole admitted model. Responses are generated by Meta.</p><p>British English interface. History is protected for this Windows account; unsent drafts remain process memory only.</p></>}
      </div> : <div className="settings-layout">
        <nav className="settings-navigation" aria-label="Settings sections">{([['providers','Providers'],['personalization','Personalization'],['usage','Usage'],['local','Local data'],['customize','Customize'],['about','About']] as const).map(([value,label]) => <button key={value} aria-current={settingsTab === value ? 'page' : undefined} onClick={() => setSettingsTab(value)}>{label}</button>)}</nav>
        <div className="settings-content">
        <div hidden={settingsTab !== 'personalization'}><Personalization bridge={bridge} locked={locked} onNavigationBlockedChange={profileNavigation}/></div>
        <section hidden={settingsTab !== 'customize'} className="settings-section"><h3>Customize workspace</h3><p>Skills and Plugins have a full-page local workspace. Opening it preserves the chat, its draft, classification and temporary attachments. Personalization stays separately in Settings and is never used to generate a package.</p><p>Save or clear any changed Personalization before opening Customize. Alternatively, explicitly close Settings to discard unsaved profile edits, then reopen it. A local profile change in progress must finish first.</p><button className="primary-action" disabled={locked || profileNavigationBlocked || workspace === 'customize'} onClick={openCustomize}>Open Customize workspace</button></section>
        <section hidden={settingsTab !== 'about'} className="settings-section"><h3>About Aster</h3><p>Aster Desktop v{packageInfo.version}. Unsigned engineering development; native, installed and live acceptance remain separate.</p><p>Personalization is exclusively local. Skills and imported plugins are inert instructions, not permissions. Context compaction is not available until its provider accounting and migration are qualified.</p><p>The owner-reported flicker remains under investigation. Startup delays are not evidence of a cure.</p><RenderingDiagnostics bridge={bridge}/></section>
        <div hidden={settingsTab !== 'providers'}>
        <section className="settings-section"><h3>Contributor terms and eligibility</h3><p>Terms revision: {status?.contributor.termsRevision ?? 'Unavailable'}. {status?.contributor.accepted ? 'Reviewed locally.' : 'Not yet reviewed.'} {status?.contributor.eligible ? 'Account eligibility recorded.' : 'Eligibility unconfirmed.'}</p><p>Contributor permits training on inputs and responses. Sensitive, personal and confidential content is prohibited; consent cannot make prohibited content admissible.</p><button className="primary-action" disabled={locked || !status} onClick={() => void nativeAction(() => bridge.acceptContributorTerms(), 'The Contributor terms or eligibility review was not accepted.')}>Review terms and eligibility in Windows</button></section>
        <section className="settings-section"><h3>Meta credential</h3><p>{status ? credentialText[status.credentialValidation] : 'The native credential status is unavailable.'}</p><p>Enter the key only in the native Windows dialogue. Authentication does not prove billing, quota or capacity.</p><div className="settings-actions"><button className="primary-action" disabled={locked || !status?.credentialAvailable} onClick={() => void nativeAction(() => bridge.configureCredential(), 'The credential could not be configured. Check the native Windows dialogue.')}>{status?.configured ? 'Change key in Windows' : 'Set up key in Windows'}</button><button className="secondary-action" disabled={locked || !status?.configured || !bridge.verifyCredential} onClick={() => void nativeAction(() => bridge.verifyCredential!(), 'Key validation could not be completed. The existing key was not labelled invalid.')}>Validate key in Windows</button><button className="secondary-action" disabled={locked || !status?.configured} onClick={() => void nativeAction(() => bridge.removeCredential(), 'The credential was not removed. Check the native confirmation.')}>Remove key…</button></div></section>
        </div><div hidden={settingsTab !== 'usage'}>
        <section className="settings-section"><h3>Monthly spending policy</h3><dl className="budget-summary"><dt>Period (Edinburgh / London)</dt><dd>{status?.budget.period ?? 'Not configured'}</dd><dt>Admission ceiling</dt><dd>£89.00</dd><dt>Recorded spending</dt><dd>{status ? formatGbp(status.budget.spentNanoGbp) : 'Unavailable'}</dd><dt>Pending reservations</dt><dd>{status ? formatGbp(status.budget.reservedNanoGbp) : 'Unavailable'}</dd><dt>Remaining admission amount</dt><dd>{status ? formatGbp(status.budget.remainingNanoGbp) : 'Unavailable'}</dd></dl>{status?.budget.warning && <p className="inline-state">The 90% spending warning has been reached.</p>}{status?.budget.requiresReconciliation && <p className="inline-state">Reconciliation required. Sending remains blocked; restarting or removing conversations does not free reservations.</p>}<p>Enter conservative values explicitly. One part per million represents 0.000001 GBP per USD; 100 basis points represents 1%. There is no automatic FX estimate or reset.</p><div className="financial-fields">{([['rate','GBP per USD (integer parts per million)'],['margin','Safety margin (basis points)'],['tax','Tax (basis points)'],['payment','Payment charges (basis points)']] as const).map(([field,label]) => <label key={field}>{label}<input inputMode="numeric" value={budgetFields[field]} onChange={event => setBudgetFields(current => ({ ...current, [field]: event.target.value }))} disabled={locked}/></label>)}<label>Valid until (local Windows time; maximum 31 days)<input type="datetime-local" value={budgetFields.validUntil} onChange={event => setBudgetFields(current => ({ ...current, validUntil: event.target.value }))} disabled={locked}/></label></div><button className="secondary-action" disabled={locked || !status} onClick={submitBudget}>Review financial configuration in Windows</button></section>
        {Boolean(status?.budget.pending.length) && <section className="settings-section"><h3>Pending request charges</h3><p>Use your Meta statement or dashboard to confirm each charge. This is owner-confirmed reconciliation, not API-measured usage. Each proposal needs a separate native confirmation; no balance reset is available.</p>{status?.budget.pending.map(item => <div className="pending-charge" key={item.requestId}><p>Request: {item.requestId}<br/>Period: {item.period} · Reserved: {formatGbp(item.reservationNanoGbp)} · {item.dispatched ? 'Dispatch may have occurred' : 'Not marked dispatched'}</p><label>Statement-confirmed charge in GBP for {item.requestId}<input inputMode="decimal" value={reconciliationCharges[item.requestId] ?? ''} onChange={event => setReconciliationCharges(current => ({ ...current, [item.requestId]: event.target.value }))} disabled={locked}/></label><button className="secondary-action" disabled={locked} onClick={() => reconcile(item.requestId)}>Review this charge in Windows</button></div>)}</section>}
        </div><div hidden={settingsTab !== 'local'}><section className="settings-section"><h3>Saved conversations</h3><p>Host-owned, encrypted history persists across restarts. Loading it restores processed display content and responses, never original files, approvals, active requests or financial authority. Historical page views are local only; reattach a PDF for another complete native reading. No history import or export is enabled.</p><button className="secondary-action" disabled={locked || !conversation} onClick={() => void removeConversation()}><Icon name="trash" size={15}/> Remove this conversation from history…</button><p>Removal needs native confirmation. Encrypted recovery records remain and disk capacity is not reclaimed. Spending, credentials and file-recovery records are unchanged.</p></section></div>
        </div>
      </div>}
    </section></div>}
  </div><div className="zoom-feedback" role="status" aria-label="Zoom level" aria-live="polite" aria-atomic="true" hidden={!zoomFeedback} style={{ '--interface-scale': zoom / 100 } as CSSProperties}>{zoomFeedback ? `${zoom}%` : ''}</div></>;
}
