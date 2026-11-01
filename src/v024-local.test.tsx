import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { Personalization } from './Personalization';
import { Customization } from './Customization';
import { baseline, fakeBridge, saved, selectClassification } from './test-fixtures';
import { emptyProfile, scalarCount, validPersonalization, validProfile } from './personalization-ui';
import { validCustomization, validCustomizationDraft, validCustomizationPackage } from './customization-ui';
import type { CustomizationDraft, CustomizationPackage, CustomizationSnapshot, PersonalizationSnapshot } from './types';

afterEach(() => cleanup());
function localFixture() {
  const fake = fakeBridge();
  let profile: PersonalizationSnapshot = { revision: 0, profile: emptyProfile() };
  let catalogue: CustomizationSnapshot = { revision: 0, entries: [{ id: 'skill:aster-builder', kind: 'skill', name: 'Aster Builder', description: 'Owner-supplied inert engineering instructions', enabled: false, readOnly: true, availability: 'instructions-only', entryRevision: 0 }, { id: 'builtin:github', kind: 'plugin', name: 'GitHub', description: 'Reviewed built-in connector', enabled: false, readOnly: true, availability: 'built-in-connector', entryRevision: 0 }] };
  const packages = new Map<string, CustomizationPackage>([['skill:aster-builder', { id: 'skill:aster-builder', kind: 'skill', slug: 'aster-builder', name: 'Aster Builder', description: 'Owner-supplied inert engineering instructions', instructions: 'Full inert synthetic skill fixture. No script, tool permission or execution authority.', enabled: false, readOnly: true, entryRevision: 0 }]]);
  fake.bridge.getPersonalization = vi.fn(async () => structuredClone(profile));
  fake.bridge.savePersonalization = vi.fn(async input => { if (input.expectedRevision !== profile.revision || !validProfile(input.profile)) throw new Error('profile_denied'); profile = { revision: profile.revision + 1, profile: structuredClone(input.profile) }; return structuredClone(profile); });
  fake.bridge.clearPersonalization = vi.fn(async input => { if (input.expectedRevision !== profile.revision) throw new Error('profile_conflict'); profile = { revision: profile.revision + 1, profile: emptyProfile() }; return structuredClone(profile); });
  fake.bridge.getCustomization = vi.fn(async () => structuredClone(catalogue));
  fake.bridge.getCustomizationPackage = vi.fn(async ({ id }) => structuredClone(packages.get(id)!));
  fake.bridge.saveCustomization = vi.fn(async input => {
    if (input.expectedRevision !== catalogue.revision || !validCustomizationDraft(input.package)) throw new Error('catalogue_denied');
    const value = input.package, id = `${value.kind}:${value.slug}`, entryRevision = catalogue.revision + 1;
    packages.set(id, { id, ...value, enabled: false, readOnly: false, entryRevision });
    catalogue = { revision: entryRevision, entries: [...catalogue.entries.filter(entry => entry.id !== id), { id, kind: value.kind, name: value.name, description: value.description, enabled: false, readOnly: false, availability: 'instructions-only', entryRevision }] };
    return structuredClone(catalogue);
  });
  fake.bridge.setCustomizationEnabled = vi.fn(async input => {
    if (input.expectedRevision !== catalogue.revision) throw new Error('catalogue_conflict');
    catalogue = { revision: catalogue.revision + 1, entries: catalogue.entries.map(entry => entry.id === input.id ? { ...entry, enabled: input.enabled } : entry) }; return structuredClone(catalogue);
  });
  fake.bridge.importCustomization = vi.fn(async () => structuredClone(catalogue));
  return { ...fake, profile: () => profile, catalogue: () => catalogue };
}
describe('v0.2.4 first-chat authentication renderer denials (T-085)', () => {
  it.each(['missing','unverified','invalid','unavailable'] as const)('blocks new-chat, import and first send for %s proof while saved history stays viewable', async credentialValidation => {
    const fake = fakeBridge({ ...baseline, credentialValidation, configured: credentialValidation !== 'missing', availability: 'needs_credential', capabilities: { ...baseline.capabilities, attachmentImport: true } }, [saved('prior', 'Prior eligible conversation')]);
    fake.bridge.selectAttachments = vi.fn(async () => []); fake.bridge.removeAttachment = vi.fn(async () => ({ removed: true }));
    render(<App bridge={fake.bridge}/>); await screen.findByText('Reply for prior');
    expect((screen.getByRole('button', { name: 'New conversation' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('button', { name: 'Add attachments' }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Synthetic draft retained locally' } }); await selectClassification('synthetic');
    fireEvent.keyDown(screen.getByRole('textbox', { name: 'Message for Aster' }), { key: 'Enter' });
    await userEvent.keyboard('{Control>}n{/Control}');
    expect(fake.bridge.createConversation).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(fake.bridge.selectAttachments).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Open Settings' })).toBeTruthy();
  });
  it('removes the notice only after a current host proof and restores it after key deletion', async () => {
    let status = { ...baseline, credentialValidation: 'unverified' as const, availability: 'needs_credential' as const } as typeof baseline;
    const fake = fakeBridge(status); vi.mocked(fake.bridge.getStatus).mockImplementation(async () => structuredClone(status));
    fake.bridge.verifyCredential = vi.fn(async () => { status = { ...status, credentialValidation: 'valid', availability: 'ready' }; return { state: 'valid' as const }; });
    vi.mocked(fake.bridge.removeCredential).mockImplementation(async () => { status = { ...status, configured: false, credentialValidation: 'missing', availability: 'needs_credential' }; return { configured: false }; });
    render(<App bridge={fake.bridge}/>); await screen.findByRole('button', { name: 'Open Settings' });
    await userEvent.click(screen.getByRole('button', { name: 'Open Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Validate key in Windows' }));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Open Settings' })).toBeNull());
    await userEvent.click(screen.getByRole('button', { name: 'Remove key…' })); await waitFor(() => expect(screen.getByRole('button', { name: 'Open Settings' })).toBeTruthy());
    expect(fake.bridge.verifyCredential).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
});
describe('strict local-only Personalization (T-093)', () => {
  it('counts Unicode scalar values and bytes; rejects overflow, forged fields and bad restore', () => {
    expect(scalarCount('😀')).toBe(1);
    expect(validProfile({ ...emptyProfile(), customInstructions: '😀'.repeat(2000) })).toBe(true);
    expect(validProfile({ ...emptyProfile(), customInstructions: '😀'.repeat(2001) })).toBe(false);
    expect(validProfile({ ...emptyProfile(), nickname: 'x'.repeat(81) })).toBe(false);
    for (const text of ['\ud800','\udc00','x\0y','x\ny']) expect(validProfile({ ...emptyProfile(), nickname: text })).toBe(false);
    expect(validProfile({ ...emptyProfile(), occupation: 'x'.repeat(121) })).toBe(false);
    expect(validProfile({ ...emptyProfile(), moreAboutYou: 'x'.repeat(1001) })).toBe(false);
    expect(validProfile({ ...emptyProfile(), apiKey: 'forged' })).toBe(false);
    expect(validPersonalization({ revision: 513, profile: emptyProfile() })).toBe(false);
    expect(validPersonalization({ revision: 0, profile: emptyProfile(), approved: true })).toBe(false);
  });
  it('saves only to the revision-bound local domain and clears the active copy', async () => {
    const fake = localFixture(); render(<Personalization bridge={fake.bridge} locked={false}/>);
    await waitFor(() => expect((screen.getByLabelText('Nickname') as HTMLInputElement).disabled).toBe(false));
    fireEvent.change(screen.getByLabelText('Nickname'), { target: { value: 'Local-only canary' } });
    fireEvent.change(screen.getByLabelText('Custom instructions'), { target: { value: 'Private profile canary <script>inert</script>' } });
    await userEvent.click(screen.getByRole('button', { name: 'Save Personalization' })); await screen.findByText('Personalization saved locally. It is never sent to Meta or GitHub.');
    expect(fake.bridge.savePersonalization).toHaveBeenCalledWith({ expectedRevision: 0, profile: { ...emptyProfile(), nickname: 'Local-only canary', customInstructions: 'Private profile canary <script>inert</script>' } });
    expect(document.querySelector('.personalization-page script')).toBeNull(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled(); expect(localStorage.length).toBe(0);
    await userEvent.click(screen.getByRole('button', { name: 'Clear Personalization' })); await screen.findByText('The active local profile was cleared. Backups and recovery remnants are not erased.');
    expect(fake.bridge.clearPersonalization).toHaveBeenCalledWith({ expectedRevision: 1 }); expect(fake.profile().profile).toEqual(emptyProfile());
  });
  it('does not leak a saved profile into the closed message intent', async () => {
    const fake = localFixture(); await fake.bridge.savePersonalization!({ expectedRevision: 0, profile: { ...emptyProfile(), nickname: 'PRIVATE_PROFILE_CANARY' } });
    render(<App bridge={fake.bridge}/>); await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(screen.getByRole('button', { name: 'Settings' })); await userEvent.click(screen.getByRole('button', { name: 'Personalization' }));
    await screen.findByDisplayValue('PRIVATE_PROFILE_CANARY'); await userEvent.click(screen.getByRole('button', { name: 'Close dialogue' }));
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Synthetic eligible example' } }); await selectClassification('synthetic'); await userEvent.click(screen.getByRole('button', { name: 'Send message' }));
    expect(JSON.stringify(vi.mocked(fake.bridge.sendMessage).mock.calls)).not.toContain('PRIVATE_PROFILE_CANARY');
  });
  it('fails closed for a stale save receipt, retains the draft and never retries', async () => {
    const fake = localFixture(); vi.mocked(fake.bridge.savePersonalization!).mockResolvedValue(undefined);
    render(<Personalization bridge={fake.bridge} locked={false}/>); await waitFor(() => expect((screen.getByLabelText('Nickname') as HTMLInputElement).disabled).toBe(false));
    fireEvent.change(screen.getByLabelText('Nickname'), { target: { value: 'Do not discard me' } }); await userEvent.click(screen.getByRole('button', { name: 'Save Personalization' }));
    await screen.findByText(/local change could not be verified/); expect((screen.getByLabelText('Nickname') as HTMLInputElement).value).toBe('Do not discard me');
    expect((screen.getByRole('button', { name: 'Save Personalization' }) as HTMLButtonElement).disabled).toBe(true); expect(fake.bridge.savePersonalization).toHaveBeenCalledOnce();
  });
});
describe('bounded inert Skills and Plugins catalogue (T-088/T-089)', () => {
  it('does not infer an empty or disabled catalogue when its native methods are unavailable', async () => {
    const fake = localFixture();
    await fake.bridge.setCustomizationEnabled!({ expectedRevision: 0, id: 'skill:aster-builder', enabled: true });
    expect(fake.catalogue().entries[0].enabled).toBe(true);
    delete fake.bridge.getCustomization;
    render(<Customization bridge={fake.bridge} locked={false}/>);
    await screen.findByText(/Existing entries and preferences are unverified; this view made no catalogue change/);
    expect(document.body.textContent).not.toMatch(/No package is installed or enabled|No packages in this category/);
    expect((screen.getByRole('button', { name: 'Add skill' }) as HTMLButtonElement).disabled).toBe(true);
    expect(fake.bridge.setCustomizationEnabled).toHaveBeenCalledOnce(); expect(fake.bridge.saveCustomization).not.toHaveBeenCalled(); expect(fake.bridge.importCustomization).not.toHaveBeenCalled();
  });
  it('rejects active/unknown package and catalogue fields, traversal, bad text and overflow', () => {
    const value: CustomizationDraft = { kind: 'skill', slug: 'synthetic-skill', name: 'Synthetic skill', description: '', instructions: 'Inert instructions.' };
    expect(validCustomizationDraft(value)).toBe(true);
    for (const slug of ['../escape','a--b','-abc','ABC','a-','a'.repeat(65)]) expect(validCustomizationDraft({ ...value, slug })).toBe(false);
    expect(validCustomizationDraft({ ...value, command: 'powershell' })).toBe(false); expect(validCustomizationDraft({ ...value, instructions: 'x'.repeat(12001) })).toBe(false);
    expect(validCustomizationDraft({ ...value, name: 'x\ny' })).toBe(false);
    expect(validCustomization({ revision: 0, entries: [] })).toBe(true); expect(validCustomization({ revision: 0, entries: [], token: 'secret' })).toBe(false);
    expect(validCustomizationPackage({ ...value, id: 'skill:synthetic-skill', enabled: false, entryRevision: 1, readOnly: false, approved: true })).toBe(false);
  });
  it('shows the supplied skill, creates an immutable package and toggles only a host preference', async () => {
    const fake = localFixture(); render(<Customization bridge={fake.bridge} locked={false}/>); await screen.findByText('Aster Builder');
    expect((screen.getByRole('switch', { name: 'Enable Aster Builder' }) as HTMLInputElement).disabled).toBe(false);
    await userEvent.click(screen.getByRole('switch', { name: 'Enable Aster Builder' })); await screen.findByText(/catalogue preference was enabled/);
    expect(fake.bridge.setCustomizationEnabled).toHaveBeenCalledWith({ expectedRevision: 0, id: 'skill:aster-builder', enabled: true });
    await userEvent.click(screen.getByRole('button', { name: 'Add skill' }));
    for (const [label,value] of [['Package identifier','synthetic-skill'],['Name','Synthetic skill'],['Description','Synthetic description'],['Instructions','<script>inert package instructions</script>']]) fireEvent.change(screen.getByLabelText(label), { target: { value } });
    await userEvent.click(screen.getByRole('button', { name: 'Save package revision' })); await screen.findByText('Synthetic skill');
    expect(fake.bridge.saveCustomization).toHaveBeenCalledWith({ expectedRevision: 1, package: { kind: 'skill', slug: 'synthetic-skill', name: 'Synthetic skill', description: 'Synthetic description', instructions: '<script>inert package instructions</script>' } });
    expect(document.querySelector('.customization-page script')).toBeNull(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Edit Synthetic skill' })); await screen.findByDisplayValue('<script>inert package instructions</script>');
    expect(fake.bridge.getCustomizationPackage).toHaveBeenCalledWith({ id: 'skill:synthetic-skill' });
  });
  it('keeps generation and unqualified account management honestly unavailable; import is native intent only', async () => {
    const fake = localFixture(); render(<Customization bridge={fake.bridge} locked={false}/>); await screen.findByText('Aster Builder');
    await userEvent.click(screen.getByRole('tab', { name: 'Plugins' }));
    expect((screen.getByRole('button', { name: 'Generate with Muse Spark' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('button', { name: 'Manage account' }) as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(screen.getByRole('button', { name: 'Import package in Windows' })); await screen.findByText('No catalogue change was made.');
    expect(fake.bridge.importCustomization).toHaveBeenCalledWith({ expectedRevision: 0 }); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(within(screen.getByRole('tabpanel')).getByText('GitHub')).toBeTruthy();
  });
  it('shows complete read-only Aster Builder instructions while still allowing its explicit enable preference', async () => {
    const fake = localFixture(); render(<Customization bridge={fake.bridge} locked={false}/>); await screen.findByText('Aster Builder');
    await userEvent.click(screen.getByRole('button', { name: 'View Aster Builder' }));
    const instructions = await screen.findByDisplayValue('Full inert synthetic skill fixture. No script, tool permission or execution authority.');
    expect((instructions as HTMLTextAreaElement).readOnly).toBe(true); expect(screen.queryByRole('button', { name: 'Save package revision' })).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Close package view' }));
    expect((screen.getByRole('switch', { name: 'Enable Aster Builder' }) as HTMLInputElement).disabled).toBe(false);
    await userEvent.click(screen.getByRole('switch', { name: 'Enable Aster Builder' })); await screen.findByText('The catalogue preference was enabled. It does not connect an account or grant host authority.');
    expect(fake.bridge.setCustomizationEnabled).toHaveBeenCalledWith({ expectedRevision: 0, id: 'skill:aster-builder', enabled: true }); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
});
