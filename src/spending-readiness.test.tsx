import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, selectClassification } from './test-fixtures';

const now = Date.parse('2026-11-05T20:33:00Z');
const maximum = Date.parse('2026-12-06T00:00:00Z') / 1000;
const unconfigured = () => ({
  ...baseline, availability: 'needs_budget' as const,
  budget: { ...baseline.budget, configured: false, monthlyPolicy: null, configurationMaxUntil: maximum },
});
const fiscallyUnconfigured = () => ({
  ...baseline, availability: 'needs_budget' as const,
  budget: { ...baseline.budget, configured: false, fiscalProfileConfigured: false, configurationExpiresAt: null, configurationMaxUntil: maximum },
});
beforeEach(() => { vi.spyOn(Date, 'now').mockReturnValue(now); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });

async function openSpending() {
  await screen.findByText('Configure the spending policy in Settings before sending.');
  await userEvent.click(screen.getByRole('button', { name: 'Review spending policy' }));
  return within(screen.getByRole('dialog', { name: 'Settings and privacy' }));
}
function fillFiscal(expiry = '2026-11-30T23:59') {
  for (const [label, value] of [['GBP per USD (integer parts per million)', '743053'], ['Safety margin (basis points)', '1000'], ['Payment charges (basis points)', '300']]) fireEvent.change(screen.getByLabelText(label), { target: { value } });
  fireEvent.change(screen.getByLabelText(/Valid until \(local Windows time/), { target: { value: expiry } });
}
const reviewFiscal = () => userEvent.click(screen.getByRole('button', { name: 'Review billing profile in Windows' }));
const reviewMonthly = () => userEvent.click(screen.getByRole('button', { name: 'Review monthly spending in Windows' }));
function fillMonthly(cap = '120') {
  fireEvent.change(screen.getByLabelText('Monthly limit (£)'), { target: { value: cap } });
}

describe('recurring monthly spending', () => {
  it('shows exactly four Usage inputs and sends optional values as native defaults', async () => {
    const fake = fakeBridge(unconfigured()); render(<App bridge={fake.bridge}/>);
    const dialog = await openSpending();
    expect(dialog.getAllByRole('textbox')).toHaveLength(4);
    expect(dialog.getByLabelText('First warning (% remaining)').getAttribute('placeholder')).toBe('10');
    expect(dialog.getByLabelText('Second warning (% remaining)').getAttribute('placeholder')).toBe('5');
    expect(dialog.getByLabelText('UK VAT (%)').getAttribute('placeholder')).toBe('20');
    expect(dialog.queryByText('Pending reservations')).toBeNull();
    fillMonthly('120.45'); await reviewMonthly();
    expect(fake.bridge.configureMonthlyBudget).toHaveBeenCalledExactlyOnceWith({ limitPence: 12045 });
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('handles exact percentages and an explicit zero VAT override without treating it as blank', async () => {
    const fake = fakeBridge(unconfigured()); render(<App bridge={fake.bridge}/>);
    await openSpending();
    fillMonthly('10.01');
    fireEvent.change(screen.getByLabelText('First warning (% remaining)'), { target: { value: '12.25' } });
    fireEvent.change(screen.getByLabelText('Second warning (% remaining)'), { target: { value: '2.5' } });
    fireEvent.change(screen.getByLabelText('UK VAT (%)'), { target: { value: '0' } });
    await reviewMonthly();
    expect(fake.bridge.configureMonthlyBudget).toHaveBeenCalledExactlyOnceWith({ limitPence: 1001, firstRemainingBps: 1225, secondRemainingBps: 250, taxBps: 0 });
  });
  it('rejects excessive cap, reversed warnings and non-decimal values before sending a proposal', async () => {
    const fake = fakeBridge(unconfigured()); render(<App bridge={fake.bridge}/>);
    const dialog = await openSpending(); fillMonthly('10000.01'); await reviewMonthly();
    expect(await dialog.findByText('Enter a monthly limit from £0.01 to £10,000.00.')).toBeTruthy();
    fillMonthly();
    fireEvent.change(screen.getByLabelText('First warning (% remaining)'), { target: { value: '1' } });
    await reviewMonthly();
    expect(await dialog.findByText('Check the amount, VAT percentage and the order of both warnings.')).toBeTruthy();
    expect(fake.bridge.configureMonthlyBudget).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('keeps a declined native review visibly uncommitted and preserves the entered draft', async () => {
    const fake = fakeBridge(unconfigured()); vi.mocked(fake.bridge.configureMonthlyBudget).mockResolvedValue({ configured: false });
    render(<App bridge={fake.bridge}/>); const dialog = await openSpending(); fillMonthly(); await reviewMonthly();
    expect(await dialog.findByText('The financial review was not confirmed. The existing policy was not changed.')).toBeTruthy();
    expect((dialog.getByLabelText('Monthly limit (£)') as HTMLInputElement).value).toBe('120');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('enables Send only after host activation and separate explicit classification', async () => {
    const inactive = unconfigured(); let activated = false;
    const fake = fakeBridge(inactive);
    vi.mocked(fake.bridge.configureMonthlyBudget).mockImplementation(async () => { activated = true; return { configured: true }; });
    vi.mocked(fake.bridge.getStatus).mockImplementation(async () => activated ? baseline : inactive);
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Configure the spending policy in Settings before sending.');
    fireEvent.change(screen.getByRole('textbox', { name: 'Message for Aster' }), { target: { value: 'Reply with the synthetic word READY.' } });
    const dialog = await openSpending(); fillMonthly('89'); await reviewMonthly();
    expect(await dialog.findByText('Monthly spending policy saved.')).toBeTruthy();
    await userEvent.click(dialog.getByRole('button', { name: 'Close dialogue' }));
    const send = screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement;
    expect(send.disabled).toBe(true);
    expect(screen.getByText('Classify the complete draft before sending.')).toBeTruthy();
    await selectClassification('synthetic');
    await waitFor(() => expect(send.disabled).toBe(false));
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    await userEvent.click(send); await waitFor(() => expect(fake.bridge.sendMessage).toHaveBeenCalledOnce());
    expect(vi.mocked(fake.bridge.sendMessage).mock.calls[0][0]).toMatchObject({ message: 'Reply with the synthetic word READY.', dataClass: 'synthetic', reasoningEffort: 'medium' });
  });
  it('shows both thresholds as the settled amount rises and blocks provider actions at the cap', async () => {
    const fake = fakeBridge({ ...baseline, budget: {...baseline.budget, warningLevel: 1, spentNanoGbp: '80100000000' } });
    render(<App bridge={fake.bridge}/>);
    expect((await screen.findByRole('alert')).textContent).toContain('at least 90%'); // host settled status
    vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...baseline, budget: {...baseline.budget, warningLevel: 2, spentNanoGbp: '84550000000' } });
    fireEvent(window, new Event('focus'));
    expect(await screen.findByText(/Second spending warning: at least 95%/)).toBeTruthy();
    vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...baseline, availability: 'budget_limit_reached', budget: {...baseline.budget, limitReached: true, warningLevel: 2, spentNanoGbp: '89000000000', remainingNanoGbp: '0' } });
    fireEvent(window, new Event('focus'));
    expect(await screen.findByText(/Provider calls resume no earlier than the next London calendar month/)).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect((screen.getByRole('button', { name: 'Validate key in Windows' }) as HTMLButtonElement).disabled).toBe(true);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
  });
  it('refreshes only the local status at the host-provided calendar boundary', async () => {
    vi.useFakeTimers();
    const fake = fakeBridge({ ...baseline, availability: 'budget_limit_reached', budget: {
      ...baseline.budget, limitReached: true, warningLevel: 2, spentNanoGbp: '89000000000', remainingNanoGbp: '0', monthResetsAt: now / 1000 + 2,
    } });
    render(<App bridge={fake.bridge}/>);
    await act(() => vi.advanceTimersByTimeAsync(0));
    expect(screen.getByText(/Provider calls resume no earlier/)).toBeTruthy();
    vi.mocked(fake.bridge.getStatus).mockResolvedValue(baseline);
    await act(() => vi.advanceTimersByTimeAsync(2_000));
    expect(screen.queryByText(/Provider calls resume no earlier/)).toBeNull();
    expect(fake.bridge.getStatus).toHaveBeenCalledTimes(2);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
  });
});
describe('independent explicit billing profile', () => {
  it('rejects the supplied out-of-policy expiry before review', async () => {
    const fake = fakeBridge(fiscallyUnconfigured()); render(<App bridge={fake.bridge}/>);
    const dialog = await openSpending(); fillFiscal('2026-12-06T20:33');
    const expiry = dialog.getByLabelText(/Valid until \(local Windows time/);
    expect(Date.parse(expiry.getAttribute('max')!) / 1000).toBe(maximum);
    await reviewFiscal();
    expect(await dialog.findByText(/Choose an expiry after now and no later than/)).toBeTruthy();
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('explains the positive safety margin instead of assuming one', async () => {
    const fake = fakeBridge(fiscallyUnconfigured()); render(<App bridge={fake.bridge}/>);
    const dialog = await openSpending(); fillFiscal();
    fireEvent.change(screen.getByLabelText('Safety margin (basis points)'), { target: { value: '0' } });
    await reviewFiscal();
    expect(await dialog.findByText('Safety margin must be an integer from 1 to 10000 basis points.')).toBeTruthy();
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled();
  });
  it('shows an allowlisted host expiry rejection within Settings', async () => {
    const fake = fakeBridge(fiscallyUnconfigured()); vi.mocked(fake.bridge.configureBudget).mockRejectedValue('budget_expiry_out_of_range');
    render(<App bridge={fake.bridge}/>); const dialog = await openSpending(); fillFiscal(); await reviewFiscal();
    expect(await dialog.findByText('The chosen expiry exceeds the current host limit. Choose an earlier date shown in the provider billing profile and review again.')).toBeTruthy();
    expect(fake.bridge.configureBudget).toHaveBeenCalledOnce(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('explains an empty ready draft without selecting a classification or starting any operation', async () => {
    const fake = fakeBridge(); render(<App bridge={fake.bridge}/>);
    expect(await screen.findByText('Write a message to enable Send.')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Send message' }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect(fake.bridge.configureBudget).not.toHaveBeenCalled(); expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
});
