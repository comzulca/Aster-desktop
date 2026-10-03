import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { nativeBridge } from './bridge';
import { formatGbp, parseGbp, MAX_CONVERSATION_BYTES, MAX_MESSAGE_BYTES, MODEL_LABEL, REASONING_EFFORTS, utf8Bytes, validConversation, validMessage, validStatus, validStreamEvent, validSummaries } from './policy';
import { DEFAULT_MODEL, type AppStatus, type Bridge, type BudgetIntent, type Conversation, type ConversationEntry, type ConversationSummary, type DataClass, type ReasoningEffort } from './types';
import wordmark from './assets/aster-wordmark.png';
import readyArtwork from './assets/aster-ready.png';

type DisplayEntry = Omit<ConversationEntry, 'state'> & { state: ConversationEntry['state'] | 'streaming' };
type MenuName = 'File' | 'Edit' | 'View' | 'Help';
const uuid = () => crypto.randomUUID();
const availabilityText: Record<AppStatus['availability'], string> = {
  needs_terms: 'Review Contributor terms in Settings before sending.',
  needs_credential: 'Set up Meta in Settings before sending.',
  needs_budget: 'Configure the spending policy in Settings before sending.',
  attention_required: 'Attention required. Check Settings; sending is unavailable.',
  ready: 'Ready for separately reviewed text requests.',
  busy: 'A request is in progress.',
  unsupported: 'The native Windows host is unavailable.',
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
  data_class_denied: 'Classify the content as synthetic or eligible public text. Other classifications are not admitted.',
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
};
function safeError(error: unknown, fallback: string): string {
  const code = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  return errorText[code] ?? fallback; // Never render provider bodies or arbitrary host error text.
}
function Icon({ name, size = 20 }: { name: 'new' | 'search' | 'settings' | 'send' | 'stop' | 'close' | 'menu' | 'trash'; size?: number }) {
  const paths: Record<typeof name, ReactNode> = {
    new: <><path d="M13 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-8"/><path d="m10 14 1-4L20 1l3 3-9 9-4 1"/></>,
    search: <><circle cx="10.5" cy="10.5" r="7"/><path d="m16 16 5 5"/></>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="m9 3 6 0 1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z"/></>,
    send: <path d="M12 20V4m-7 7 7-7 7 7"/>,
    stop: <rect x="6" y="6" width="12" height="12" rx="1" fill="currentColor" stroke="none"/>,
    close: <path d="m5 5 14 14M19 5 5 19"/>,
    menu: <path d="M3 6h18M3 12h18M3 18h18"/>,
    trash: <><path d="M4 6h16M9 3h6M6 6l1 15h10l1-15M10 10v7M14 10v7"/></>,
  };
  return <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">{paths[name]}</svg>;
}
function UnionJack() {
  return <svg aria-hidden="true" width="32" height="20" viewBox="0 0 60 36"><defs><clipPath id="flag-boundary"><rect width="60" height="36"/></clipPath></defs><g clipPath="url(#flag-boundary)"><rect width="60" height="36" fill="#012169"/><path d="m0 0 60 36m0-36L0 36" stroke="#fff" strokeWidth="8"/><path d="m0 0 60 36m0-36L0 36" stroke="#c8102e" strokeWidth="3"/><path d="M30 0v36M0 18h60" stroke="#fff" strokeWidth="12"/><path d="M30 0v36M0 18h60" stroke="#c8102e" strokeWidth="7"/></g></svg>;
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
  const [activeRequest, setActiveRequest] = useState<string | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [menu, setMenu] = useState<MenuName | null>(null);
  const [dialog, setDialog] = useState<'settings' | 'help' | null>(null);
  const [budgetFields, setBudgetFields] = useState({ rate: '', margin: '', tax: '', payment: '', validUntil: '' });
  const [reconciliationCharges, setReconciliationCharges] = useState<Record<string, string>>({});
  const newAccountId = useRef(uuid().replaceAll('-', ''));
  const activeRef = useRef<string | null>(null);
  const busyRef = useRef(false);
  const statusRef = useRef<AppStatus | null>(null);
  const conversationRef = useRef<string | null>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const dialogStateRef = useRef<'settings' | 'help' | null>(null);
  const dialogOpener = useRef<HTMLElement | null>(null);
  const settingsButton = useRef<HTMLButtonElement>(null);
  const draftRef = useRef<HTMLTextAreaElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const historyRef = useRef<HTMLElement>(null);
  const menuRegion = useRef<HTMLDivElement>(null);
  const terminal = useRef(new Set<string>());
  const eventTerminals = useRef(new Set<string>());
  const endRef = useRef<HTMLDivElement>(null);
  const locked = busy || Boolean(activeRequest) || Boolean(status?.activeRequest);

  function adopt(value: Conversation | null) {
    conversationRef.current = value?.id ?? null;
    setConversation(value); setEntries(value?.entries ?? []);
    setDraft(''); setDataClass(''); setNotice(null);
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

  useEffect(() => {
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    async function initialize() {
      markBusy(true);
      try {
        unsubscribe = await bridge.subscribe(event => {
          if (!validStreamEvent(event) || event.conversationId !== conversationRef.current || event.requestId !== activeRef.current || terminal.current.has(event.requestId)) return;
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
        if (!disposed) { statusRef.current = null; setStatus(null); setHostState('unavailable'); setNotice(safeError(error, 'The native host or its saved-history contract is unavailable or incompatible. Privileged actions are disabled.')); }
      }
      finally { if (!disposed) markBusy(false); }
    }
    void initialize();
    return () => { disposed = true; unsubscribe?.(); };
  }, [bridge]);

  useEffect(() => {
    const area = draftRef.current;
    if (area) { area.style.height = 'auto'; area.style.height = `${Math.min(Math.max(area.scrollHeight, 48), 220)}px`; }
  }, [draft]);
  useEffect(() => { endRef.current?.scrollIntoView?.({ block: 'end', behavior: 'auto' }); }, [entries]);
  useEffect(() => {
    dialogStateRef.current = dialog;
    if (dialog) dialogRef.current?.querySelector<HTMLElement>('button:not(:disabled), input:not(:disabled)')?.focus();
    else if (dialogOpener.current) {
      const previous = dialogOpener.current; dialogOpener.current = null;
      (previous.isConnected && !(previous as HTMLButtonElement).disabled ? previous : settingsButton.current)?.focus();
    }
  }, [dialog]);
  useEffect(() => {
    function shortcut(event: globalThis.KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      const key = event.key.toLowerCase();
      if (!['n', 'k'].includes(key)) return;
      event.preventDefault();
      if (dialogStateRef.current) return;
      if (key === 'n') void newConversation();
      else { setSidebarOpen(true); setSearchOpen(true); requestAnimationFrame(() => { if (!dialogStateRef.current) searchRef.current?.focus(); }); }
    }
    function outside(event: MouseEvent) { if (menuRegion.current && !menuRegion.current.contains(event.target as Node)) setMenu(null); }
    window.addEventListener('keydown', shortcut); window.addEventListener('mousedown', outside);
    return () => { window.removeEventListener('keydown', shortcut); window.removeEventListener('mousedown', outside); };
  }, []);

  async function newConversation() {
    if (isLocked() || !statusRef.current?.historyAvailable) return;
    markBusy(true); setMenu(null);
    try {
      const saved: unknown = await bridge.createConversation();
      if (!validConversation(saved)) throw new Error('history_unavailable');
      adopt(saved); await refreshList(); setSidebarOpen(false); draftRef.current?.focus();
    } catch (error) { setNotice(safeError(error, 'A new conversation could not be created. Existing history is unchanged.')); }
    finally { markBusy(false); }
  }
  async function chooseConversation(id: string) {
    if (isLocked() || !statusRef.current?.historyAvailable) return;
    markBusy(true);
    try { adopt(await readConversation(id)); setSidebarOpen(false); draftRef.current?.focus(); }
    catch (error) { setNotice(safeError(error, 'The conversation could not be loaded. Existing history is unchanged.')); }
    finally { markBusy(false); }
  }
  async function removeConversation() {
    const id = conversationRef.current;
    if (!id || isLocked()) return;
    markBusy(true); setMenu(null);
    try {
      const result = await bridge.deleteConversation({ conversationId: id });
      if (result.deleted) {
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
  async function send() {
    const currentStatus = statusRef.current;
    if (isLocked() || !currentStatus || currentStatus.availability !== 'ready' || !validMessage(draft) || !dataClass || !['synthetic', 'public_eligible'].includes(dataClass) || !REASONING_EFFORTS.includes(reasoningEffort)) return;
    const message = draft;
    const classification = dataClass;
    markBusy(true); setNotice(null);
    let id = conversationRef.current;
    let issuedRequestId: string | null = null;
    try {
      if (!id) {
        const value: unknown = await bridge.createConversation();
        if (!validConversation(value)) throw new Error('history_unavailable');
        id = value.id; conversationRef.current = id; setConversation(value); setEntries(value.entries); await refreshList();
      }
      const requestId = uuid().replaceAll('-', '');
      issuedRequestId = requestId;
      activeRef.current = requestId; setActiveRequest(requestId); terminal.current.delete(requestId);
      setEntries(current => [...current, { id: uuid(), role: 'user', content: message, requestId, state: 'completed' }, { id: uuid(), role: 'assistant', content: '', requestId, state: 'streaming' }]);
      setDraft('');
      const response = await bridge.sendMessage({ conversationId: id, requestId, modelId: DEFAULT_MODEL, message, dataClass: classification, reasoningEffort });
      if (response.requestId !== requestId || response.accepted !== true) throw new Error('request_denied');
    } catch (error) {
      if (issuedRequestId && eventTerminals.current.has(issuedRequestId)) return; // The terminal event won this race.
      // A declined native review has no saved intent. Keep the exact unsent
      // draft, but never replace a newer draft after a terminal-winning race.
      if (conversationRef.current === id && (!issuedRequestId || activeRef.current === issuedRequestId)) {
        setDraft(message); setDataClass(classification);
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
  function openDialog(next: 'settings' | 'help', opener: HTMLElement) {
    dialogOpener.current = opener.closest('.menu-group')?.querySelector<HTMLElement>('[data-menu]') ?? opener;
    dialogStateRef.current = next; setMenu(null); setDialog(next);
  }
  function dialogKeys(event: KeyboardEvent<HTMLElement>) {
    if (event.key === 'Escape') { event.preventDefault(); setDialog(null); }
    if (event.key !== 'Tab') return;
    const controls = Array.from(dialogRef.current?.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),select:not(:disabled)') ?? []);
    if (!controls.length) return;
    if (event.shiftKey && document.activeElement === controls[0]) { event.preventDefault(); controls.at(-1)?.focus(); }
    else if (!event.shiftKey && document.activeElement === controls.at(-1)) { event.preventDefault(); controls[0].focus(); }
  }
  function menuKeys(event: KeyboardEvent<HTMLElement>) {
    if (event.key === 'Escape') { setMenu(null); menuRegion.current?.querySelector<HTMLButtonElement>(`[data-menu="${menu}"]`)?.focus(); return; }
    if (!['ArrowDown','ArrowUp','Home','End'].includes(event.key)) return;
    const items = Array.from(event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)'));
    if (!items.length) return;
    event.preventDefault(); const index = items.indexOf(document.activeElement as HTMLButtonElement);
    items[event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length].focus();
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

  const canSend = !locked && status?.availability === 'ready' && validMessage(draft) && ['synthetic', 'public_eligible'].includes(dataClass) && REASONING_EFFORTS.includes(reasoningEffort);
  const filteredSummaries = summaries.filter(item => item.title.toLocaleLowerCase('en-GB').includes(search.toLocaleLowerCase('en-GB')));
  const bytes = utf8Bytes(draft);
  const availability = hostState === 'checking' ? 'Checking the native host…' : hostState === 'unavailable' ? 'Native host unavailable or incompatible.' : status ? availabilityText[status.availability] : 'Unavailable';
  return <div className="app-shell">
    <a href="#main" className="skip-link">Skip to content</a>
    <header className="titlebar">
      <button className="mobile-menu icon-button" aria-label="Open navigation" onClick={() => setSidebarOpen(true)}><Icon name="menu"/></button>
      <div className="titlebar-menu" ref={menuRegion}>
        {(['File','Edit','View','Help'] as MenuName[]).map(name => <div className="menu-group" key={name}>
          <button data-menu={name} aria-haspopup="menu" aria-expanded={menu === name} onClick={() => setMenu(menu === name ? null : name)} onKeyDown={event => { if (event.key === 'ArrowDown') { event.preventDefault(); setMenu(name); requestAnimationFrame(() => menuRegion.current?.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')?.focus()); } }}>{name}</button>
          {menu === name && <div role="menu" aria-label={name} className="menu-popup" onKeyDown={menuKeys}>
            {name === 'File' && <><button role="menuitem" disabled={locked || !status?.historyAvailable} onClick={() => void newConversation()}>New conversation <kbd>Ctrl N</kbd></button><button role="menuitem" disabled={locked || !conversation} onClick={() => void removeConversation()}>Remove conversation…</button><button role="menuitem" onClick={event => openDialog('settings', event.currentTarget)}>Settings</button></>}
            {name === 'Edit' && <><button role="menuitem" disabled={locked || !draft} onClick={() => { setMenu(null); draftRef.current?.focus(); draftRef.current?.select(); }}>Select message text</button><button role="menuitem" disabled={locked || !draft} onClick={() => { setDraft(''); setMenu(null); draftRef.current?.focus(); }}>Clear message draft</button></>}
            {name === 'View' && <><button role="menuitem" onClick={() => { setMenu(null); setSidebarOpen(true); requestAnimationFrame(() => { if (!dialogStateRef.current) historyRef.current?.focus(); }); }}>Show conversation navigation</button><button role="menuitem" onClick={() => { setMenu(null); setSearchOpen(true); setSidebarOpen(true); requestAnimationFrame(() => { if (!dialogStateRef.current) searchRef.current?.focus(); }); }}>Search conversations <kbd>Ctrl K</kbd></button></>}
            {name === 'Help' && <button role="menuitem" onClick={event => openDialog('help', event.currentTarget)}>Using Aster</button>}
          </div>}
        </div>)}
      </div>
      <span className="language-indicator" role="img" aria-label="English (United Kingdom)" title="English (United Kingdom)"><UnionJack/><span className="visually-hidden">English (United Kingdom)</span></span>
    </header>
    {sidebarOpen && <button className="sidebar-backdrop" aria-label="Close navigation" onClick={() => setSidebarOpen(false)}/>}
    <aside className={`sidebar ${sidebarOpen ? 'sidebar-open' : ''}`} aria-label="Conversation navigation">
      <div className="brand-slot"><img src={wordmark} alt="Aster AI" className="brand-image"/></div>
      <div className="sidebar-tools"><button className="icon-button" aria-label="New conversation" title="New conversation (Ctrl N)" onClick={() => void newConversation()} disabled={locked || !status?.historyAvailable}><Icon name="new"/></button><button className="icon-button" aria-label="Search conversations" title="Search conversations (Ctrl K)" onClick={() => { setSearchOpen(current => !current); requestAnimationFrame(() => searchRef.current?.focus()); }}><Icon name="search"/></button></div>
      {searchOpen && <label className="search-field"><span className="visually-hidden">Search saved conversations</span><input ref={searchRef} type="search" value={search} onChange={event => setSearch(event.target.value)} placeholder="Search conversations"/></label>}
      <div className="section-label">Conversations <span>{summaries.length}</span></div>
      <nav ref={historyRef} className="history-list" aria-label="Saved conversations" tabIndex={0}>
        {filteredSummaries.length ? filteredSummaries.map(item => <button key={item.id} className={`history-row ${conversation?.id === item.id ? 'selected' : ''}`} aria-current={conversation?.id === item.id ? 'page' : undefined} onClick={() => void chooseConversation(item.id)} disabled={locked} title={item.title || 'Untitled conversation'}><span>{item.title || 'Untitled conversation'}</span><small>{item.entryCount} {item.entryCount === 1 ? 'message' : 'messages'}</small></button>) : <p className="muted-line">{search ? 'No matching conversations.' : status?.historyAvailable ? 'No saved conversations yet.' : 'Saved history unavailable.'}</p>}
      </nav>
      <footer className="sidebar-footer"><p>{status?.historyAvailable ? 'History is saved on this Windows account.' : 'Saved history is currently unavailable.'}<br/>Responses are generated by Meta.</p><button ref={settingsButton} className="settings-button" onClick={event => openDialog('settings', event.currentTarget)}><Icon name="settings"/><span>Settings</span></button></footer>
    </aside>
    <main className="main-panel" id="main" tabIndex={-1}>
      <div className="content-area">
        {!entries.length ? <section className="welcome" aria-labelledby="welcome-title"><img className="ready-artwork" src={readyArtwork} alt=""/><h1 id="welcome-title" className="visually-hidden">Ready when you are.</h1></section> : <section className="messages" aria-label="Conversation messages">
          {entries.map(entry => <article key={entry.id} className={`message message-${entry.role}`}><div className="message-avatar" aria-hidden="true">{entry.role === 'assistant' ? 'A' : 'Y'}</div><div className="message-body"><div className="message-meta"><strong>{entry.role === 'user' ? 'You' : 'Aster'}</strong>{entry.role === 'assistant' && <span>{MODEL_LABEL}</span>}</div><div className="message-text">{entry.content || (entry.state === 'streaming' ? 'Waiting for a response…' : '')}</div>{entry.role === 'assistant' && <p className={entry.state === 'completed' || entry.state === 'streaming' ? 'visually-hidden' : 'inline-state'} role="status">{entry.state === 'streaming' ? 'Receiving response' : entry.state === 'completed' ? 'Response completed' : entry.state === 'interrupted' ? 'Interrupted before restart. This incomplete response is not reused as context.' : entry.state === 'refused' ? 'Response refused. Not reused as context.' : entry.state === 'truncated' ? 'Response truncated. Not reused as context.' : entry.state === 'cancelled' ? 'Response stopped. Displayed text may be incomplete; spending may remain unresolved.' : 'Response failed. Displayed text may be incomplete; spending may remain unresolved.'}</p>}</div></article>)}<div ref={endRef}/>
        </section>}
      </div>
      <div className="composer-zone">
        {notice && <div className="notice" role="status"><span>{notice}</span><button className="icon-button" aria-label="Dismiss notice" onClick={() => setNotice(null)}><Icon name="close" size={16}/></button></div>}
        <div className="composer"><label className="visually-hidden" htmlFor="message-input">Message for Aster</label><textarea ref={draftRef} id="message-input" placeholder="Ask Aster" value={draft} onChange={event => setDraft(event.target.value)} disabled={locked} onKeyDown={event => { if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) { event.preventDefault(); void send(); } }} rows={2} aria-describedby="composer-guidance message-limit"/>
          <div className="composer-controls"><span className="fixed-model">{MODEL_LABEL}</span><div className="composer-options"><label><span className="visually-hidden">Reasoning effort</span><select aria-label="Reasoning effort" value={reasoningEffort} onChange={event => setReasoningEffort(event.target.value as ReasoningEffort)} disabled={locked}>{REASONING_EFFORTS.map(value => <option key={value} value={value}>{value === 'xhigh' ? 'Extra high' : value[0].toUpperCase() + value.slice(1)}</option>)}</select></label><label><span className="visually-hidden">Content classification</span><select aria-label="Content classification" value={dataClass} onChange={event => setDataClass(event.target.value as DataClass | '')} disabled={locked}><option value="">Classify content</option><option value="synthetic">Synthetic</option><option value="public_eligible">Eligible public</option></select></label></div>{activeRequest || status?.activeRequest ? <button className="send-button" aria-label="Stop response" title="Stop does not erase unresolved charges" onClick={() => void cancel()} disabled={!activeRequest || cancelling}><Icon name="stop"/></button> : <button className="send-button" aria-label="Send message" onClick={() => void send()} disabled={!canSend}><Icon name="send"/></button>}</div>
        </div>
        <p id="message-limit" className={`message-limit ${bytes > MAX_MESSAGE_BYTES ? 'over-limit' : ''}`}>{bytes.toLocaleString('en-GB')} / {MAX_MESSAGE_BYTES.toLocaleString('en-GB')} UTF-8 bytes{bytes > MAX_MESSAGE_BYTES ? ' — message too long' : ''}</p>
        <p id="composer-guidance" className="composer-hint">Enter sends · Shift+Enter adds a line · Only eligible public or synthetic text. Meta may use inputs and responses for training.</p>
        <p className="availability" role="status">{availability}</p>
      </div>
    </main>
    {dialog && <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) setDialog(null); }}><section ref={dialogRef} className="settings-panel" role="dialog" aria-modal="true" aria-labelledby="dialog-title" onKeyDown={dialogKeys}><header className="settings-header"><h2 id="dialog-title">{dialog === 'settings' ? 'Settings and privacy' : 'Using Aster'}</h2><button className="icon-button" aria-label="Close dialogue" onClick={() => setDialog(null)}><Icon name="close"/></button></header>
      {dialog === 'help' ? <div className="help-copy"><p>Aster uses one model: {MODEL_LABEL}. Responses are generated externally, not on this laptop.</p><p>Use only synthetic examples or public material whose rights and eligibility are established. Personal, confidential, sensitive and unknown content is prohibited. A public source can still contain personal information. Each full outgoing conversation needs fresh native disclosure review.</p><p>Ctrl+N creates a conversation; Ctrl+K searches saved conversation titles. Enter sends and Shift+Enter adds a line. Stop ends the visible request, not a possible charge. Failed, stopped, refused, truncated and interrupted responses are not automatically reused as context.</p><p>Saved history is encrypted for this Windows account. Other processes under the same account remain a risk. Removing a conversation hides it from history; encrypted recovery records remain and disk capacity is not reclaimed. Credentials, spending and file-recovery records are not removed.</p><p>No project attachments, file writes, voice, browser, shell, web or model tools are enabled. The £89 monthly admission ceiling is not a provider billing balance and does not cover activity outside Aster.</p></div> : <>
        <section className="settings-section"><h3>Contributor terms and eligibility</h3><p>Terms revision: {status?.contributor.termsRevision ?? 'Unavailable'}. {status?.contributor.accepted ? 'Reviewed locally.' : 'Not yet reviewed.'} {status?.contributor.eligible ? 'Account eligibility recorded.' : 'Eligibility unconfirmed.'}</p><p>Contributor permits training on inputs and responses. Sensitive, personal and confidential content is prohibited; consent cannot make prohibited content admissible.</p><button className="primary-action" disabled={locked || !status} onClick={() => void nativeAction(() => bridge.acceptContributorTerms(), 'The Contributor terms or eligibility review was not accepted.')}>Review terms and eligibility in Windows</button></section>
        <section className="settings-section"><h3>Meta credential</h3><p>{status?.configured ? 'Configured for this Windows account. This is not proof of a live connection.' : 'Not configured. Enter the key only in the native Windows dialogue.'}</p><div className="settings-actions"><button className="primary-action" disabled={locked || !status?.credentialAvailable} onClick={() => void nativeAction(() => bridge.configureCredential(), 'The credential could not be configured. Check the native Windows dialogue.')}>{status?.configured ? 'Change key in Windows' : 'Set up key in Windows'}</button><button className="secondary-action" disabled={locked || !status?.configured} onClick={() => void nativeAction(() => bridge.removeCredential(), 'The credential was not removed. Check the native confirmation.')}>Remove key…</button></div></section>
        <section className="settings-section"><h3>Monthly spending policy</h3><dl className="budget-summary"><dt>Period (Edinburgh / London)</dt><dd>{status?.budget.period ?? 'Not configured'}</dd><dt>Admission ceiling</dt><dd>£89.00</dd><dt>Recorded spending</dt><dd>{status ? formatGbp(status.budget.spentNanoGbp) : 'Unavailable'}</dd><dt>Pending reservations</dt><dd>{status ? formatGbp(status.budget.reservedNanoGbp) : 'Unavailable'}</dd><dt>Remaining admission amount</dt><dd>{status ? formatGbp(status.budget.remainingNanoGbp) : 'Unavailable'}</dd></dl>{status?.budget.warning && <p className="inline-state">The 90% spending warning has been reached.</p>}{status?.budget.requiresReconciliation && <p className="inline-state">Reconciliation required. Sending remains blocked; restarting or removing conversations does not free reservations.</p>}<p>Enter conservative values explicitly. One part per million represents 0.000001 GBP per USD; 100 basis points represents 1%. There is no automatic FX estimate or reset.</p><div className="financial-fields">{([['rate','GBP per USD (integer parts per million)'],['margin','Safety margin (basis points)'],['tax','Tax (basis points)'],['payment','Payment charges (basis points)']] as const).map(([field,label]) => <label key={field}>{label}<input inputMode="numeric" value={budgetFields[field]} onChange={event => setBudgetFields(current => ({ ...current, [field]: event.target.value }))} disabled={locked}/></label>)}<label>Valid until (local Windows time; maximum 31 days)<input type="datetime-local" value={budgetFields.validUntil} onChange={event => setBudgetFields(current => ({ ...current, validUntil: event.target.value }))} disabled={locked}/></label></div><button className="secondary-action" disabled={locked || !status} onClick={submitBudget}>Review financial configuration in Windows</button></section>
        {Boolean(status?.budget.pending.length) && <section className="settings-section"><h3>Pending request charges</h3><p>Use your Meta statement or dashboard to confirm each charge. This is owner-confirmed reconciliation, not API-measured usage. Each proposal needs a separate native confirmation; no balance reset is available.</p>{status?.budget.pending.map(item => <div className="pending-charge" key={item.requestId}><p>Request: {item.requestId}<br/>Period: {item.period} · Reserved: {formatGbp(item.reservationNanoGbp)} · {item.dispatched ? 'Dispatch may have occurred' : 'Not marked dispatched'}</p><label>Statement-confirmed charge in GBP for {item.requestId}<input inputMode="decimal" value={reconciliationCharges[item.requestId] ?? ''} onChange={event => setReconciliationCharges(current => ({ ...current, [item.requestId]: event.target.value }))} disabled={locked}/></label><button className="secondary-action" disabled={locked} onClick={() => reconcile(item.requestId)}>Review this charge in Windows</button></div>)}</section>}
        <section className="settings-section"><h3>Saved conversations</h3><p>Host-owned, encrypted history persists across restarts. Loading it restores display text, never approvals, active requests or financial authority. No import or export is enabled.</p><button className="secondary-action" disabled={locked || !conversation} onClick={() => void removeConversation()}><Icon name="trash" size={15}/> Remove this conversation from history…</button><p>Removal needs native confirmation. Encrypted recovery records remain and disk capacity is not reclaimed. Spending, credentials and file-recovery records are unchanged.</p></section>
      </>}
    </section></div>}
  </div>;
}
