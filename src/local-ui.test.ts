import { describe, expect, it } from 'vitest';
import { DraftMemory, editSelection, findMatches, menuGroups, shortcutAction } from './local-ui';

describe('v0.2.1 local-only menu/edit boundaries (T-066/T-070)', () => {
  it('maps only the included commands, exact groups and keyboard intents', () => {
    expect(menuGroups.File.flat().map(item => item.label)).toEqual(['New Chat', 'Quit Aster']);
    expect(menuGroups.Edit.flat().map(item => item.label)).toEqual(['Undo', 'Redo', 'Cut', 'Copy', 'Paste', 'Delete', 'Select All', 'Settings…']);
    expect(menuGroups.Help.flat().map(item => item.label)).toEqual(['Documentation', 'Keyboard Shortcuts', "What's New", 'Troubleshooting', 'System Status', 'About Aster']);
    expect(JSON.stringify(menuGroups)).not.toMatch(/Browser|Terminal|Open Folder|Update|Feedback|Temporary|Log Out/);
    expect(shortcutAction({ key: 'n', ctrlKey: true })).toBe('new');
    expect(shortcutAction({ key: 's', ctrlKey: true, shiftKey: true })).toBe('sidebar');
    expect(shortcutAction({ key: '{', ctrlKey: true, shiftKey: true })).toBe('previous');
    expect(shortcutAction({ key: '}', ctrlKey: true, shiftKey: true })).toBe('next');
    expect(shortcutAction({ key: 'F11' })).toBe('fullscreen');
    expect(shortcutAction({ key: 'n', ctrlKey: true, altKey: true })).toBeNull();
    expect(shortcutAction({ key: 'n', ctrlKey: true, isComposing: true })).toBeNull();
  });
  it('preserves bounded per-chat drafts and undo history without persisted approval', () => {
    const memory = new DraftMemory();
    memory.write('first', 'First draft'); memory.write('second', 'Second draft');
    expect(memory.read('first')).toBe('First draft');
    memory.write('first', 'Changed'); expect(memory.undo('first')).toBe('First draft');
    expect(memory.redo('first')).toBe('Changed');
    memory.write('first', 'New edit'); expect(memory.canRedo('first')).toBe(false);
    expect(memory.write('large', 'é'.repeat(16385))).toBe(false);
    for (let i = 0; i < 130; i++) memory.write(`c${i}`, 'Draft');
    expect(memory.size).toBeLessThanOrEqual(129);
    expect(memory.hasDrafts).toBe(true); memory.clear(); expect(memory.hasDrafts).toBe(false);
  });
  it('edits only a validated selected range and refuses over-limit replacement', () => {
    expect(editSelection('Alpha Beta', 6, 10, 'Gamma')).toEqual({ text: 'Alpha Gamma', caret: 11 });
    expect(editSelection('text', -1, 3, '')).toBeNull();
    expect(editSelection('text', 3, 2, '')).toBeNull();
    expect(editSelection('text', 0, 0, 'é'.repeat(16385))).toBeNull();
  });
  it('finds literal Unicode text at original indices and bounds highlight storage', () => {
    expect(findMatches('İx hello', 'hello')).toEqual({ count: 1, spans: [{ start: 3, end: 8 }] });
    expect(findMatches('İx hello', 'İ')).toEqual({ count: 1, spans: [{ start: 0, end: 1 }] });
    expect(findMatches('[x] .*', '[x]')).toEqual({ count: 1, spans: [{ start: 0, end: 3 }] });
    const many = findMatches('a'.repeat(2000), 'a'); expect(many.count).toBe(2000); expect(many.spans).toHaveLength(512);
  });
});
