import { useEffect, useId, useRef, useState } from 'react';
import { component, validNotes, validVaultReceipt, validVaultStatus } from './obsidian-ui';
import type { ObsidianBridge, ObsidianNotes, ObsidianStatus } from './obsidian-ui';

const messages: Record<string, string> = {
  cancelled: 'No change was made.',
  obsidian_not_selected: 'Choose a Markdown vault folder first.',
  obsidian_invalid: 'Use a supported local folder and portable topic or note name.',
  obsidian_limit: 'This vault operation exceeds its local limits. Narrow the lookup or shorten the note.',
  obsidian_conflict: 'That name already exists, or the vault changed. Nothing was overwritten.',
  obsidian_topic_missing: 'Create the topic folder in Windows before adding a note.',
  obsidian_verify_note: 'Aster could not verify the change. Inspect the local folder before trying again.',
  obsidian_io_error: 'Windows could not complete the local operation. Check the folder and try again.',
  obsidian_unavailable: 'Aster_obsidian is unavailable in this host. No vault access was granted.',
};
function message(error: unknown) {
  const code = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  return Object.hasOwn(messages, code) ? messages[code] : 'Aster could not verify the vault operation.';
}

export function VaultSettings({ bridge, locked, onBusyChange }: {
  bridge: ObsidianBridge; locked: boolean; onBusyChange?: (busy: boolean) => void;
}) {
  const id = useId();
  const [status, setStatus] = useState<ObsidianStatus | null>(null);
  const [notices, setNotices] = useState('Checking for a local vault…');
  const [topic, setTopic] = useState('');
  const [query, setQuery] = useState('');
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [notes, setNotes] = useState<ObsidianNotes | null>(null);
  const [busy, setBusy] = useState(false);
  const [checking, setChecking] = useState(true);
  const session = useRef(0);
  const work = useRef(false);
  const enabled = Boolean(bridge.getObsidianStatus && bridge.selectObsidianVault &&
    bridge.clearObsidianVault && bridge.listObsidianNotes && bridge.readObsidianNote &&
    bridge.createObsidianNote && bridge.createObsidianTopic);
  function updateStatus(value: unknown) {
    if (!validVaultStatus(value)) throw new Error('invalid_vault_status');
    setStatus(value); setNotes(null); setTopic(''); setQuery(''); setTitle(''); setContent('');
    setNotices(value.selected ? 'Vault connected for this session.' : 'No vault is connected.');
  }
  useEffect(() => {
    const current = ++session.current;
    work.current = false;
    setBusy(false);
    setStatus(null); setNotes(null); setTopic(''); setQuery(''); setTitle(''); setContent('');
    setChecking(enabled);
    setNotices(enabled ? 'Checking for a local vault…' : messages.obsidian_unavailable);
    if (enabled) void bridge.getObsidianStatus!().then(value => {
      if (session.current === current) updateStatus(value);
    }).catch(error => {
      if (session.current === current) setNotices(message(error));
    }).finally(() => {
      if (session.current === current) setChecking(false);
    });
    return () => { session.current++; };
  }, [bridge, enabled]);
  useEffect(() => () => onBusyChange?.(false), [bridge, onBusyChange]);
  async function run<T>(operation: () => Promise<T>, accept: (value: T) => void) {
    if (!enabled || checking || locked || work.current) return;
    const current = session.current;
    work.current = true; onBusyChange?.(true); setBusy(true); setNotices('');
    try {
      const result = await operation();
      if (session.current === current) accept(result);
    } catch (error) {
      if (session.current === current) setNotices(message(error));
    } finally {
      if (session.current === current) {
        work.current = false; onBusyChange?.(false); setBusy(false);
      }
    }
  }
  const blocked = !enabled || checking || locked || busy;
  const topicValid = component(topic);
  const queryValid = new TextEncoder().encode(query).byteLength <= 256;
  const noteValid = topicValid && component(title) && !!content.trim() &&
    new TextEncoder().encode(content).byteLength <= 16_384 && content.trim().split(/\s+/u).length <= 1200;
  return <section className="settings-section obsidian-page" aria-labelledby={id + '-heading'}>
    <h3 id={id + '-heading'}>Aster_obsidian</h3>
    <p>Work with a single local Markdown folder. Note contents stay out of chat and are never sent to Meta from this workspace.</p>
    <p>{status?.selected ? 'Selected vault: ' + status.label : 'Choose a folder in Windows. Reconnect it after restarting Aster.'}</p>
    <div className="settings-actions">
      <button className="secondary-action" disabled={blocked} onClick={() => void run(() => bridge.selectObsidianVault!(), updateStatus)}>Choose vault in Windows</button>
      {status?.selected && <button className="secondary-action" disabled={blocked} onClick={() => void run(() => bridge.clearObsidianVault!(), updateStatus)}>Disconnect vault</button>}
    </div>
    {status?.selected && <>
      <label htmlFor={id + '-topic'}>Topic folder</label>
      <input id={id + '-topic'} value={topic} disabled={blocked} maxLength={160} onChange={event => { setTopic(event.target.value); setNotes(null); }}/>
      <label htmlFor={id + '-query'}>Filter note titles (optional)</label>
      <input id={id + '-query'} value={query} disabled={blocked} maxLength={160} onChange={event => { setQuery(event.target.value); setNotes(null); }}/>
      <div className="settings-actions">
        <button className="secondary-action" disabled={blocked || !topicValid || !queryValid} onClick={() => {
          setNotes(null);
          void run(() => bridge.listObsidianNotes!({ topic, query }), value => {
            if (!validNotes(value, topic)) throw new Error('invalid_vault_listing');
            setNotes(value);
          });
        }}>Find local notes</button>
        <button className="secondary-action" disabled={blocked || !topicValid} onClick={() => void run(() => bridge.createObsidianTopic!({ topic }), value => {
          if (!validVaultReceipt(value, topic)) throw new Error('invalid_vault_receipt');
          setNotices('Created local topic: ' + value.relativePath);
        })}>Review new topic in Windows</button>
      </div>
      {notes && (notes.notes.length ? <ul aria-label="Local note results">{notes.notes.map(({ relativePath }) => <li key={relativePath}>
        <button className="secondary-action" disabled={blocked} aria-label={'View ' + relativePath + ' in Windows'} onClick={() => void run(() => bridge.readObsidianNote!({ relativePath }), value => {
          if (value !== null) throw new Error('invalid_vault_view_receipt');
          setNotices('Closed the local note viewer.');
        })}>{relativePath}</button>
      </li>)}</ul> : <p>{notes.limited ? 'No matches in the entries checked.' : 'No matching notes in this topic.'}</p>)}
      {notes?.limited && <p>The lookup is limited to 128 directory entries and three results. A title filter only applies within the entries checked.</p>}
      <h4>Create a Markdown note</h4>
      <p>Windows shows the proposed path and contents for confirmation. Existing notes are not overwritten.</p>
      <label htmlFor={id + '-title'}>New note title (without .md)</label>
      <input id={id + '-title'} value={title} disabled={blocked} maxLength={160} onChange={event => setTitle(event.target.value)}/>
      <label htmlFor={id + '-content'}>New note contents</label>
      <textarea id={id + '-content'} value={content} disabled={blocked} rows={5} maxLength={16_384} onChange={event => setContent(event.target.value)}/>
      <div className="settings-actions"><button className="primary-action" disabled={blocked || !noteValid} onClick={() => void run(() => bridge.createObsidianNote!({ topic, title, content }), value => {
        if (!validVaultReceipt(value, topic, title)) throw new Error('invalid_vault_receipt');
        setTitle(''); setContent(''); setNotes(null); setNotices('Saved locally: ' + value.relativePath);
      })}>Review new note in Windows</button></div>
    </>}
    {notices && <p className="inline-state" role="status">{notices}</p>}
  </section>;
}
