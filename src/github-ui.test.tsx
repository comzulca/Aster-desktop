import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { GitHubAccount } from './GitHubAccount';
import { validGitHubFile, validGitHubRepositories, validGitHubStatus } from './github-ui';
import { fakeBridge } from './test-fixtures';
import type { GitHubStatus } from './types';

afterEach(() => cleanup());
const disconnected = (): GitHubStatus => ({ enabled: true, state: 'disconnected', account: null, repository: null, userCode: null, verificationUri: 'https://github.com/login/device', nextPollSeconds: 0, inFlight: false, mutationsAvailable: false });
const identity = { installation_id: 10, repository_id: 20, owner: 'synthetic-owner', name: 'fixture-repo' };
function fixture() {
  const fake = fakeBridge(); let status = disconnected();
  fake.bridge.getGitHubStatus = vi.fn(async () => structuredClone(status));
  fake.bridge.beginGitHubConnection = vi.fn(async () => { status = { ...status, state: 'authorising', userCode: 'ABCD-EFGH', nextPollSeconds: 5 }; });
  fake.bridge.pollGitHubConnection = vi.fn(async () => { status = { ...status, state: 'connected', userCode: null, nextPollSeconds: 0, account: { id: 30, login: 'synthetic-account' } }; });
  fake.bridge.cancelGitHubConnection = vi.fn(async () => { status = disconnected(); });
  fake.bridge.disconnectGitHub = vi.fn(async () => { status = disconnected(); });
  fake.bridge.listGitHubRepositories = vi.fn(async () => [{ identity, private: true, canRead: true, canWrite: true }]);
  fake.bridge.selectGitHubRepository = vi.fn(async input => { status = { ...status, repository: { ...identity, installation_id: input.installationId, repository_id: input.repositoryId } }; });
  fake.bridge.readGitHubFile = vi.fn(async input => ({ ...input, blobSha: 'a'.repeat(40), content: '<script>PRIVATE_REPOSITORY_CANARY</script>' }));
  return { ...fake, setStatus: (value: GitHubStatus) => { status = value; } };
}
describe('strict GitHub account renderer seams (T-090/T-091/T-096)', () => {
  it.each(['pending', 'failed', 'invalid', 'unavailable'] as const)('does not infer account absence from %s native status', async state => {
    const fake = fixture();
    if (state === 'pending') fake.bridge.getGitHubStatus = vi.fn(() => new Promise<GitHubStatus>(() => {}));
    if (state === 'failed') fake.bridge.getGitHubStatus = vi.fn(async () => { throw new Error('synthetic_status_failure'); });
    if (state === 'invalid') fake.bridge.getGitHubStatus = vi.fn(async () => ({ ...disconnected(), unexpected: true } as GitHubStatus));
    if (state === 'unavailable') delete fake.bridge.getGitHubStatus;
    render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>);
    if (state === 'failed' || state === 'invalid') await screen.findByText(/native connection status could not be verified/);
    if (state === 'unavailable') await screen.findByText(/account management is unavailable/);
    expect(screen.getByText(/Connection: unverified.*Active account status is unknown/)).toBeTruthy();
    expect(document.body.textContent).not.toMatch(/No authenticated account is active|No account was connected|No connection has been made|receipt reports no active authenticated account/);
    expect(fake.bridge.beginGitHubConnection).not.toHaveBeenCalled(); expect(fake.bridge.disconnectGitHub).not.toHaveBeenCalled();
  });
  it('does not infer disconnection when a known account operation cannot be verified', async () => {
    const fake = fixture(); fake.setStatus({ ...disconnected(), state: 'connected', account: { id: 30, login: 'synthetic-account' } });
    fake.bridge.disconnectGitHub = vi.fn(async () => { throw new Error('synthetic_operation_failure'); });
    render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>); await screen.findByText(/Authenticated account: synthetic-account/);
    fireEvent.click(screen.getByRole('button', { name: 'Disconnect in Windows' })); await screen.findByText(/native operation could not be verified/);
    expect(screen.getByText(/Connection: unverified.*Active account status is unknown/)).toBeTruthy();
    expect(document.body.textContent).not.toMatch(/receipt reports no active authenticated account|No authenticated account is active/);
    expect(fake.bridge.disconnectGitHub).toHaveBeenCalledOnce(); expect(fake.bridge.beginGitHubConnection).not.toHaveBeenCalled();
  });
  it('rejects unknown authority, endpoint, secrets, bad identity and impossible status', () => {
    const value = disconnected(); expect(validGitHubStatus(value)).toBe(true);
    expect(validGitHubStatus({ ...value, state: 'authorising', userCode: 'ABCD-EFGH', nextPollSeconds: 0 })).toBe(true);
    for (const patch of [{ accessToken: 'secret' }, { verificationUri: 'https://attacker.invalid/device' }, { mutationsAvailable: true }, { nextPollSeconds: -1 }, { nextPollSeconds: 901 }, { state: 'connected', account: null }, { state: 'authorising', userCode: null }, { account: { id: Number.MAX_SAFE_INTEGER + 1, login: 'synthetic' } }, { repository: identity }]) expect(validGitHubStatus({ ...value, ...patch })).toBe(false);
    expect(validGitHubRepositories([{ identity, private: true, canRead: true, canWrite: false }])).toBe(true);
    expect(validGitHubRepositories([{ identity, private: true, canRead: true, canWrite: false, approved: true }])).toBe(false);
    const file = { path: 'README.md', branch: 'main', blobSha: 'a'.repeat(40), content: 'Synthetic text' };
    expect(validGitHubFile(file, 'README.md', 'main')).toBe(true); expect(validGitHubFile({ ...file, content: 'x'.repeat(65537) }, 'README.md', 'main')).toBe(false);
    expect(validGitHubFile({ ...file, path: 'other.md' }, 'README.md', 'main')).toBe(false);
  });
  it('makes no unsolicited OAuth call, displays only the fixed page and cancels explicit authorisation', async () => {
    const fake = fixture(); render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>);
    await screen.findByText(/Connection: disconnected/); expect(fake.bridge.beginGitHubConnection).not.toHaveBeenCalled(); expect(fake.bridge.pollGitHubConnection).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Connect or change account in Windows' })); await screen.findByText('ABCD-EFGH');
    expect(screen.getByText('https://github.com/login/device')).toBeTruthy(); expect(screen.queryByRole('link')).toBeNull();
    expect(fake.bridge.pollGitHubConnection).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel authorisation' })); await screen.findByText('Authorisation was cancelled. Old callbacks cannot connect an account.');
    expect(fake.bridge.cancelGitHubConnection).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('drops late begin results after unmount and cancels its host session', async () => {
    const fake = fixture(); let resolveBegin: (() => void) | undefined;
    fake.bridge.beginGitHubConnection = vi.fn(() => new Promise<void>(resolve => { resolveBegin = resolve; }));
    const view = render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>); await screen.findByText(/Connection: disconnected/);
    fireEvent.click(screen.getByRole('button', { name: 'Connect or change account in Windows' })); view.unmount();
    await act(async () => { resolveBegin!(); });
    expect(fake.bridge.cancelGitHubConnection).toHaveBeenCalledOnce(); expect(fake.bridge.pollGitHubConnection).not.toHaveBeenCalled();
  });
  it('keeps selected private file content inert and outside every Meta request', async () => {
    const fake = fixture(); fake.setStatus({ ...disconnected(), state: 'connected', account: { id: 30, login: 'synthetic-account' } });
    render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>); await screen.findByText(/Authenticated account: synthetic-account/);
    fireEvent.click(screen.getByRole('button', { name: 'List authorised repositories' })); await screen.findByRole('button', { name: 'Select synthetic-owner/fixture-repo in Windows' });
    fireEvent.click(screen.getByRole('button', { name: 'Select synthetic-owner/fixture-repo in Windows' })); await screen.findByText(/Selected: synthetic-owner\/fixture-repo/);
    fireEvent.click(screen.getByRole('button', { name: 'Read file locally' })); await screen.findByText('<script>PRIVATE_REPOSITORY_CANARY</script>');
    expect(document.querySelector('.github-account script')).toBeNull(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(fake.bridge.readGitHubFile).toHaveBeenCalledWith({ path: 'README.md', branch: 'main' });
    fireEvent.click(screen.getByRole('button', { name: 'Disconnect in Windows' })); await waitFor(() => expect(screen.queryByText('<script>PRIVATE_REPOSITORY_CANARY</script>')).toBeNull());
    expect(fake.bridge.disconnectGitHub).toHaveBeenCalledOnce();
  });
  it('does not claim an account change when native disconnect review is declined', async () => {
    const fake = fixture(); fake.setStatus({ ...disconnected(), state: 'connected', account: { id: 30, login: 'synthetic-account' } });
    fake.bridge.disconnectGitHub = vi.fn(async () => {});
    render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>); await screen.findByText(/Authenticated account: synthetic-account/);
    fireEvent.click(screen.getByRole('button', { name: 'Disconnect in Windows' })); await screen.findByText(/native review did not confirm the requested change/);
    expect(screen.queryByText(/Aster forgot its local connection/)).toBeNull(); expect(screen.getByText(/Authenticated account: synthetic-account/)).toBeTruthy();
  });
  it('preserves an existing connection when a change-account review is declined and the popup closes', async () => {
    const fake = fixture(); fake.setStatus({ ...disconnected(), state: 'connected', account: { id: 30, login: 'synthetic-account' } });
    fake.bridge.beginGitHubConnection = vi.fn(async () => {});
    const onClose = vi.fn(); const view = render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={onClose}/>); await screen.findByText(/Authenticated account: synthetic-account/);
    fireEvent.click(screen.getByRole('button', { name: 'Connect or change account in Windows' })); await waitFor(() => expect((screen.getByRole('button', { name: 'Connect or change account in Windows' }) as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(screen.getByRole('button', { name: 'Close account popup' })); await waitFor(() => expect(onClose).toHaveBeenCalledOnce()); view.unmount();
    expect(fake.bridge.cancelGitHubConnection).not.toHaveBeenCalled(); expect(fake.bridge.pollGitHubConnection).not.toHaveBeenCalled();
  });
  it('allows exact native forgetting when no account was restored without claiming saved-token presence or deletion', async () => {
    const fake = fixture(); render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={() => {}}/>); await screen.findByText(/Connection: disconnected/);
    const button = screen.getByRole('button', { name: 'Forget saved connection in Windows' }); expect((button as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(button); await screen.findByText(/declined native review leaves saved credentials untouched/);
    expect(fake.bridge.disconnectGitHub).toHaveBeenCalledOnce(); expect(fake.bridge.beginGitHubConnection).not.toHaveBeenCalled(); expect(document.body.textContent).not.toMatch(/saved token exists|saved credentials were deleted/i);
  });
  it('cancels an owned repository-read operation on close exactly once and discards its late result', async () => {
    const fake = fixture(); fake.setStatus({ ...disconnected(), state: 'connected', account: { id: 30, login: 'synthetic-account' }, repository: identity });
    let resolveRead: ((value: { path: string; branch: string; blobSha: string; content: string }) => void) | undefined;
    fake.bridge.readGitHubFile = vi.fn(() => new Promise<{ path: string; branch: string; blobSha: string; content: string }>(resolve => { resolveRead = resolve; }));
    const onClose = vi.fn(); const view = render(<GitHubAccount bridge={fake.bridge} locked={false} onClose={onClose}/>); await screen.findByText(/Selected: synthetic-owner\/fixture-repo/);
    fireEvent.click(screen.getByRole('button', { name: 'Read file locally' })); await waitFor(() => expect(fake.bridge.readGitHubFile).toHaveBeenCalledOnce());
    fireEvent.click(screen.getByRole('button', { name: 'Close account popup' })); await waitFor(() => expect(onClose).toHaveBeenCalledOnce()); view.unmount();
    await act(async () => { resolveRead!({ path: 'README.md', branch: 'main', blobSha: 'a'.repeat(40), content: 'LATE_PRIVATE_READ_CANARY' }); });
    expect(fake.bridge.cancelGitHubConnection).toHaveBeenCalledOnce(); expect(document.body.textContent).not.toContain('LATE_PRIVATE_READ_CANARY');
  });
});
