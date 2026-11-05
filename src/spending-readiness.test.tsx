import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, selectClassification } from './test-fixtures';

const now = Date.parse('2026-11-05T20:33:00Z');
const maximum = Date.parse('2026-12-06T00:00:00Z') / 1000;
const unconfigured = () => ({ ...baseline, availability: 'needs_budget' as const, budget: { ...baseline.budget, configured: false, configurationExpiresAt: null, configurationMaxUntil: maximum } });
beforeEach(() => { vi.spyOn(Date, 'now').mockReturnValue(now); });
afterEach(() => { cleanup(); vi.restoreAllMocks(); });

async function openUsage() {
  await screen.findByText('Configure the spending policy in Settings before sending.');
  await userEvent.click(screen.getByRole('button', { name: 'Review spending policy' }));
  return screen.getByRole('dialog', { name: 'Settings and privacy' });
}
function fill(expiry = '2026-11-30T23:59') {
  for (const [label, value] of [['GBP per USD (integer parts per million)', '743053'], ['Safety margin (basis points)', '1000'], ['Tax (basis points)', '2000'], ['Payment charges (basis points)', '300']]) fireEvent.change(screen.getByLabelText(label), { target: { value } });
  fireEvent.change(screen.getByLabelText(/Valid until \(local Windows time/), { target: { value: expiry } });
}
const review = () => userEvent.click(screen.getByRole('button', { name: 'Review financial configuration in Windows' }));

describe('spending activation and Send prerequisites (T-102/T-104)', () => {
  it('explains the supplied out-of-policy expiry inside Settings before native review', async () => {
    const fake = fakeBridge(unconfigured()); render(<App bridge={fake.bridge}/>);
    const dialog = await openUsage(); fill('2026-12-06T20:33');
    expect(within(dialog).getByText('Not active — complete and confirm the financial review.')).toBeTruthy();
    const expiry = within(dialog).getByLabelText(/Valid until \(local Windows time/);
    expect(Date.parse(expiry.getAttribute('max')!) / 1000).toBe(maximum);
    await review();
    expect(await within(dialog).findByText(/Choose an expiry after now and no later than/)).toBeTruthy();
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('explains the positive safety margin instead of submitting a zero margin', async () => {
    const fake = fakeBridge(unconfigured()); render(<App bridge={fake.bridge}/>);
    const dialog = await openUsage(); fill();
    fireEvent.change(screen.getByLabelText('Safety margin (basis points)'), { target: { value: '0' } });
    await review();
    expect(await within(dialog).findByText('Safety margin must be an integer from 1 to 10000 basis points.')).toBeTruthy();
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
  });
  it('keeps a declined financial review visibly inactive and preserves the chosen values', async () => {
    const fake = fakeBridge(unconfigured()); vi.mocked(fake.bridge.configureBudget).mockResolvedValue({ configured: false });
    render(<App bridge={fake.bridge}/>); const dialog = await openUsage(); fill(); await review();
    expect(await within(dialog).findByText('The financial review was not confirmed. The existing policy was not changed.')).toBeTruthy();
    expect(within(dialog).getByText('Not active — complete and confirm the financial review.')).toBeTruthy();
    expect((within(dialog).getByLabelText('GBP per USD (integer parts per million)') as HTMLInputElement).value).toBe('743053');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('shows an allowlisted host expiry rejection inside the active dialog without exposing raw bodies', async () => {
    const fake = fakeBridge(unconfigured()); vi.mocked(fake.bridge.configureBudget).mockRejectedValue('budget_expiry_out_of_range');
    render(<App bridge={fake.bridge}/>); const dialog = await openUsage(); fill(); await review();
    expect(await within(dialog).findByText('The chosen expiry exceeds the current host limit. Choose an earlier date shown in Usage and review again.')).toBeTruthy();
    expect(fake.bridge.configureBudget).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('enables Send only after confirmed host activation and separate explicit classification', async () => {
    const inactive = unconfigured(); let activated = false;
    const fake = fakeBridge(inactive);
    vi.mocked(fake.bridge.configureBudget).mockImplementation(async () => { activated = true; return { configured: true }; });
    vi.mocked(fake.bridge.getStatus).mockImplementation(async () => activated ? { ...inactive, availability: 'ready', budget: { ...inactive.budget, configured: true, configurationExpiresAt: Date.parse('2026-11-30T23:59:00Z') / 1000 } } : inactive);
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Configure the spending policy in Settings before sending.');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Reply with the synthetic word READY.' } });
    const dialog = await openUsage(); fill(); await review();
    expect(await within(dialog).findByText('The spending policy is active. Close Settings, write a message and classify its content before sending.')).toBeTruthy();
    expect(within(dialog).getByText('Active')).toBeTruthy();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Close dialogue' }));
    const send = screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement;
    expect(send.disabled).toBe(true);
    expect(screen.getByText(/Open the reasoning menu and choose Classify content/)).toBeTruthy();
    await selectClassification('synthetic');
    await waitFor(() => expect(send.disabled).toBe(false));
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    await userEvent.click(send); await waitFor(() => expect(fake.bridge.sendMessage).toHaveBeenCalledOnce());
    expect(vi.mocked(fake.bridge.sendMessage).mock.calls[0][0]).toMatchObject({ message: 'Reply with the synthetic word READY.', dataClass: 'synthetic', reasoningEffort: 'medium' });
  });
  it('explains an empty ready draft without selecting a classification or starting any operation', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>);
    expect(await screen.findByText('Write a message to enable Send.')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
});
