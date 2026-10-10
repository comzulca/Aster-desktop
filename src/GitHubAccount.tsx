import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import type { Bridge, GitHubFile, GitHubRepository, GitHubStatus } from './types';
import { validGitHubFile, validGitHubRepositories, validGitHubStatus } from './github-ui';

export function GitHubAccount({ bridge, locked, onClose }: { bridge: Bridge; locked: boolean; onClose: () => void }) {
  const [status, setStatus] = useState<GitHubStatus | null>(null);
  const [repositories, setRepositories] = useState<GitHubRepository[]>([]);
  const [file, setFile] = useState<GitHubFile | null>(null);
  const [path, setPath] = useState('README.md');
  const [branch, setBranch] = useState('main');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('Reading the native connection status…');
  const mounted = useRef(false);
  const epoch = useRef(0);
  const authorising = useRef(false);
  const inFlight = useRef(false);
  const cancelIssued = useRef(false);
  const timer = useRef<ReturnType<typeof window.setTimeout> | null>(null);
  const region = useRef<HTMLElement>(null);
  const available = Boolean(bridge.getGitHubStatus && bridge.beginGitHubConnection && bridge.pollGitHubConnection && bridge.cancelGitHubConnection && bridge.disconnectGitHub && bridge.listGitHubRepositories && bridge.selectGitHubRepository && bridge.readGitHubFile);
  function stopPolling() { epoch.current++; if (timer.current !== null) window.clearTimeout(timer.current); timer.current = null; authorising.current = false; }
  function cancelOwned(): Promise<void> {
    const owned = authorising.current || inFlight.current;
    stopPolling();
    if (!owned || cancelIssued.current) return Promise.resolve();
    cancelIssued.current = true;
    return bridge.cancelGitHubConnection?.().then(() => {}).catch(() => {}) ?? Promise.resolve();
  }
  async function readStatus(): Promise<GitHubStatus> {
    const value: unknown = await bridge.getGitHubStatus!();
    if (!validGitHubStatus(value)) throw new Error('github_status_invalid');
    return value;
  }
  useEffect(() => {
    mounted.current = true; let disposed = false;
    region.current?.querySelector<HTMLButtonElement>('button')?.focus();
    if (!available) setMessage('GitHub account management is unavailable in this host. Existing account status is unknown; this view made no connection change.');
    else void readStatus().then(value => { if (!disposed) { setStatus(value); setMessage(''); } }).catch(() => { if (!disposed) setMessage('The native connection status could not be verified. Existing account status is unknown; this view made no connection change.'); });
    return () => { disposed = true; mounted.current = false; void cancelOwned(); };
  }, [bridge, available]);
  async function poll(sequence: number) {
    if (!mounted.current || !authorising.current || epoch.current !== sequence || inFlight.current || !available) return;
    inFlight.current = true; cancelIssued.current = false; setBusy(true);
    try {
      await bridge.pollGitHubConnection!();
      const value = await readStatus();
      if (!mounted.current || epoch.current !== sequence) return;
      setStatus(value);
      if (value.state === 'authorising') timer.current = window.setTimeout(() => void poll(sequence), Math.max(1, value.nextPollSeconds) * 1000);
      else { authorising.current = false; setMessage(value.state === 'connected' ? 'The native host confirmed this account. Choose a repository separately.' : 'Authorisation ended without an active account. Nothing was retried.'); }
    } catch { if (mounted.current && epoch.current === sequence) { stopPolling(); cancelIssued.current = true; void bridge.cancelGitHubConnection?.().catch(() => {}); setStatus(null); setMessage('Authorisation could not be verified. Polling stopped; nothing was retried. Reconnect deliberately.'); } }
    finally { inFlight.current = false; if (mounted.current) setBusy(false); }
  }
  async function action(operation: () => Promise<unknown>, thenPoll = false, success = '', confirmed?: (value: GitHubStatus) => boolean) {
    if (locked || inFlight.current || !available) return;
    stopPolling(); const sequence = epoch.current; authorising.current = thenPoll; inFlight.current = true; cancelIssued.current = false; setBusy(true); setMessage(''); setRepositories([]); setFile(null);
    try {
      await operation(); const value = await readStatus();
      if (!mounted.current || epoch.current !== sequence) return;
      setStatus(value); setMessage(confirmed && !confirmed(value) ? 'The native review did not confirm the requested change. The current account status is shown; no change is assumed.' : success);
      if (thenPoll && value.state === 'authorising') { authorising.current = true; timer.current = window.setTimeout(() => void poll(sequence), Math.max(1, value.nextPollSeconds) * 1000); }
      else authorising.current = false;
    } catch { if (mounted.current && epoch.current === sequence) { stopPolling(); if (thenPoll) { cancelIssued.current = true; void bridge.cancelGitHubConnection?.().catch(() => {}); } setStatus(null); setMessage('The native operation could not be verified. Nothing was retried; no account change is assumed.'); } }
    finally { inFlight.current = false; if (mounted.current) setBusy(false); }
  }
  async function listRepositories() {
    if (locked || inFlight.current || status?.state !== 'connected' || !bridge.listGitHubRepositories) return;
    const sequence = epoch.current; inFlight.current = true; cancelIssued.current = false; setBusy(true); setMessage('');
    try { const value: unknown = await bridge.listGitHubRepositories(); if (!validGitHubRepositories(value)) throw new Error('invalid_repositories'); if (mounted.current && epoch.current === sequence) setRepositories(value); }
    catch { if (mounted.current && epoch.current === sequence) setMessage('Repository access could not be verified. No repository was selected.'); }
    finally { inFlight.current = false; if (mounted.current) setBusy(false); }
  }
  async function readFile() {
    if (locked || inFlight.current || !status?.repository || !bridge.readGitHubFile || !path || !branch || path.length > 512 || branch.length > 256) return;
    const sequence = epoch.current, expectedPath = path, expectedBranch = branch; inFlight.current = true; cancelIssued.current = false; setBusy(true); setMessage(''); setFile(null);
    try { const value: unknown = await bridge.readGitHubFile({ path: expectedPath, branch: expectedBranch }); if (!validGitHubFile(value, expectedPath, expectedBranch)) throw new Error('invalid_file'); if (mounted.current && epoch.current === sequence) setFile(value); }
    catch { if (mounted.current && epoch.current === sequence) setMessage('The bounded file read was not admitted or verified. No content was sent to Meta.'); }
    finally { inFlight.current = false; if (mounted.current) setBusy(false); }
  }
  async function close() { await cancelOwned(); onClose(); }
  async function cancelAuthorisation() {
    stopPolling(); cancelIssued.current = true; setStatus(null); setMessage('Stopping this authorisation…');
    try { await bridge.cancelGitHubConnection!(); const value = await readStatus(); if (mounted.current) { setStatus(value); setMessage('Authorisation was cancelled. Old callbacks cannot connect an account.'); } }
    catch { if (mounted.current) setMessage('Cancellation could not be verified. Reconnect deliberately; old UI callbacks are discarded.'); }
  }
  function keys(event: KeyboardEvent<HTMLElement>) {
    event.stopPropagation();
    if (event.key === 'Escape') { event.preventDefault(); void close(); }
    if (event.key !== 'Tab') return;
    const controls = [...(region.current?.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled)') ?? [])];
    if (event.shiftKey && document.activeElement === controls[0]) { event.preventDefault(); controls.at(-1)?.focus(); }
    else if (!event.shiftKey && document.activeElement === controls.at(-1)) { event.preventDefault(); controls[0]?.focus(); }
  }
  const disabled = locked || busy || !available;
  return <div className="github-backdrop"><section ref={region} className="github-account" role="dialog" aria-modal="true" aria-labelledby="github-account-title" onKeyDown={keys}>
    <header><h3 id="github-account-title">GitHub account</h3><button className="secondary-action" onClick={() => void close()}>Close account popup</button></header>
    <p>Aster Desktop Engineering uses its own GitHub App, not Codex's connection. No private repository or personal profile is automatically sent to Meta.</p>
    <p>Connection: {status?.state.replaceAll('_', ' ') ?? 'unverified'}. {status?.account ? `Authenticated account: ${status.account.login}.` : status ? 'The captured host receipt reports no active authenticated account.' : 'Active account status is unknown.'}</p>
    {!status?.enabled && <p>Enable the GitHub slider in Plugins before connecting. Enabling a preference does not itself connect an account.</p>}
    <div className="settings-actions"><button className="primary-action" disabled={disabled || !status?.enabled || status.state === 'authorising'} onClick={() => void action(() => bridge.beginGitHubConnection!(), true)}>Connect or change account in Windows</button><button className="secondary-action" disabled={disabled || !status?.enabled || status.state === 'authorising'} onClick={() => void action(() => bridge.disconnectGitHub!(), false, 'The host reports no active local account. Remote GitHub authorisation is unchanged; a declined native review leaves saved credentials untouched.', value => value.account === null && value.state !== 'connected')}>{status?.account ? 'Disconnect in Windows' : 'Forget saved connection in Windows'}</button>{status?.state === 'authorising' && <button className="secondary-action" disabled={locked} onClick={() => void cancelAuthorisation()}>Cancel authorisation</button>}</div>
    {status?.state === 'authorising' && <div className="github-device"><p>Visit this exact page manually: <code>https://github.com/login/device</code></p><p>Enter only the code you just requested in Aster:</p><strong>{status.userCode}</strong><p>Check the App and account in GitHub before authorising. Never approve unsolicited device codes. The host controls expiry and polling.</p></div>}
    {status?.state === 'connected' && <><button className="secondary-action" disabled={disabled} onClick={() => void listRepositories()}>List authorised repositories</button>{repositories.length > 0 && <ul className="github-repositories">{repositories.map(repository => <li key={`${repository.identity.installation_id}:${repository.identity.repository_id}`}><span>{repository.identity.owner}/{repository.identity.name} · {repository.private ? 'Private' : 'Public'} · {repository.canWrite ? 'Write ceiling' : 'Read ceiling'}</span><button className="secondary-action" disabled={disabled || !repository.canRead} onClick={() => void action(() => bridge.selectGitHubRepository!({ installationId: repository.identity.installation_id, repositoryId: repository.identity.repository_id }), false, '', value => value.repository?.installation_id === repository.identity.installation_id && value.repository?.repository_id === repository.identity.repository_id)}>Select {repository.identity.owner}/{repository.identity.name} in Windows</button></li>)}</ul>}
      {status.repository && <section className="github-file"><h4>Local repository reader</h4><p>Selected: {status.repository.owner}/{status.repository.name}. Reading does not approve provider disclosure or a write.</p><label>File path<input value={path} maxLength={512} disabled={disabled} onChange={event => setPath(event.target.value)}/></label><label>Branch<input value={branch} maxLength={256} disabled={disabled} onChange={event => setBranch(event.target.value)}/></label><button className="secondary-action" disabled={disabled || !path || !branch} onClick={() => void readFile()}>Read file locally</button>{file && <><p>Blob SHA: <code>{file.blobSha}</code></p><pre>{file.content}</pre></>}</section>}
    </>}
    <p>File changes, issues, pull requests and merges are unavailable in this implementation slice. No mutation is silently substituted or automatically retried.</p>
    <p>Disconnect forgets Aster's local token custody; revoke the App separately in your GitHub account settings if required.</p>
    {message && <p className="inline-state" role="status">{message}</p>}
  </section></div>;
}
