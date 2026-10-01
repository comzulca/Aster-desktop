import { describe, expect, it } from 'vitest';
import { safeRelativePath, validStatus, validStreamEvent } from './policy';
import { DEFAULT_MODEL, MUSE_MODEL } from './types';

describe('untrusted host boundaries in the renderer', () => {
  it('fails closed for unknown model catalogs and policy versions', () => {
    const value = { policyVersion: 2, configured: true, defaultModel: DEFAULT_MODEL,
      models: [{ id: DEFAULT_MODEL, label: 'Nemotron' }, { id: MUSE_MODEL, label: 'Muse' }],
      activeRequest: false, projectSelected: false, projectLabel: null,
      capabilities: { projectRead: false, proposals: false, applyNewFile: false, applyExistingFile: false } };
    expect(validStatus(value)).toBe(true);
    expect(validStatus({ ...value, policyVersion: 1 })).toBe(false);
    expect(validStatus({ ...value, models: [...value.models, { id: 'evil', label: 'Injected' }] })).toBe(false);
    expect(validStatus({ ...value, defaultModel: MUSE_MODEL })).toBe(false);
    expect(validStatus({ ...value, capabilities: { ...value.capabilities, projectRead: true } })).toBe(true);
    expect(validStatus({ ...value, capabilities: { ...value.capabilities, applyExistingFile: true } })).toBe(false);
    expect(validStatus({ ...value, capabilities: { ...value.capabilities, applyNewFile: true } })).toBe(false);
  });

  it('treats unknown/malformed stream events and unsafe paths as inert', () => {
    expect(validStreamEvent({ requestId: 'x', conversationId: 'c', type: 'content', text: 'ok' })).toBe(true);
    expect(validStreamEvent({ requestId: 'x', conversationId: 'c', type: 'commit', text: 'apply=true' })).toBe(false);
    expect(validStreamEvent({ requestId: 'x', conversationId: 'c', type: 'terminal', status: 'verified' })).toBe(false);
    for (const bad of ['../secret', '/absolute', '\\\\server\\share', 'C:/test', 'a//b', '.git/../file', 'a:b', 'folder/./file'])
      expect(safeRelativePath(bad)).toBe(false);
    expect(safeRelativePath('src/example.txt')).toBe(true);
  });
});
