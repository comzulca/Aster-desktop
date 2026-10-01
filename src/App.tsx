import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { nativeBridge } from './bridge';
import { MODELS, safeRelativePath, validStatus, validStreamEvent } from './policy';
import { DEFAULT_MODEL, MUSE_MODEL, type AppStatus, type Bridge, type FileReceipt, type ModelId, type StreamEvent } from './types';

type Entry = {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  model?: ModelId;
  requestId?: string;
  state?: 'streaming' | 'completed' | 'failed' | 'cancelled';
  attachmentPath?: string;
  proposalReviewed?: boolean;
};

const newId = () => crypto.randomUUID();

function Icon({ name, size = 19 }: { name: 'compose' | 'search' | 'folder' | 'settings' | 'send' | 'plus' | 'shield' | 'bolt' | 'close' | 'menu' | 'stop' | 'file' | 'key'; size?: number }) {
  const paths: Record<typeof name, ReactNode> = {
    compose: <><path d="m12 19 7-7 2 2-7 7-4 1 1-4 7-7 2 2"/><path d="M4 20h5"/></>,
    search: <><circle cx="10.8" cy="10.8" r="7"/><path d="m16 16 5 5"/></>,
    folder: <path d="M3 7.5h7l2 2H21V20H3z"/>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="M10 2h4l.6 2.2 2.1.9 2-.9 2.8 2.8-.9 2 .9 2.1L24 12l-2.5.6-.9 2.1.9 2-2.8 2.8-2-.9-2.1.9L14 22h-4l-.6-2.5-2.1-.9-2 .9-2.8-2.8.9-2-.9-2.1L0 12l2.5-.9.9-2.1-.9-2 2.8-2.8 2 .9 2.1-.9z" transform="translate(2 2) scale(.83)"/></>,
    send: <><path d="M12 20V4m-7 7 7-7 7 7"/></>,
    plus: <path d="M12 4v16M4 12h16"/>,
    shield: <><path d="M12 2 4 5v6c0 5 3 9 8 11 5-2 8-6 8-11V5z"/><path d="m9 12 2 2 4-4"/></>,
    bolt: <path d="m13 2-9 11h7l-1 9 10-12h-7z"/>,
    close: <path d="M5 5 19 19M19 5 5 19"/>,
    menu: <path d="M3 6h18M3 12h18M3 18h18"/>,
    stop: <rect x="5" y="5" width="14" height="14" rx="2"/>,
    file: <><path d="M6 2h8l5 5v15H6z"/><path d="M14 2v5h5"/></>,
    key: <><circle cx="8" cy="15" r="4"/><path d="M11 12 21 2m-5 5 2 2m-4 0 2 2"/></>,
  };
  return <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">{paths[name]}</svg>;
}

export function App({ bridge = nativeBridge }: { bridge?: Bridge }) {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [connection, setConnection] = useState<'checking' | 'ready' | 'unavailable'>('checking');
  const [notice, setNotice] = useState<string | null>(null);
  const [model, setModel] = useState<ModelId>(DEFAULT_MODEL);
  const [conversationId, setConversationId] = useState(newId);
  const [entries, setEntries] = useState<Entry[]>([]);
  const [draft, setDraft] = useState('');
  const [activeRequest, setActiveRequest] = useState<string | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [secretBusy, setSecretBusy] = useState(false);
  const [workspaceBusy, setWorkspaceBusy] = useState(false);
  const [attachment, setAttachment] = useState<FileReceipt | null>(null);
  const draftRef = useRef<HTMLTextAreaElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const settingsTriggerRef = useRef<HTMLButtonElement>(null);
  const settingsOpenerRef = useRef<HTMLButtonElement | null>(null);
  const settingsOpenRef = useRef(false);
  const settingsDialogRef = useRef<HTMLElement>(null);
  const conversationRef = useRef(conversationId);
  const activeRef = useRef<string | null>(null);
  const terminalRef = useRef(new Set<string>());

  useEffect(() => { conversationRef.current = conversationId; }, [conversationId]);
  useEffect(() => { bottomRef.current?.scrollIntoView?.({ block: 'end', behavior: 'smooth' }); }, [entries]);
  useEffect(() => {
    if (settingsOpen) {
      settingsDialogRef.current?.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus();
    } else if (settingsOpenerRef.current) {
      const opener = settingsOpenerRef.current;
      settingsOpenerRef.current = null;
      // Restore after the modal is removed; credential changes can remove the banner opener.
      if (opener.isConnected && !opener.disabled) opener.focus();
      else settingsTriggerRef.current?.focus();
    }
  }, [settingsOpen]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    async function initialize() {
      try {
        unlisten = await bridge.subscribe((event: StreamEvent) => {
          if (!validStreamEvent(event) || event.conversationId !== conversationRef.current ||
              event.requestId !== activeRef.current || terminalRef.current.has(event.requestId)) return;
          if (event.type === 'content') {
            setEntries((current) => current.map((entry) => entry.requestId === event.requestId && entry.role === 'assistant'
              ? { ...entry, content: (entry.content + event.text!).slice(0, 8 * 1024 * 1024) } : entry));
            return;
          }
          terminalRef.current.add(event.requestId);
          activeRef.current = null;
          setActiveRequest(null);
          setCancelling(false);
          void refreshStatus();
          setEntries((current) => current.map((entry) => entry.requestId === event.requestId && entry.role === 'assistant'
            ? { ...entry, state: event.status } : entry));
          if (event.status === 'failed') setNotice('La solicitud terminó con un error. No se reintentó ni se cambió de modelo.');
          if (event.status === 'cancelled') setNotice('Solicitud cancelada.');
        });
        if (disposed) { unlisten(); return; }
        const value: unknown = await bridge.getStatus();
        if (!validStatus(value)) throw new Error('UNSUPPORTED_HOST');
        if (!disposed) { setStatus(value); setConnection('ready'); }
      } catch {
        if (!disposed) { setStatus(null); setConnection('unavailable'); }
      }
    }
    void initialize();
    return () => { disposed = true; unlisten?.(); };
  }, [bridge]);

  useEffect(() => {
    function shortcut(event: globalThis.KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      if (settingsOpenRef.current && ['n', 'k'].includes(event.key.toLowerCase())) {
        event.preventDefault();
        return;
      }
      if (event.key.toLowerCase() === 'n' && !activeRef.current) { event.preventDefault(); startNewChat(); }
      if (event.key.toLowerCase() === 'k') {
        event.preventDefault(); setSearchOpen(true); setSidebarOpen(true);
        requestAnimationFrame(() => { if (!settingsOpenRef.current) searchRef.current?.focus(); });
      }
    }
    window.addEventListener('keydown', shortcut);
    return () => window.removeEventListener('keydown', shortcut);
  }, []);

  async function refreshStatus() {
    try {
      const value: unknown = await bridge.getStatus();
      if (!validStatus(value)) throw new Error('UNSUPPORTED_HOST');
      setStatus(value);
      setConnection('ready');
    } catch {
      setStatus(null);
      setConnection('unavailable');
    }
  }

  function startNewChat() {
    if (activeRef.current) return;
    const id = newId();
    conversationRef.current = id;
    setConversationId(id);
    setEntries([]);
    setDraft('');
    setAttachment(null);
    setModel(DEFAULT_MODEL);
    setNotice(null);
    setSidebarOpen(false);
    draftRef.current?.focus();
  }

  async function configureCredential() {
    if (!status || secretBusy) return;
    setSecretBusy(true);
    setNotice(null);
    try {
      await bridge.configureCredential(); // Native Windows input: no secret through the renderer.
      await refreshStatus();
    } catch { setNotice('No se pudo configurar la credencial en Windows. Comprueba la ventana nativa.'); }
    finally { setSecretBusy(false); }
  }

  async function removeCredential() {
    if (!status || secretBusy || activeRef.current) return;
    setSecretBusy(true);
    setNotice(null);
    try {
      await bridge.removeCredential(); // Host owns the native confirmation.
      await refreshStatus();
    } catch { setNotice('La credencial no se eliminó; consulta la ventana nativa.'); }
    finally { setSecretBusy(false); }
  }

  async function selectProject() {
    if (!status || activeRef.current || workspaceBusy) return;
    setWorkspaceBusy(true);
    setNotice(null);
    try {
      await bridge.selectProject();
      setAttachment(null);
      await refreshStatus();
    } catch { setNotice('No se seleccionó un proyecto.'); }
    finally { setWorkspaceBusy(false); }
  }

  async function attachFile() {
    if (!status?.projectSelected || !status.capabilities.projectRead || activeRef.current || workspaceBusy) return;
    setWorkspaceBusy(true);
    setNotice(null);
    try {
      const requestId = newId();
      const selected = await bridge.selectProjectFile({ conversationId, requestId });
      if (selected && (selected.requestId !== requestId || !safeRelativePath(selected.relativePath) ||
          !/^[a-f0-9]{64}$/.test(selected.sha256) || !Number.isSafeInteger(selected.byteCount) ||
          selected.byteCount < 1 || selected.byteCount > 1024 * 1024)) throw new Error('INVALID_FILE_RECEIPT');
      setAttachment(selected);
    } catch { setAttachment(null); setNotice('No se pudo adjuntar el archivo. Elige un archivo de texto dentro del proyecto.'); }
    finally { setWorkspaceBusy(false); }
  }

  async function reviewProposal(entry: Entry) {
    if (!status?.projectSelected || !status.capabilities.proposals || !entry.requestId ||
        entry.state !== 'completed' || activeRef.current || workspaceBusy || entry.proposalReviewed) return;
    setWorkspaceBusy(true);
    setNotice(null);
    try {
      const result = await bridge.reviewProjectProposal({ conversationId, requestId: entry.requestId, proposalJson: entry.content });
      setEntries((current) => current.map((item) => item.id === entry.id ? { ...item, proposalReviewed: true } : item));
      const notices = {
        verified: `Archivo creado y verificado: ${result.relativePath}`,
        attention_required: 'La operación necesita revisión. Comprueba el archivo antes de continuar; Aster conserva el registro de recuperación.',
        denied: 'Propuesta cancelada en la revisión nativa.',
        proposal_only: 'Reemplazo revisado. La aplicación de cambios a archivos existentes aún no está disponible.',
      };
      setNotice(notices[result.status] ?? 'Resultado de propuesta no reconocido.');
    } catch { setNotice('No se pudo revisar la propuesta. La respuesta debe ser un único JSON de cambio permitido; el archivo o la selección pueden haber caducado.'); }
    finally { setWorkspaceBusy(false); }
  }

  async function send() {
    const message = draft.trim();
    if (!status?.configured || connection !== 'ready' || !message || activeRef.current ||
        workspaceBusy || secretBusy || !MODELS.some((item) => item.id === model) || message.length > 32_768) return;
    const selectedFile = attachment;
    const requestId = selectedFile?.requestId ?? newId();
    terminalRef.current.delete(requestId);
    activeRef.current = requestId;
    setActiveRequest(requestId);
    setNotice(null);
    setEntries((current) => [...current,
      { id: newId(), role: 'user', content: message, attachmentPath: selectedFile?.relativePath },
      { id: newId(), role: 'assistant', content: '', model, requestId, state: 'streaming' },
    ]);
    setDraft('');
    setAttachment(null);
    try {
      const intent = { conversationId, requestId, modelId: model, message };
      const result = selectedFile
        ? await bridge.sendProjectMessage({ ...intent, relativePath: selectedFile.relativePath, expectedSha256: selectedFile.sha256 })
        : await bridge.sendMessage(intent);
      if (result.requestId !== requestId || !result.accepted) throw new Error('REQUEST_DENIED');
    } catch {
      if (activeRef.current !== requestId) return; // Terminal event already won the race.
      terminalRef.current.add(requestId);
      activeRef.current = null;
      setActiveRequest(null);
      setEntries((current) => current.map((entry) => entry.requestId === requestId && entry.role === 'assistant'
        ? { ...entry, state: 'failed' } : entry));
      setNotice('Solicitud rechazada o sin conexión. No se reintentó ni se cambió de modelo.');
    }
  }

  async function cancel() {
    if (!activeRef.current || cancelling) return;
    const requestId = activeRef.current;
    setCancelling(true);
    try {
      const result = await bridge.cancelMessage(requestId);
      if (!result.cancelled && activeRef.current === requestId) setNotice('No fue posible solicitar la cancelación.');
    } catch { if (activeRef.current === requestId) setNotice('No fue posible solicitar la cancelación.'); }
    finally { setCancelling(false); }
  }

  function onComposerKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      void send();
    }
  }

  function openSettings(opener: HTMLButtonElement) {
    settingsOpenerRef.current = opener;
    settingsOpenRef.current = true;
    setSettingsOpen(true);
  }

  function closeSettings() {
    settingsOpenRef.current = false;
    setSettingsOpen(false);
  }

  function onSettingsKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === 'Escape') {
      event.preventDefault();
      closeSettings();
    }
    if (event.key !== 'Tab' || !settingsDialogRef.current) return;
    const buttons = Array.from(settingsDialogRef.current.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'));
    if (!buttons.length) return;
    if (event.shiftKey && document.activeElement === buttons[0]) { event.preventDefault(); buttons[buttons.length - 1].focus(); }
    else if (!event.shiftKey && document.activeElement === buttons[buttons.length - 1]) { event.preventDefault(); buttons[0].focus(); }
  }

  const statusText = connection === 'checking' ? 'Verificando anfitrión' :
    connection === 'unavailable' ? 'Anfitrión no disponible' :
      !status?.configured ? 'Clave sin configurar' : 'Clave configurada · conexión no verificada';
  const canSend = connection === 'ready' && Boolean(status?.configured) && !status?.activeRequest && Boolean(draft.trim()) &&
    draft.length <= 32_768 && !activeRequest && !workspaceBusy && !secretBusy;
  const filtered = searchTerm ? entries.filter((entry) => entry.content.toLocaleLowerCase('es').includes(searchTerm.toLocaleLowerCase('es'))) : entries;

  return <div className="app-shell">
    <a className="skip-link" href="#main">Saltar al contenido</a>
    <header className="titlebar">
      <button className="mobile-menu icon-button" aria-label="Abrir navegación" onClick={() => setSidebarOpen(true)}><Icon name="menu" /></button>
      <div className="titlebar-menu" aria-label="Menús pendientes de implementación"><button disabled title="No disponible en esta versión">Archivo</button><button disabled title="No disponible en esta versión">Editar</button><button disabled title="No disponible en esta versión">Ver</button><button disabled title="No disponible en esta versión">Ayuda</button></div>
      <span className="titlebar-brand"><span className="mini-orbit">A</span> Aster <span className="version-mark">v0.1.0</span></span>
      <span className="titlebar-meta">Windows · sesión temporal</span>
    </header>

    {sidebarOpen && <button className="sidebar-backdrop" aria-label="Cerrar navegación" onClick={() => setSidebarOpen(false)} />}
    <aside className={`sidebar ${sidebarOpen ? 'sidebar-open' : ''}`} aria-label="Navegación">
      <div className="brand-slot"><span className="brand-image" role="img" aria-label="Aster AI; logotipo de la referencia original" /></div>
      <button className="nav-primary" onClick={startNewChat} disabled={Boolean(activeRequest)}><Icon name="compose" /><span>Nueva conversación</span><kbd>Ctrl N</kbd></button>
      <button className="nav-link" onClick={() => { setSearchOpen((current) => !current); requestAnimationFrame(() => searchRef.current?.focus()); }}><Icon name="search" /><span>Buscar en esta sesión</span><kbd>Ctrl K</kbd></button>
      {searchOpen && <label className="search-field"><span className="visually-hidden">Buscar mensajes en esta sesión</span><input ref={searchRef} type="search" value={searchTerm} onChange={(event) => setSearchTerm(event.target.value)} placeholder="Buscar mensajes" /></label>}
      <div className="section-label"><span>CONVERSACIÓN</span><span>{entries.length ? 1 : 0}</span></div>
      <div className="session-list" aria-live="off">{entries.length ? <div className="session-row"><Icon name="compose" size={16}/><span>Sesión actual</span></div> : <p className="muted-line">Todavía no hay mensajes</p>}</div>
      <div className="sidebar-divider" />
      <div className="section-label"><span>PROYECTO</span></div>
      <button className="nav-link project-button" onClick={() => void selectProject()} disabled={!status || Boolean(activeRequest) || workspaceBusy}><Icon name="folder" /><span>Seleccionar carpeta</span><Icon name="plus" size={15}/></button>
      <p className="project-label" title={status?.projectLabel ?? undefined}>{status?.projectSelected ? status.projectLabel : 'Sin proyecto seleccionado'}</p>
      <div className="sidebar-bottom">
        <div className="sidebar-tip"><Icon name="shield" size={21}/><span><strong>Control local</strong><small>Sin historial guardado en disco.</small></span></div>
        <button className="account-button" onClick={(event) => openSettings(event.currentTarget)}><span className="avatar">A</span><span><strong>Ajustes</strong><small>{status?.configured ? 'Clave configurada' : 'NVIDIA sin configurar'}</small></span><Icon name="settings" size={18}/></button>
      </div>
    </aside>

    <main className="main-panel" id="main">
      <div className="conversation-heading"><strong>{entries.length ? 'Sesión actual' : 'Nueva conversación'}</strong><span className={`connection ${connection === 'ready' && status?.configured ? 'connected' : ''}`} role="status" aria-live="polite"><span className="status-dot" />{statusText}</span><button ref={settingsTriggerRef} className="icon-button" aria-label="Abrir ajustes" onClick={(event) => openSettings(event.currentTarget)}><Icon name="settings"/></button></div>
      <div className="content-area">
        {!entries.length ? <section className="welcome" aria-labelledby="welcome-title">
          <div className="welcome-icon" role="img" aria-label="Emblema Aster de la imagen original" />
          <p className="eyebrow">ASISTENTE DE ESCRITORIO · CONTROL LOCAL</p>
          <h1 id="welcome-title">¿En qué quieres trabajar?</h1>
          <p className="welcome-description">Inicia una conversación. Se conserva solo durante esta sesión.</p>
          <div className="starter-actions">
            <button onClick={() => { setDraft('Ayúdame a revisar una decisión de seguridad, identifica riesgos y controles.'); draftRef.current?.focus(); }}><Icon name="shield"/><span><strong>Revisar una decisión de seguridad</strong><small>Identificar riesgos y controles</small></span></button>
            <button onClick={() => { setDraft('Ayúdame a planificar una implementación a partir de requisitos verificables.'); draftRef.current?.focus(); }}><Icon name="bolt"/><span><strong>Planificar una implementación</strong><small>Convertir requisitos en pasos</small></span></button>
          </div>
        </section> : <section className="messages" aria-label="Mensajes de la sesión">
          {filtered.length ? filtered.map((entry) => <article key={entry.id} className={`message message-${entry.role}`}>
            <div className="message-avatar" aria-hidden="true">{entry.role === 'assistant' ? 'A' : 'T'}</div>
            <div className="message-body"><div className="message-meta"><strong>{entry.role === 'user' ? 'Tú' : 'Aster'}</strong>{entry.model && <span>{MODELS.find((item) => item.id === entry.model)?.label}</span>}</div>
              <div className="message-text">{entry.content || (entry.state === 'streaming' ? 'Esperando respuesta…' : '')}</div>
              {entry.attachmentPath && <p className="inline-state">Archivo enviado tras revisión nativa: {entry.attachmentPath}</p>}
              {entry.role === 'assistant' && <span className="visually-hidden" role="status" aria-live="polite">{entry.state === 'streaming' ? 'Recibiendo respuesta' : entry.state === 'completed' ? 'Respuesta completada' : entry.state === 'cancelled' ? 'Respuesta cancelada' : 'Respuesta fallida'}</span>}
              {entry.role === 'assistant' && entry.state === 'failed' && <p className="inline-state" role="status">La respuesta terminó con un error; podría estar incompleta.</p>}
              {entry.role === 'assistant' && entry.state === 'cancelled' && <p className="inline-state" role="status">Solicitud cancelada. El texto mostrado podría estar incompleto.</p>}
              {entry.role === 'assistant' && entry.state === 'completed' && entry.content && <button className="text-action" onClick={() => void reviewProposal(entry)} disabled={!status?.projectSelected || !status.capabilities.proposals || workspaceBusy || Boolean(activeRequest) || entry.proposalReviewed} title="Revisar un único JSON de propuesta en la ventana nativa"><Icon name="file" size={15}/> {entry.proposalReviewed ? 'Propuesta revisada' : 'Revisar propuesta'}</button>}
            </div>
          </article>) : <p className="empty-search">No hay mensajes que coincidan con la búsqueda.</p>}
          <div ref={bottomRef} />
        </section>}
      </div>

      <div className="composer-zone">
        {notice && <div className="notice" role="status"><span>{notice}</span><button className="icon-button" onClick={() => setNotice(null)} aria-label="Cerrar aviso"><Icon name="close" size={16}/></button></div>}
        {!status?.configured && <div className="connect-banner"><Icon name="key"/><div><strong>{connection === 'unavailable' ? 'Abre la app de escritorio para conectarte' : 'Configura NVIDIA para enviar'}</strong><small>{connection === 'unavailable' ? 'La vista web no tiene acceso al anfitrión de Windows.' : 'La clave se introduce en una ventana nativa de Windows.'}</small></div><button onClick={(event) => openSettings(event.currentTarget)} disabled={connection === 'unavailable'}>Abrir ajustes</button></div>}
        {status?.projectSelected && <div className="project-context"><span><Icon name="folder" size={16}/>{status.projectLabel}</span><small>{status.capabilities.projectRead ? 'Adjunta un archivo de texto; cada envío requiere revisión nativa' : 'Acceso a archivos no disponible en este anfitrión'}</small></div>}
        {attachment && <div className="project-context"><span><Icon name="file" size={16}/>{attachment.relativePath} · {attachment.byteCount.toLocaleString('es')} bytes</span><button className="icon-button" aria-label="Quitar archivo adjunto" onClick={() => setAttachment(null)} disabled={Boolean(activeRequest)}><Icon name="close" size={16}/></button></div>}
        <div className="composer"><label htmlFor="message-input" className="visually-hidden">Mensaje para Aster</label><textarea ref={draftRef} id="message-input" placeholder="Pregúntale a Aster lo que necesites" value={draft} onChange={(event) => setDraft(event.target.value)} onKeyDown={onComposerKeyDown} maxLength={32_768} rows={2} disabled={Boolean(activeRequest)} aria-describedby="composer-hint" />
          <div className="composer-controls"><button className="composer-add" type="button" aria-label="Adjuntar archivo" onClick={() => void attachFile()} disabled={!status?.projectSelected || !status.capabilities.projectRead || Boolean(activeRequest) || workspaceBusy} title="Seleccionar un archivo de texto del proyecto"><Icon name="plus" size={20}/></button><label htmlFor="model-select" className="visually-hidden">Modelo</label><select id="model-select" value={model} onChange={(event) => { const next = event.target.value; if (next === DEFAULT_MODEL || next === MUSE_MODEL) setModel(next); }} disabled={!status || Boolean(activeRequest) || workspaceBusy}>{MODELS.map((entry) => <option key={entry.id} value={entry.id}>{entry.label}</option>)}</select><span className="stream-indicator"><Icon name="bolt" size={15}/> Streaming</span><span className="composer-spacer"/>{activeRequest ? <button className="send-button cancel-button" aria-label="Cancelar solicitud" onClick={() => void cancel()} disabled={cancelling}><Icon name="stop" size={18}/></button> : <button className="send-button" aria-label="Enviar mensaje" onClick={() => void send()} disabled={!canSend}><Icon name="send" size={19}/></button>}</div>
        </div>
        <p id="composer-hint" className="composer-hint">Enter envía · Mayús+Enter añade una línea · El mensaje se transmite a NVIDIA cuando lo envías</p>
        <p className="footer-note">Sesión temporal · archivos bajo tu control · sin web ni terminal</p>
      </div>
    </main>

    {settingsOpen && <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) closeSettings(); }}><section ref={settingsDialogRef} onKeyDown={onSettingsKeyDown} className="settings-panel" role="dialog" aria-modal="true" aria-labelledby="settings-title"><div className="settings-header"><div><p className="eyebrow">ASTER DESKTOP</p><h2 id="settings-title">Conexión y privacidad</h2></div><button className="icon-button" aria-label="Cerrar ajustes" onClick={closeSettings}><Icon name="close"/></button></div>
      <div className="settings-row"><Icon name="key"/><div><strong>Credencial de NVIDIA</strong><p>{status?.configured ? 'Configurada para esta cuenta de Windows. Conexión aún sin verificar.' : 'Sin configurar. La clave solo se introduce en la ventana nativa.'}</p></div></div>
      <div className="settings-actions"><button className="primary-action" disabled={!status || secretBusy} onClick={() => void configureCredential()}>{status?.configured ? 'Cambiar clave en Windows' : 'Configurar clave en Windows'}</button>{status?.configured && <button className="secondary-action" disabled={secretBusy || Boolean(activeRequest)} onClick={() => void removeCredential()}>Eliminar clave</button>}</div>
      <div className="privacy-note"><Icon name="shield"/><p>El historial de chat no se guarda en disco. Windows Credential Manager protege la clave bajo tu cuenta; otros procesos ejecutados con esa misma cuenta pueden acceder a credenciales genéricas.</p></div>
      <p className="hint">Seleccionar una carpeta permite elegir archivos concretos. Cada envío de su contenido y cada creación requieren revisión nativa. Los reemplazos de archivos existentes quedan como propuestas.</p>
    </section></div>}
  </div>;
}
