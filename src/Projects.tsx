import { useEffect, useId, useRef, useState, type FormEvent, type ReactNode } from 'react';
import type { Bridge, ProjectSnapshot } from './types';

const opaqueId = /^(?:[a-f0-9]{32}|[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12})$/;
const reserved = /[\p{Cc}\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/u;

function validName(name: string) {
  return name.length > 0 && name === name.trim() && [...name].length <= 80 && !reserved.test(name);
}

function checkedSnapshot(value: unknown): ProjectSnapshot {
  if (!value || typeof value !== 'object') throw new Error('project_catalogue_invalid');
  const snapshot = value as ProjectSnapshot;
  if (!Number.isSafeInteger(snapshot.revision) || snapshot.revision < 0 || snapshot.revision > 1024 ||
      !Array.isArray(snapshot.projects) || snapshot.projects.length > 64 ||
      !snapshot.assignments || typeof snapshot.assignments !== 'object' || Array.isArray(snapshot.assignments)) {
    throw new Error('project_catalogue_invalid');
  }
  const ids = new Set<string>();
  for (const project of snapshot.projects) {
    if (!project || typeof project.id !== 'string' || !opaqueId.test(project.id) || ids.has(project.id) ||
        typeof project.name !== 'string' || !validName(project.name)) throw new Error('project_catalogue_invalid');
    ids.add(project.id);
  }
  const assignments = Object.entries(snapshot.assignments);
  if (assignments.length > 512 || assignments.some(([id, project]) => !opaqueId.test(id) || !ids.has(project))) {
    throw new Error('project_catalogue_invalid');
  }
  return snapshot;
}

function ProjectGlyph({ symbol }: { symbol: 'plus' | 'chevron' | 'sliders' }) {
  return <svg aria-hidden="true" width={symbol === 'chevron' ? 14 : 20} height={symbol === 'chevron' ? 14 : 20} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
    {symbol === 'plus' ? <path d="M12 3v18M3 12h18"/> : symbol === 'chevron' ? <path d="m6 9 6 6 6-6"/> :
      <><path d="M2 6h9m7 0h4M2 18h3m7 0h10"/><circle cx="14.5" cy="6" r="3"/><circle cx="8.5" cy="18" r="3"/></>}
  </svg>;
}

/** Project metadata never becomes Send context or changes the selected conversation. */
export function Projects({ bridge, ready, disabled, conversationId, children }: {
  bridge: Bridge;
  ready: boolean;
  disabled: boolean;
  conversationId?: string;
  children(projectConversationIds?: ReadonlySet<string>, catalogueEmpty?: boolean): ReactNode;
}) {
  const [snapshot, setSnapshot] = useState<ProjectSnapshot>();
  const [expanded, setExpanded] = useState(true);
  const [panel, setPanel] = useState<'create' | 'manage'>();
  const [selected, setSelected] = useState<string>();
  const [name, setName] = useState('');
  const [destination, setDestination] = useState('');
  const [pending, setPending] = useState(false);
  const [issue, setIssue] = useState('');
  const [notice, setNotice] = useState('');
  const id = useId();
  const toggle = useRef<HTMLButtonElement>(null);
  const plus = useRef<HTMLButtonElement>(null);
  const manage = useRef<HTMLButtonElement>(null);
  const nameField = useRef<HTMLInputElement>(null);
  const targetField = useRef<HTMLSelectElement>(null);
  const busy = useRef(false);
  const returnFocus = useRef<'create' | 'manage' | undefined>(undefined);
  const generation = useRef(0);
  const canList = ready && Boolean(bridge.listProjects);
  const currentSelection = snapshot?.projects.find(project => project.id === selected);
  const assigned = conversationId && snapshot ? snapshot.assignments[conversationId] ?? '' : '';
  const filtered = currentSelection && snapshot
    ? new Set(Object.entries(snapshot.assignments).filter(([, project]) => project === currentSelection.id).map(([conversation]) => conversation))
    : undefined;

  useEffect(() => {
    const revision = ++generation.current;
    if (!canList || !bridge.listProjects) {
      setSnapshot(undefined);
      setPanel(undefined);
      setIssue('');
      return;
    }
    setSnapshot(undefined);
    setPanel(undefined);
    setIssue('');
    void bridge.listProjects().then(checkedSnapshot).then(value => {
      if (revision === generation.current) setSnapshot(value);
    }).catch(() => {
      if (revision === generation.current) setIssue('Projects are unavailable on this Windows account.');
    });
    return () => { generation.current++; };
  }, [bridge, canList]);
  useEffect(() => {
    if (panel === 'create') nameField.current?.focus();
    if (panel === 'manage') targetField.current?.focus();
  }, [panel]);
  useEffect(() => {
    if (panel === 'manage') setDestination(assigned);
  }, [assigned, conversationId, panel]);

  useEffect(() => {
    if (panel || pending || !returnFocus.current) return;
    const target = returnFocus.current === 'manage' ? manage : plus;
    returnFocus.current = undefined;
    target.current?.focus();
  }, [panel, pending]);

  function closePanel() {
    returnFocus.current = panel;
    setPanel(undefined);
  }
  function openPanel(next: 'create' | 'manage') {
    if (panel === next) { closePanel(); return; }
    setIssue(''); setNotice(''); setName(''); setDestination(assigned); setPanel(next);
  }
  async function write(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!panel || !snapshot || disabled || busy.current) return;
    const proposed = name.trim();
    if (panel === 'create' && (!bridge.createProject || !validName(proposed))) return;
    if (panel === 'manage' && (!bridge.assignConversationProject || !conversationId || destination === assigned ||
        (destination && !snapshot.projects.some(project => project.id === destination)))) return;
    if (panel === 'create' && snapshot.projects.some(project => project.name.toLowerCase() === proposed.toLowerCase())) {
      setIssue('A project with that name already exists.'); return;
    }
    const token = generation.current;
    const projectIds = new Set(snapshot.projects.map(project => project.id));
    busy.current = true; setPending(true); setIssue(''); setNotice('');
    try {
      const raw = panel === 'create'
        ? await bridge.createProject!({ expectedRevision: snapshot.revision, name: proposed })
        : await bridge.assignConversationProject!({ expectedRevision: snapshot.revision, conversationId: conversationId!, projectId: destination || null });
      const updated = checkedSnapshot(raw);
      if (generation.current !== token) return;
      if (updated.revision !== snapshot.revision + 1) throw new Error('project_catalogue_invalid');
      if (panel === 'create') {
        const created = updated.projects.find(project => !projectIds.has(project.id) && project.name === proposed);
        if (!created) throw new Error('project_catalogue_invalid');
        setExpanded(true); setSelected(created.id); setNotice('Project created.');
      } else {
        if ((updated.assignments[conversationId!] ?? '') !== destination) throw new Error('project_catalogue_invalid');
        setNotice(destination ? 'Conversation assigned to project.' : 'Conversation moved out of the project.');
      }
      setSnapshot(updated); closePanel();
    } catch {
      if (generation.current === token) {
        setIssue('The project change could not be verified. Refresh the catalogue before trying again.');
        try {
          const current = checkedSnapshot(await bridge.listProjects?.());
          if (generation.current === token) setSnapshot(current);
        } catch {
          if (generation.current === token) { setSnapshot(undefined); setPanel(undefined); }
        }
      }
    } finally {
      busy.current = false; setPending(false);
    }
  }

  return <>
    <section className="projects-sidebar" aria-label="Projects">
      <div className="projects-heading">
        <button ref={toggle} type="button" className="projects-toggle" aria-expanded={expanded} aria-controls={id}
          onClick={() => setExpanded(value => !value)} disabled={disabled || pending}>Projects<span className={!expanded ? 'projects-chevron-collapsed' : ''}><ProjectGlyph symbol="chevron"/></span></button>
        <button ref={manage} type="button" className="icon-button project-icon" aria-label="Manage project assignment" title="Manage project assignment"
          aria-expanded={panel === 'manage'} disabled={disabled || pending || !snapshot?.projects.length || !bridge.assignConversationProject}
          onClick={() => openPanel('manage')}><ProjectGlyph symbol="sliders"/></button>
        <button ref={plus} type="button" className="icon-button project-icon" aria-label="Create project" title="Create project"
          aria-expanded={panel === 'create'} disabled={disabled || pending || !snapshot || snapshot.projects.length >= 64 || !bridge.createProject}
          onClick={() => openPanel('create')}><ProjectGlyph symbol="plus"/></button>
      </div>
      {panel && <form className="projects-form" aria-label={panel === 'create' ? 'New project' : 'Project assignment'} onSubmit={event => void write(event)}
        onKeyDown={event => { if (event.key === 'Escape' && !busy.current) { event.preventDefault(); event.stopPropagation(); closePanel(); } }}>
        {panel === 'create' ? <label>Project name<input ref={nameField} value={name} maxLength={160} disabled={disabled || pending}
          onChange={event => { setName(event.target.value); setIssue(''); }}/></label>
          : conversationId ? <label>Project for this conversation<select ref={targetField} value={destination} disabled={disabled || pending}
            onChange={event => setDestination(event.target.value)}><option value="">No project</option>
            {snapshot?.projects.map(project => <option key={project.id} value={project.id}>{project.name}</option>)}</select></label>
          : <p>Open or start a saved conversation to assign it to a project.</p>}
        <div className="projects-form-actions">
          <button type="button" className="secondary-action" onClick={closePanel} disabled={pending}>Cancel</button>
          <button type="submit" className="primary-action" disabled={disabled || pending || (panel === 'create' ? !validName(name.trim()) : !conversationId || destination === assigned)}>
            {panel === 'create' ? 'Create' : 'Save assignment'}
          </button>
        </div>
      </form>}
      {issue && <p role="alert" className="projects-line">{issue}</p>}
      {notice && <span className="visually-hidden" role="status">{notice}</span>}
      <div id={id} className="projects-items" hidden={!expanded}>
        {snapshot?.projects.length ? snapshot.projects.map(project => <button key={project.id} type="button" className="project-row" aria-pressed={currentSelection?.id === project.id}
          disabled={disabled || pending} title={project.name} onClick={() => setSelected(currentSelection?.id === project.id ? undefined : project.id)}>{project.name}</button>)
          : !issue && <p className="projects-line">{snapshot ? 'No projects' : canList ? 'Loading projects…' : 'Projects unavailable'}</p>}
      </div>
      {currentSelection && <button type="button" className="clear-project-filter" disabled={disabled || pending}
        onClick={() => { setSelected(undefined); toggle.current?.focus(); }}>Clear project filter</button>}
    </section>
    {children(filtered, snapshot?.projects.length === 0)}
  </>;
}
