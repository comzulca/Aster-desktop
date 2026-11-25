import { cleanup, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, saved } from './test-fixtures';
import type { ProjectSnapshot } from './types';

const atlas = 'a'.repeat(32);
const first = 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb';
const second = 'c'.repeat(32);
const empty: ProjectSnapshot = { revision: 0, projects: [], assignments: {} };
const created: ProjectSnapshot = { revision: 1, projects: [{ id: atlas, name: 'Atlas' }], assignments: {} };

afterEach(cleanup);

describe('local Projects sidebar', () => {
  it('replaces the lead sidebar label and creates an encrypted-host project through the plus control', async () => {
    const { bridge } = fakeBridge();
    bridge.listProjects = vi.fn(async () => empty);
    bridge.createProject = vi.fn(async () => created);
    const user = userEvent.setup();
    render(<App bridge={bridge}/>);
    const projects = screen.getByRole('region', { name: 'Projects' });
    expect(await within(projects).findByText('No projects')).toBeTruthy();
    const plus = within(projects).getByRole('button', { name: 'Create project' });
    await user.click(plus);
    const form = within(projects).getByRole('form', { name: 'New project' });
    const name = within(form).getByRole('textbox', { name: 'Project name' });
    expect(document.activeElement).toBe(name);
    await user.type(name, ' Atlas ');
    await user.click(within(form).getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(within(projects).getByRole('button', { name: 'Atlas' }).getAttribute('aria-pressed')).toBe('true'));
    expect(document.activeElement).toBe(plus);
    expect(bridge.createProject).toHaveBeenCalledExactlyOnceWith({ expectedRevision: 0, name: 'Atlas' });
    expect(bridge.createConversation).not.toHaveBeenCalled();
    expect(bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('cancels on Escape and rejects duplicate names without writing', async () => {
    const { bridge } = fakeBridge();
    bridge.listProjects = vi.fn(async () => created);
    bridge.createProject = vi.fn(async () => created);
    const user = userEvent.setup();
    render(<App bridge={bridge}/>);
    await screen.findByRole('button', { name: 'Atlas' });
    const plus = screen.getByRole('button', { name: 'Create project' });
    await user.click(plus);
    await user.type(screen.getByRole('textbox', { name: 'Project name' }), 'atlas');
    await user.click(screen.getByRole('button', { name: 'Create' }));
    expect(screen.getByRole('alert').textContent).toContain('already exists');
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('form', { name: 'New project' })).toBeNull();
    expect(document.activeElement).toBe(plus);
    expect(bridge.createProject).not.toHaveBeenCalled();
  });

  it('filters existing history and assigns only the selected saved conversation on explicit save', async () => {
    const { bridge } = fakeBridge(baseline, [saved(first, 'Draft for Atlas', 2), saved(second, 'Unrelated chat')]);
    bridge.listProjects = vi.fn(async () => created);
    bridge.assignConversationProject = vi.fn(async () => ({ ...created, revision: 2, assignments: { [first]: atlas } }));
    const user = userEvent.setup();
    render(<App bridge={bridge}/>);
    const history = screen.getByRole('navigation', { name: 'Saved conversations' });
    await within(history).findByRole('button', { name: /Draft for Atlas/ });
    await user.click(within(history).getByRole('button', { name: /Draft for Atlas/ }));
    await user.click(await screen.findByRole('button', { name: 'Manage project assignment' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Project for this conversation' }), atlas);
    expect(bridge.assignConversationProject).not.toHaveBeenCalled();
    await user.click(screen.getByRole('button', { name: 'Save assignment' }));
    await screen.findByText('Conversation assigned to project.');
    expect(bridge.assignConversationProject).toHaveBeenCalledExactlyOnceWith({ expectedRevision: 1, conversationId: first, projectId: atlas });
    await user.click(screen.getByRole('button', { name: 'Atlas' }));
    expect(within(history).getByRole('button', { name: /Draft for Atlas/ })).toBeTruthy();
    expect(within(history).queryByRole('button', { name: /Unrelated chat/ })).toBeNull();
    await user.click(screen.getByRole('button', { name: 'All conversations' }));
    expect(within(history).getByRole('button', { name: /Unrelated chat/ })).toBeTruthy();
    expect(bridge.sendMessage).not.toHaveBeenCalled();
  });
});
