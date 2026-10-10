import { useEffect, useRef, useState } from 'react';
import type { Bridge, CustomizationDraft, CustomizationEntry, CustomizationSnapshot, GenerationBinding, GenerationFinished, PackageKind } from './types';
import { validCustomization, validCustomizationDraft, validCustomizationPackage } from './customization-ui';
import { scalarCount } from './personalization-ui';
import { utf8Bytes } from './policy';
import { GitHubAccount } from './GitHubAccount';
import { GenerateCustomization } from './GenerateCustomization';

const draftFor = (kind: PackageKind): CustomizationDraft => ({ kind, slug: '', name: '', description: '', instructions: '' });
export function Customization({ bridge, locked, credentialValidated = false, onGenerationBound, onGenerationFinished, onNavigationBlockedChange }: { bridge: Bridge; locked: boolean; credentialValidated?: boolean; onGenerationBound?: (binding: GenerationBinding) => void; onGenerationFinished?: (result: GenerationFinished) => void; onNavigationBlockedChange?: (blocked: boolean) => void }) {
  const [kind, setKind] = useState<PackageKind>('skill');
  const [snapshot, setSnapshot] = useState<CustomizationSnapshot | null>(null);
  const [search, setSearch] = useState('');
  const [draft, setDraft] = useState<CustomizationDraft | null>(null);
  const [readOnly, setReadOnly] = useState(false);
  const [accountOpen, setAccountOpen] = useState(false);
  const [generationOpen, setGenerationOpen] = useState(false);
  const [generationBusy, setGenerationBusy] = useState(false);
  const accountOpener = useRef<HTMLButtonElement | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('Loading the local catalogue…');
  const work = useRef(false);
  const mounted = useRef(false);
  const available = Boolean(bridge.getCustomization && bridge.getCustomizationPackage && bridge.saveCustomization && bridge.setCustomizationEnabled && bridge.importCustomization);
  const navigationBlocked = busy || generationOpen || accountOpen || Boolean(draft && !readOnly);
  useEffect(() => { onNavigationBlockedChange?.(navigationBlocked); }, [navigationBlocked, onNavigationBlockedChange]);
  useEffect(() => () => { onNavigationBlockedChange?.(false); }, [onNavigationBlockedChange]);
  useEffect(() => {
    let disposed = false; mounted.current = true;
    if (!available) { setMessage('The local catalogue is unavailable in this host. Existing entries and preferences are unverified; this view made no catalogue change.'); return () => { mounted.current = false; }; }
    void bridge.getCustomization!().then(value => {
      if (disposed) return;
      if (!validCustomization(value)) throw new Error('catalogue_invalid');
      setSnapshot(value); setMessage('');
    }).catch(() => { if (!disposed) setMessage('The protected local catalogue is unavailable. Existing entries were not overwritten.'); });
    return () => { disposed = true; mounted.current = false; };
  }, [bridge, available]);
  async function action(operation: () => Promise<unknown>, success: string, changed = true) {
    if (locked || generationOpen || accountOpen || work.current || !snapshot || !available) return;
    work.current = true; setBusy(true); setMessage('');
    try {
      await operation();
      const value: unknown = await bridge.getCustomization!();
      if (!validCustomization(value) || value.revision < snapshot.revision || (changed && value.revision <= snapshot.revision)) throw new Error('catalogue_receipt_invalid');
      if (mounted.current) { setSnapshot(value); setDraft(null); setMessage(value.revision === snapshot.revision ? 'No catalogue change was made.' : success); }
    } catch {
      if (mounted.current) { setSnapshot(null); setMessage('The catalogue change could not be verified. Nothing was retried; cancel any editor, then leave and reopen Customize to reload it.'); }
    } finally { work.current = false; if (mounted.current) setBusy(false); }
  }
  async function edit(entry: CustomizationEntry) {
    if (locked || navigationBlocked || work.current || !bridge.getCustomizationPackage || entry.availability === 'reference-unavailable') return;
    work.current = true; setBusy(true); setMessage('');
    try {
      const value: unknown = await bridge.getCustomizationPackage({ id: entry.id });
      if (!validCustomizationPackage(value) || value.id !== entry.id || value.kind !== entry.kind || value.entryRevision !== entry.entryRevision || value.readOnly !== entry.readOnly) throw new Error('package_invalid');
      if (mounted.current) { setReadOnly(value.readOnly); setDraft({ kind: value.kind, slug: value.slug, name: value.name, description: value.description, instructions: value.instructions }); }
    } catch { if (mounted.current) setMessage('This immutable package revision could not be loaded. Nothing was changed.'); }
    finally { work.current = false; if (mounted.current) setBusy(false); }
  }
  const disabled = locked || busy || generationBusy || accountOpen || !available || !snapshot;
  const entries = snapshot?.entries.filter(entry => entry.kind === kind && `${entry.name} ${entry.description}`.toLocaleLowerCase('en-GB').includes(search.toLocaleLowerCase('en-GB'))) ?? [];
  function changeCategory(value: PackageKind) {
    if (locked || navigationBlocked || work.current) return;
    setKind(value); setDraft(null); setSearch('');
  }
  return <section className="customization-page" aria-labelledby="customization-heading">
    <aside className="customization-subrail" aria-label="Customize navigation"><h2>Customize</h2><div className="customization-tabs" role="tablist" aria-label="Customization categories" aria-orientation="vertical">{([['plugin','Plugins'],['skill','Skills']] as const).map(([value,label]) => <button key={value} id={`customization-tab-${value}`} role="tab" aria-selected={kind === value} tabIndex={kind === value ? 0 : -1} aria-controls="customization-content" disabled={locked || navigationBlocked} onClick={() => changeCategory(value)} onKeyDown={event => { if (['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End'].includes(event.key)) { event.preventDefault(); const next = event.key === 'Home' ? 'plugin' : event.key === 'End' ? 'skill' : value === 'skill' ? 'plugin' : 'skill'; changeCategory(next); if (!locked && !navigationBlocked) requestAnimationFrame(() => document.getElementById(`customization-tab-${next}`)?.focus()); } }}><span aria-hidden="true">{value === 'skill' ? '◇' : '⊕'}</span>{label}</button>)}</div><p>Local instruction packages and the built-in GitHub connector only. No marketplace or third-party execution.</p></aside>
    <div id="customization-content" className="customization-content" role="tabpanel" aria-labelledby={`customization-tab-${kind}`}><header className="customization-header"><div><h1 id="customization-heading">{kind === 'skill' ? 'Skills' : 'Plugins'}</h1><p>{kind === 'skill' ? 'Manage bounded task instructions without granting host authority.' : 'Manage inert packages and the reviewed built-in GitHub connector.'}</p></div><label><span className="visually-hidden">Search {kind === 'skill' ? 'skills' : 'plugins'}</span><input type="search" maxLength={256} value={search} disabled={locked || busy || generationOpen || accountOpen} onChange={event => setSearch(event.target.value)} placeholder={`Search ${kind === 'skill' ? 'skills' : 'plugins'}`}/></label></header>
      <div className="settings-actions"><button className="primary-action" disabled={disabled || generationOpen || Boolean(draft)} onClick={() => { setReadOnly(false); setDraft(draftFor(kind)); }}>Add {kind}</button><button className="secondary-action" disabled={disabled || generationOpen || Boolean(draft)} onClick={() => void action(() => bridge.importCustomization!({ expectedRevision: snapshot!.revision }), 'The reviewed inert package was imported. No script or MCP server was executed.', false)}>Import package in Windows</button><button className="secondary-action" disabled={disabled || generationOpen || Boolean(draft) || !credentialValidated || !bridge.generateCustomization} title={bridge.generateCustomization ? credentialValidated ? 'Generate an eligible draft through a separate exact native review; saving stays explicit.' : 'Configure and validate the Meta key in Providers before generation.' : 'Muse-assisted generation is unavailable in this host.'} onClick={() => { setMessage(''); setGenerationOpen(true); }}>Generate with Muse Spark</button></div>
      <p className="customization-boundary">Imports use a bounded JSON package selected in Windows. Instructions are inert data; scripts, hooks, package installation and third-party MCP execution are unavailable. {!bridge.generateCustomization && 'Muse-assisted generation is not available in this host.'}</p>
      {generationOpen ? <GenerateCustomization bridge={bridge} kind={kind} locked={locked || busy || !credentialValidated} onBusyChange={setGenerationBusy} onBound={onGenerationBound} onFinished={onGenerationFinished} onClose={() => { setGenerationOpen(false); setGenerationBusy(false); }} onDraft={value => { setReadOnly(false); setDraft(value); setGenerationOpen(false); setGenerationBusy(false); setMessage('Muse returned a completed bounded draft. Review every field and choose Save package revision separately; nothing was saved or enabled.'); }}/> : draft ? <form className="package-editor" onSubmit={event => { event.preventDefault(); if (!readOnly && snapshot && validCustomizationDraft(draft)) void action(() => bridge.saveCustomization!({ expectedRevision: snapshot.revision, package: { ...draft } }), 'A new immutable package revision was saved. No execution authority was granted.'); }}>
        <h4>{readOnly ? 'View immutable package' : draft.name ? 'Edit package revision' : `Create ${kind}`}</h4>
        <label>Package identifier<input value={draft.slug} readOnly={readOnly} disabled={disabled} onChange={event => setDraft({ ...draft, slug: event.target.value })}/></label><p>3–64 lowercase letters, digits and single hyphens, starting with a letter.</p>
        <label>Name<input value={draft.name} readOnly={readOnly} disabled={disabled} onChange={event => setDraft({ ...draft, name: event.target.value })}/></label><p>{scalarCount(draft.name)} / 80 characters · {utf8Bytes(draft.name)} / 320 bytes</p>
        <label>Description<input value={draft.description} readOnly={readOnly} disabled={disabled} onChange={event => setDraft({ ...draft, description: event.target.value })}/></label><p>{scalarCount(draft.description)} / 320 characters · {utf8Bytes(draft.description)} / 1,280 bytes</p>
        <label>Instructions<textarea value={draft.instructions} readOnly={readOnly} rows={8} disabled={disabled} onChange={event => setDraft({ ...draft, instructions: event.target.value })}/></label><p>{scalarCount(draft.instructions)} / 12,000 characters · {utf8Bytes(draft.instructions)} / 48,000 bytes</p>
        <div className="settings-actions"><button className="secondary-action" type="button" disabled={busy} onClick={() => setDraft(null)}>{readOnly ? 'Close package view' : 'Cancel editor'}</button>{!readOnly && <button className="primary-action" type="submit" disabled={disabled || !validCustomizationDraft(draft)}>Save package revision</button>}</div>
      </form> : <div className="customization-catalogue"><h4>Local catalogue</h4>{entries.length ? <ul aria-label={`Local ${kind === 'skill' ? 'skills' : 'plugins'}`}>{entries.map(entry => <li className="package-card" key={entry.id}>
        <span className="package-emblem" aria-hidden="true">{entry.kind === 'skill' ? '◇' : '⊕'}</span><div className="package-copy"><strong>{entry.name}</strong><p>{entry.description}</p><small>{entry.availability === 'reference-unavailable' ? 'Owner reference not imported; no bundled redistribution claim.' : entry.availability === 'built-in-connector' ? 'Built-in connector preference; account connection is separate.' : 'Inert instructions only; no script or MCP execution.'}</small></div>
        <label className="package-switch"><span className="visually-hidden">Enable {entry.name}</span><input type="checkbox" role="switch" checked={entry.enabled} disabled={disabled || entry.availability === 'reference-unavailable'} onChange={event => { const enabled = event.target.checked; if (snapshot) void action(() => bridge.setCustomizationEnabled!({ expectedRevision: snapshot.revision, id: entry.id, enabled }), enabled ? 'The catalogue preference was enabled. It does not connect an account or grant host authority.' : 'The catalogue preference was disabled.'); }}/></label>
        {entry.availability !== 'reference-unavailable' && entry.availability !== 'built-in-connector' && <button className="secondary-action" disabled={disabled} onClick={() => void edit(entry)}>{entry.readOnly ? 'View' : 'Edit'} {entry.name}</button>}
        {entry.availability === 'built-in-connector' && <button className="secondary-action" disabled={disabled || !bridge.getGitHubStatus} title={bridge.getGitHubStatus ? 'Manage the dedicated Aster GitHub App account in Windows.' : 'A qualified host account-management route is not available yet.'} onClick={event => { accountOpener.current = event.currentTarget; setAccountOpen(true); }}>Manage account</button>}
      </li>)}</ul> : <p>{snapshot ? search ? 'No matching packages.' : 'No packages in this category.' : 'Catalogue not available.'}</p>}</div>}
      {message && <p className="inline-state" role="status">{message}</p>}
      {accountOpen && <GitHubAccount bridge={bridge} locked={locked} onClose={() => { setAccountOpen(false); requestAnimationFrame(() => { const opener = accountOpener.current; if (mounted.current && opener?.isConnected && !opener.disabled) opener.focus(); }); }}/>}
    </div>
  </section>;
}
