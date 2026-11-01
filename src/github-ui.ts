import type { GitHubFile, GitHubRepository, GitHubRepositoryIdentity, GitHubStatus } from './types';
import { utf8Bytes } from './policy';
import { wellFormedText } from './personalization-ui';
const record = (value: unknown): value is Record<string, unknown> => Boolean(value) && typeof value === 'object' && !Array.isArray(value);
const exact = (value: Record<string, unknown>, keys: string[]) => Object.keys(value).sort().join('|') === [...keys].sort().join('|');
const identity = (value: unknown) => typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
const login = (value: unknown) => typeof value === 'string' && /^[A-Za-z0-9][A-Za-z0-9-]{0,38}$/.test(value);
export function validGitHubRepositoryIdentity(value: unknown): value is GitHubRepositoryIdentity {
  return record(value) && exact(value, ['installation_id','repository_id','owner','name']) && identity(value.installation_id) && identity(value.repository_id) && login(value.owner) && typeof value.name === 'string' && /^[A-Za-z0-9._-]{1,100}$/.test(value.name) && !['.','..'].includes(value.name);
}
export function validGitHubStatus(value: unknown): value is GitHubStatus {
  if (!record(value) || !exact(value, ['enabled','state','account','repository','userCode','verificationUri','nextPollSeconds','inFlight','mutationsAvailable']) ||
    typeof value.enabled !== 'boolean' || !['disabled','disconnected','authorising','connected','reconnect_required'].includes(String(value.state)) ||
    typeof value.inFlight !== 'boolean' || value.mutationsAvailable !== false || value.verificationUri !== 'https://github.com/login/device' ||
    !Number.isSafeInteger(value.nextPollSeconds) || (value.nextPollSeconds as number) < 0 || (value.nextPollSeconds as number) > 900 ||
    !(value.account === null || record(value.account) && exact(value.account, ['id','login']) && identity(value.account.id) && login(value.account.login)) ||
    !(value.repository === null || validGitHubRepositoryIdentity(value.repository)) ||
    !(value.userCode === null || typeof value.userCode === 'string' && /^[A-Z0-9]{4}-[A-Z0-9]{4}$/.test(value.userCode))) return false;
  if (value.state === 'connected' && (!value.enabled || value.account === null || value.userCode !== null)) return false;
  // Zero means the host's next interval is already due; it is not authority to
  // bypass the host timer. The UI still schedules no faster than once a second.
  if (value.state === 'authorising' && (!value.enabled || value.userCode === null || value.account !== null)) return false;
  if (value.repository !== null && value.state !== 'connected') return false;
  return true;
}
export function validGitHubRepositories(value: unknown): value is GitHubRepository[] {
  return Array.isArray(value) && value.length <= 1000 && value.every(item => record(item) && exact(item, ['identity','private','canRead','canWrite']) && validGitHubRepositoryIdentity(item.identity) && ['private','canRead','canWrite'].every(key => typeof item[key] === 'boolean')) &&
    new Set(value.map(item => `${item.identity.installation_id}:${item.identity.repository_id}`)).size === value.length;
}
export function validGitHubFile(value: unknown, path: string, branch: string): value is GitHubFile {
  return record(value) && exact(value, ['path','branch','blobSha','content']) && value.path === path && value.branch === branch && typeof value.blobSha === 'string' && /^[a-f0-9]{40}$/.test(value.blobSha) && typeof value.content === 'string' && wellFormedText(value.content) && utf8Bytes(value.content) <= 65536;
}
