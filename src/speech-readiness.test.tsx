import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline, fakeBridge, saved } from './test-fixtures';
import type { AppStatus, Bridge } from './types';

afterEach(cleanup);
const button = (name: string) => screen.getByRole('button', { name }) as HTMLButtonElement;
const draft = () => screen.getByRole('textbox', { name: 'Message for Aster' }) as HTMLTextAreaElement;
const write = (value: string) => fireEvent.change(draft(), { target: { value } });
const absent: AppStatus = { ...baseline, configured: false, credentialValidation: 'missing', availability: 'needs_credential' };

function speechFixture(status = baseline) {
  const fake = fakeBridge(status);
  let resolve!: (text: string) => void;
  let reject!: (error: string) => void;
  let notify!: Parameters<NonNullable<Bridge['subscribeSpeechPhase']>>[0];
  const unsubscribe = vi.fn();
  fake.bridge.subscribeSpeechPhase = vi.fn(async callback => { notify = callback; return unsubscribe; });
  fake.bridge.startSpeechCapture = vi.fn((id: string) => new Promise<string>((done, fail) => {
    notify({ sessionId: id, phase: 'recording' });
    resolve = done;
    reject = fail;
  }));
  fake.bridge.finishSpeechCapture = vi.fn(async () => {});
  fake.bridge.cancelSpeechCapture = vi.fn(async () => {});
  return { ...fake, phase: (id: string, phase: 'recording' | 'transcribing') => act(() => notify({ sessionId: id, phase })), unsubscribe, result: (text: string) => act(async () => resolve(text)), failure: (error: string) => act(async () => reject(error)) };
}

describe('local dictation', () => {
  it('captures without Meta credentials and stages the reviewed transcript without sending or creating a conversation', async () => {
    const fake = speechFixture(absent);
    render(<App bridge={fake.bridge}/>);
    await screen.findByRole('button', { name: 'Open Settings' });
    write('Synthetic');
    await userEvent.click(button('Dictate message'));
    const id = vi.mocked(fake.bridge.startSpeechCapture!).mock.calls[0][0];
    expect(id).toMatch(/^[a-f0-9-]{36}$/i);
    write('Synthetic typed draft');
    await userEvent.click(button('Finish dictation'));
    expect(fake.bridge.finishSpeechCapture).toHaveBeenCalledWith(id);
    await fake.result('extra words');
    expect(draft().value).toBe('Synthetic typed draft extra words');
    expect(screen.getByLabelText('Content classification').textContent).toBe('Not classified');
    expect(fake.bridge.createConversation).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
  });

  it('blocks Enter during dictation and requires a new classification for the resulting draft', async () => {
    const fake = speechFixture();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    write('Typed');
    await userEvent.click(within(screen.getByRole('group', { name: 'Classify this draft' })).getByRole('button', { name: 'Synthetic' }));
    expect(button('Send message').disabled).toBe(false);
    await userEvent.click(button('Dictate message'));
    fireEvent.keyDown(draft(), { key: 'Enter' });
    expect(button('Send message').disabled).toBe(true);
    await fake.result('spoken');
    expect(draft().value).toBe('Typed spoken');
    expect(button('Send message').disabled).toBe(true);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it.each(['cancel', 'settings', 'unmount'] as const)('discards a late result after %s', async reason => {
    const fake = speechFixture();
    const app = render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    write('Keep this draft');
    await userEvent.click(button('Dictate message'));
    if (reason === 'unmount') app.unmount();
    else await userEvent.click(button(reason === 'cancel' ? 'Cancel dictation' : 'Settings'));
    expect(fake.bridge.cancelSpeechCapture).toHaveBeenCalledWith(vi.mocked(fake.bridge.startSpeechCapture!).mock.calls[0][0]);
    await fake.result('late audio');
    if (reason !== 'unmount') expect(draft().value).toBe('Keep this draft');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('blocks a second capture and Send until the original capture settles, even after a successful cancellation request', async () => {
    const fake = speechFixture();
    let acknowledge!: () => void;
    vi.mocked(fake.bridge.cancelSpeechCapture!).mockImplementation(() => new Promise<void>(done => { acknowledge = done; }));
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    write('Keep typed text');
    await userEvent.click(button('Synthetic'));
    await userEvent.click(button('Dictate message'));
    await userEvent.click(button('Cancel dictation'));
    expect(button('Stopping dictation').disabled).toBe(true);
    await act(async () => acknowledge());
    expect(button('Stopping dictation').disabled).toBe(true);
    expect(button('Send message').disabled).toBe(true);
    fake.phase(vi.mocked(fake.bridge.startSpeechCapture!).mock.calls[0][0], 'recording');
    expect(button('Stopping dictation').disabled).toBe(true);
    await fake.failure('speech_cancelled');
    expect(button('Dictate message').disabled).toBe(false);
    expect(button('Send message').disabled).toBe(false);
    expect(draft().value).toBe('Keep typed text');
    expect(fake.bridge.startSpeechCapture).toHaveBeenCalledOnce();
  });

  it('keeps a rejected cancellation visible across Settings and conversation changes until capture ends', async () => {
    const fake = speechFixture();
    vi.mocked(fake.bridge.cancelSpeechCapture!).mockRejectedValue('private cancel diagnostic');
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(button('Dictate message'));
    await userEvent.click(button('Settings'));
    expect(await within(screen.getByRole('dialog')).findByText(/Check the Windows microphone indicator/)).toBeTruthy();
    await userEvent.click(button('Close dialogue'));
    await userEvent.click(button('New conversation'));
    await waitFor(() => expect(fake.bridge.createConversation).toHaveBeenCalled());
    expect(button('Microphone stop unconfirmed').disabled).toBe(true);
    expect(document.body.textContent).not.toContain('private cancel diagnostic');
    write('New draft');
    await fake.result('late words');
    expect(draft().value).toBe('New draft');
    expect(button('Dictate message').disabled).toBe(false);
    expect(fake.bridge.cancelSpeechCapture).toHaveBeenCalledOnce();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('does not treat failed finish and synchronous cancellation as microphone release', async () => {
    const fake = speechFixture();
    vi.mocked(fake.bridge.finishSpeechCapture!).mockRejectedValue('private finish diagnostic');
    vi.mocked(fake.bridge.cancelSpeechCapture!).mockImplementation(() => { throw new Error('private synchronous diagnostic'); });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    write('Keep typed text');
    await userEvent.click(button('Synthetic'));
    await userEvent.click(button('Dictate message'));
    await userEvent.click(button('Finish dictation'));
    const warning = await screen.findByRole('button', { name: 'Microphone stop unconfirmed' });
    expect((warning as HTMLButtonElement).disabled).toBe(true);
    expect(button('Send message').disabled).toBe(true);
    const id = vi.mocked(fake.bridge.startSpeechCapture!).mock.calls[0][0];
    expect(fake.bridge.finishSpeechCapture).toHaveBeenCalledWith(id);
    expect(fake.bridge.cancelSpeechCapture).toHaveBeenCalledWith(id);
    expect(document.body.textContent).not.toContain('private');
    await fake.result('late words');
    expect(draft().value).toBe('Keep typed text');
    expect(button('Dictate message').disabled).toBe(false);
    expect(button('Send message').disabled).toBe(false);
  });

  it('retains the microphone warning when the page is shown again', async () => {
    const fake = speechFixture();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(button('Dictate message'));
    const hidden = vi.spyOn(document, 'hidden', 'get');
    try {
      hidden.mockReturnValue(true);
      fireEvent(document, new Event('visibilitychange'));
      hidden.mockReturnValue(false);
      fireEvent(document, new Event('visibilitychange'));
      expect(button('Stopping dictation').disabled).toBe(true);
      await fake.result('late words');
      expect(draft().value).toBe('');
      expect(button('Dictate message').disabled).toBe(false);
    } finally { hidden.mockRestore(); }
  });

  it('ignores a late cancellation rejection after the original capture settled and another started', async () => {
    const fake = speechFixture();
    let refuse!: (error: string) => void;
    vi.mocked(fake.bridge.cancelSpeechCapture!).mockImplementationOnce(() => new Promise<void>((_, reject) => { refuse = reject; }));
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(button('Dictate message'));
    await userEvent.click(button('Cancel dictation'));
    await fake.failure('speech_cancelled');
    await userEvent.click(button('Dictate message'));
    await act(async () => refuse('late error'));
    expect(button('Finish dictation').disabled).toBe(false);
    await fake.result('Current capture');
    expect(draft().value).toBe('Current capture');
  });

  it('preserves the entire draft on UTF-8 overflow', async () => {
    const fake = speechFixture();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    const original = 'x'.repeat(32766);
    write(original);
    await userEvent.click(button('Dictate message'));
    await fake.result('🦊');
    expect(draft().value).toBe(original);
    expect(screen.getByText(/The complete dictation would exceed/)).toBeTruthy();
  });
  it('only observes native phases of its session and keeps transcribing terminal for phase updates', async () => {
    const fake = speechFixture();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(button('Dictate message'));
    const id = vi.mocked(fake.bridge.startSpeechCapture!).mock.calls[0][0];
    fake.phase(crypto.randomUUID(), 'transcribing');
    expect(button('Finish dictation').disabled).toBe(false);
    fake.phase(id, 'transcribing');
    expect(button('Transcribing speech').disabled).toBe(true);
    fake.phase(id, 'recording');
    expect(button('Transcribing speech').disabled).toBe(true);
    await fake.result('Synthetic words');
    expect(fake.unsubscribe).toHaveBeenCalledOnce();
    expect(draft().value).toBe('Synthetic words');
  });

  it('cancels before phase subscription completes without starting capture', async () => {
    const fake = speechFixture();
    let ready!: (stop: () => void) => void;
    vi.mocked(fake.bridge.subscribeSpeechPhase!).mockImplementation(() => new Promise(resolve => { ready = resolve; }));
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(button('Dictate message'));
    expect(button('Preparing dictation').disabled).toBe(true);
    await userEvent.click(button('Cancel dictation'));
    await act(async () => ready(fake.unsubscribe));
    expect(fake.unsubscribe).toHaveBeenCalledOnce();
    expect(fake.bridge.startSpeechCapture).not.toHaveBeenCalled();
    expect(fake.bridge.cancelSpeechCapture).not.toHaveBeenCalled();
  });
  it.each(['speech_release_unconfirmed', 'private speech diagnostic fixture'])('keeps dictation unavailable after uncertain capture termination (%s)', async error => {
    const fake = speechFixture();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    write('Keep this text');
    await userEvent.click(button('Dictate message'));
    await userEvent.click(button('Cancel dictation'));
    await fake.failure(error);
    await screen.findByText(/Restart Aster before dictating again/);
    expect(document.body.textContent).not.toContain('private speech diagnostic fixture');
    expect(draft().value).toBe('Keep this text');
    expect(button('Microphone release unconfirmed').disabled).toBe(true);
    write('Still editable');
    await userEvent.click(button('Synthetic'));
    expect(button('Send message').disabled).toBe(false);
  });
  it('keeps a malformed terminal reply from clearing the microphone warning', async () => {
    const fake = speechFixture();
    vi.mocked(fake.bridge.startSpeechCapture!).mockResolvedValue(null as unknown as string);
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    await userEvent.click(button('Dictate message'));
    expect(button('Microphone release unconfirmed').disabled).toBe(true);
    expect(draft().value).toBe('');
  });
});

describe('external and partially saved Windows credentials', () => {
  it('explains a model completion that exhausted its limit before returning visible text', async () => {
    const conversation = saved('synthetic_limit', 'A synthetic request');
    conversation.entries[1].state = 'truncated';
    conversation.entries[1].content = '';
    const fake = fakeBridge(baseline, [conversation]);
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Meta reached the output limit before returning visible text. Try a shorter request or a lower reasoning effort.');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('requires a fresh classification when an already classified draft is edited', async () => {
    const fake = fakeBridge();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Ready for separately reviewed requests.');
    write('Synthetic');
    await userEvent.click(button('Synthetic'));
    expect(button('Send message').disabled).toBe(false);
    write('Changed content');
    fireEvent.keyDown(draft(), { key: 'Enter' });
    expect(button('Send message').disabled).toBe(true);
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('rereads the saved state even when the verification action fails after storage', async () => {
    const fake = fakeBridge(absent);
    let state = absent;
    vi.mocked(fake.bridge.getStatus).mockImplementation(async () => structuredClone(state));
    vi.mocked(fake.bridge.configureCredential).mockImplementation(async () => {
      state = { ...absent, configured: true, credentialValidation: 'unverified' };
      throw new Error('meta_unavailable');
    });
    render(<App bridge={fake.bridge}/>);
    await userEvent.click(await screen.findByRole('button', { name: 'Open Settings' }));
    await userEvent.click(button('Set up key in Windows'));
    await screen.findByText(/Meta is temporarily unavailable/);
    expect(button('Change key in Windows').disabled).toBe(false);
    expect(button('Validate key in Windows').disabled).toBe(false);
    expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });

  it('recognises an externally saved key on window focus and keeps validation explicit', async () => {
    const fake = fakeBridge(absent);
    render(<App bridge={fake.bridge}/>);
    await userEvent.click(await screen.findByRole('button', { name: 'Open Settings' }));
    expect(button('Validate key in Windows').disabled).toBe(true);
    vi.mocked(fake.bridge.getStatus).mockResolvedValue({ ...absent, configured: true, credentialValidation: 'unverified' });
    fireEvent(window, new Event('focus'));
    await waitFor(() => expect(button('Validate key in Windows').disabled).toBe(false));
    expect(fake.bridge.verifyCredential).not.toHaveBeenCalled();
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
  it('explains a cancelled metadata check in the credential settings', async () => {
    const fake = fakeBridge({ ...absent, configured: true, credentialValidation: 'unverified' });
    vi.mocked(fake.bridge.verifyCredential!).mockRejectedValue('credential_review_cancelled');
    render(<App bridge={fake.bridge}/>);
    await userEvent.click(await screen.findByRole('button', { name: 'Open Settings' }));
    await userEvent.click(button('Validate key in Windows'));
    await screen.findByText('Key validation was cancelled in Windows. Select Validate key in Windows when you are ready.');
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
  });
});
