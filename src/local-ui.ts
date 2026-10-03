import { MAX_MESSAGE_BYTES, utf8Bytes } from './policy';

export type MenuName = 'File' | 'Edit' | 'View' | 'Help';
export type LocalAction = 'new' | 'quit' | 'undo' | 'redo' | 'cut' | 'copy' | 'paste' | 'delete' | 'select' | 'settings' | 'sidebar' | 'find' | 'search' | 'previous' | 'next' | 'zoomIn' | 'zoomOut' | 'zoomReset' | 'fullscreen' | 'documentation' | 'shortcuts' | 'news' | 'troubleshooting' | 'status' | 'about';
type Item = { action: LocalAction; label: string; shortcut?: string };
export const menuGroups: Record<MenuName, Item[][]> = {
  File: [[{ action: 'new', label: 'New Chat', shortcut: 'Ctrl+N' }], [{ action: 'quit', label: 'Quit Aster', shortcut: 'Ctrl+Q' }]],
  Edit: [[{ action: 'undo', label: 'Undo', shortcut: 'Ctrl+Z' }, { action: 'redo', label: 'Redo', shortcut: 'Ctrl+Y' }], [{ action: 'cut', label: 'Cut', shortcut: 'Ctrl+X' }, { action: 'copy', label: 'Copy', shortcut: 'Ctrl+C' }, { action: 'paste', label: 'Paste', shortcut: 'Ctrl+V' }, { action: 'delete', label: 'Delete' }], [{ action: 'select', label: 'Select All', shortcut: 'Ctrl+A' }], [{ action: 'settings', label: 'Settings…', shortcut: 'Ctrl+,' }]],
  View: [[{ action: 'sidebar', label: 'Toggle Sidebar', shortcut: 'Ctrl+Shift+S' }], [{ action: 'find', label: 'Find…', shortcut: 'Ctrl+F' }, { action: 'search', label: 'Search Conversations…', shortcut: 'Ctrl+K' }], [{ action: 'previous', label: 'Previous Chat', shortcut: 'Ctrl+Shift+[' }, { action: 'next', label: 'Next Chat', shortcut: 'Ctrl+Shift+]' }], [{ action: 'zoomIn', label: 'Zoom In', shortcut: 'Ctrl++' }, { action: 'zoomOut', label: 'Zoom Out', shortcut: 'Ctrl+-' }, { action: 'zoomReset', label: 'Actual Size', shortcut: 'Ctrl+0' }], [{ action: 'fullscreen', label: 'Toggle Full Screen', shortcut: 'F11' }]],
  Help: [[{ action: 'documentation', label: 'Documentation' }, { action: 'shortcuts', label: 'Keyboard Shortcuts', shortcut: 'Ctrl+/' }, { action: 'news', label: "What's New" }], [{ action: 'troubleshooting', label: 'Troubleshooting' }, { action: 'status', label: 'System Status' }], [{ action: 'about', label: 'About Aster' }]],
};
export function shortcutAction(event: { key: string; ctrlKey?: boolean; metaKey?: boolean; shiftKey?: boolean; altKey?: boolean; isComposing?: boolean }): LocalAction | null {
  if (event.altKey || event.isComposing) return null;
  if (event.key === 'F11') return 'fullscreen';
  if (!event.ctrlKey && !event.metaKey) return null;
  const key = event.key.toLowerCase();
  if (event.shiftKey) return ({ s: 'sidebar', '[': 'previous', '{': 'previous', ']': 'next', '}': 'next', '=': 'zoomIn', '+': 'zoomIn' } as const)[key as 's'] ?? null;
  return ({ n: 'new', q: 'quit', z: 'undo', y: 'redo', x: 'cut', c: 'copy', v: 'paste', a: 'select', ',': 'settings', f: 'find', k: 'search', '=': 'zoomIn', '+': 'zoomIn', '-': 'zoomOut', '0': 'zoomReset', '/': 'shortcuts' } as const)[key as 'n'] ?? null;
}
type DraftHistory = { text: string; before: string[]; after: string[] };
export class DraftMemory {
  private records = new Map<string, DraftHistory>();
  get size() { return this.records.size; }
  get hasDrafts() { return [...this.records.values()].some(item => item.text.length > 0); }
  read(key: string) { return this.records.get(key)?.text ?? ''; }
  canUndo(key: string) { return Boolean(this.records.get(key)?.before.length); }
  canRedo(key: string) { return Boolean(this.records.get(key)?.after.length); }
  private trim(stack: string[]) { while (stack.length > 16 || stack.reduce((sum, value) => sum + utf8Bytes(value), 0) > 65536) stack.shift(); }
  write(key: string, text: string): boolean {
    if (utf8Bytes(text) > MAX_MESSAGE_BYTES || (!this.records.has(key) && this.records.size >= 129)) return false;
    const record = this.records.get(key) ?? { text: '', before: [], after: [] };
    if (record.text !== text) { record.before.push(record.text); this.trim(record.before); record.after = []; record.text = text; }
    this.records.set(key, record); return true;
  }
  undo(key: string) { const item = this.records.get(key); if (!item?.before.length) return this.read(key); item.after.push(item.text); this.trim(item.after); item.text = item.before.pop()!; return item.text; }
  redo(key: string) { const item = this.records.get(key); if (!item?.after.length) return this.read(key); item.before.push(item.text); this.trim(item.before); item.text = item.after.pop()!; return item.text; }
  forget(key: string) { this.records.delete(key); }
  clear() { this.records.clear(); }
}
export function editSelection(text: string, start: number, end: number, replacement: string): { text: string; caret: number } | null {
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || end < start || end > text.length || replacement.length > MAX_MESSAGE_BYTES) return null;
  const result = text.slice(0, start) + replacement + text.slice(end);
  return utf8Bytes(result) <= MAX_MESSAGE_BYTES ? { text: result, caret: start + replacement.length } : null;
}
export function findMatches(text: string, query: string): { count: number; spans: { start: number; end: number }[] } {
  const spans: { start: number; end: number }[] = []; let count = 0;
  if (!query || query.length > 256) return { count, spans };
  const literal = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const expression = new RegExp(literal, 'giu');
  for (let match = expression.exec(text); match; match = expression.exec(text)) { count++; if (spans.length < 512) spans.push({ start: match.index, end: match.index + match[0].length }); }
  return { count, spans };
}
export const helpTitles = { documentation: 'Documentation', shortcuts: 'Keyboard Shortcuts', news: "What's New", troubleshooting: 'Troubleshooting', status: 'System Status', about: 'About Aster' } as const;
export type HelpPage = keyof typeof helpTitles;
