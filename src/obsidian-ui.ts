export interface ObsidianStatus { selected: boolean; label: string }
export interface ObsidianNotes { notes: { relativePath: string }[]; limited: boolean }
export interface ObsidianReceipt { relativePath: string }
export interface ObsidianBridge {
  getObsidianStatus?(): Promise<ObsidianStatus>;
  selectObsidianVault?(): Promise<ObsidianStatus>;
  clearObsidianVault?(): Promise<ObsidianStatus>;
  listObsidianNotes?(input: { topic: string; query: string }): Promise<ObsidianNotes>;
  readObsidianNote?(input: { relativePath: string }): Promise<unknown>;
  createObsidianNote?(input: { topic: string; title: string; content: string }): Promise<ObsidianReceipt>;
  createObsidianTopic?(input: { topic: string }): Promise<ObsidianReceipt>;
}
function record(value: unknown, keys: string[]): value is Record<string, unknown> {
  return !!value && typeof value === 'object' && Object.keys(value).sort().join('|') === keys.sort().join('|');
}
export function component(value: string) {
  return !!value && Array.from(value).length <= 80 && value === value.trim() && !value.endsWith('.') &&
    !/[~<>:"/\\|?*\u0000-\u001f\u007f]/u.test(value) && !value.startsWith('.') && !/^[\s\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]|[\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/u.test(value) &&
    !/^(?:access_tokens?|credentials|secret)|\.(?:pem|key|p12|pfx|kdbx)$/iu.test(value) &&
    !/^(?:CON|PRN|AUX|NUL|CLOCK\$|CONIN\$|CONOUT\$|COM[0-9¹²³]|LPT[0-9¹²³])(?:[. ]|$)/iu.test(value);
}
export function validVaultStatus(value: unknown): value is ObsidianStatus {
  return record(value, ['selected', 'label']) && typeof value.selected === 'boolean' &&
    typeof value.label === 'string' && value.label.length <= 260 &&
    (value.selected ? value.label.length > 0 && !/[\\/\u0000-\u001f\u007f]/u.test(value.label) : value.label === '');
}
function matchesPath(value: unknown, topic: string, title?: string) {
  if (typeof value !== 'string') return false;
  const [folder, name, ...extra] = value.split('/');
  return extra.length === 0 && folder === topic && typeof name === 'string' && /\.md$/iu.test(name) &&
    component(name.slice(0, -3)) && (title === undefined || name === title + '.md');
}
export function validNotes(value: unknown, topic: string): value is ObsidianNotes {
  if (!record(value, ['notes', 'limited']) || !Array.isArray(value.notes) || value.notes.length > 3 ||
      typeof value.limited !== 'boolean') return false;
  const seen = new Set<string>();
  return value.notes.every(note => {
    if (!record(note, ['relativePath']) || typeof note.relativePath !== 'string' ||
        !matchesPath(note.relativePath, topic) || seen.has(note.relativePath.toLowerCase())) return false;
    seen.add(note.relativePath.toLowerCase()); return true;
  });
}
export function validVaultReceipt(value: unknown, topic: string, title?: string): value is ObsidianReceipt {
  return record(value, ['relativePath']) &&
    (title === undefined ? value.relativePath === topic : matchesPath(value.relativePath, topic, title));
}
