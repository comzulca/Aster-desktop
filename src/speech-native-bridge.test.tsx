import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { baseline } from './test-fixtures';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.resetAllMocks();
});

function transportFixture() {
  const speechEvent = 'aster://speech-phase';
  const start = 'start_speech_capture';
  const stop = vi.fn();
  let reply!: (text: string) => void;
  let reject!: (reason: string) => void;
  let notify!: Parameters<typeof listen>[1];
  vi.stubGlobal('__TAURI_INTERNALS__', {});
  vi.mocked(listen).mockImplementation(async (name, handler) => {
    if (name === speechEvent) {
      notify = handler;
      return stop;
    }
    return () => {};
  });
  const ipc = vi.mocked(invoke);
  ipc.mockImplementation(async <T,>(command: string): Promise<T> => {
    if (command === 'get_app_status') return {
      ...baseline, configured: false, credentialAvailable: false,
      credentialValidation: 'missing', availability: 'needs_credential',
    } as T;
    if (command === 'list_projects') return { revision: 0, projects: [], assignments: [] } as T;
    if (command === 'list_conversations') return [] as T;
    if (command === start) return new Promise<string>((resolve, refuse) => {
      reply = resolve;
      reject = refuse;
    }) as Promise<T>;
    if (command === 'finish_speech_capture' || command === 'record_render_event') return undefined as T;
    throw new Error('Unexpected fixture IPC');
  });
  return {
    ipc, stop,
    speechCalls: () => ipc.mock.calls.filter(([name]) => name.endsWith('_speech_capture')),
    phase: (payload: unknown) => act(() => notify({ event: speechEvent, id: 1, payload })),
    text: (value: string) => act(async () => reply(value)),
    failure: (reason: string) => act(async () => reject(reason)),
  };
}

describe('composer dictation through the production Tauri adapter', () => {
  it('binds the icon to one native session and stages the result without provider or history calls', async () => {
    const fake = transportFixture();
    render(<App />);
    await screen.findByRole('button', { name: 'Open Settings' });
    const box = screen.getByRole<HTMLTextAreaElement>('textbox', { name: 'Message for Aster' });
    fireEvent.change(box, { target: { value: 'Borrador:' } });
    await userEvent.click(screen.getByRole('button', { name: 'Dictate message' }));
    expect(listen).toHaveBeenCalledWith('aster://speech-phase', expect.any(Function), { target: 'main' });
    expect(fake.speechCalls()).toEqual([['start_speech_capture', { sessionId: expect.any(String) }]]);
    const { sessionId } = fake.speechCalls()[0][1] as { sessionId: string };
    expect(sessionId).toMatch(/^[\da-f]{8}(?:-[\da-f]{4}){3}-[\da-f]{12}$/);
    fake.phase({ sessionId, phase: 'recording', extra: true });
    fake.phase({ sessionId: crypto.randomUUID(), phase: 'recording' });
    expect(screen.getByRole('button', { name: 'Preparing dictation' })).toHaveProperty('disabled', true);
    fake.phase({ sessionId, phase: 'recording' });
    await userEvent.click(screen.getByRole('button', { name: 'Finish dictation' }));
    fake.phase({ sessionId, phase: 'transcribing' });
    expect(screen.getByText('Transcribing locally…')).toBeTruthy();
    await fake.text('El proyecto Atlas tiene tres mapas de prueba.');
    expect(box.value).toBe('Borrador: El proyecto Atlas tiene tres mapas de prueba.');
    expect(fake.speechCalls()).toEqual([
      ['start_speech_capture', { sessionId }], ['finish_speech_capture', { sessionId }],
    ]);
    expect(fake.stop).toHaveBeenCalledOnce();
    const commands = fake.ipc.mock.calls.map(([command]) => command);
    for (const unexpected of ['send_message', 'create_conversation', 'verify_meta_credential']) {
      expect(commands).not.toContain(unexpected);
    }
  });

  it('preserves the draft when the native request returns a microphone error', async () => {
    const fake = transportFixture();
    render(<App />);
    await screen.findByRole('button', { name: 'Open Settings' });
    const box = screen.getByRole<HTMLTextAreaElement>('textbox', { name: 'Message for Aster' });
    fireEvent.change(box, { target: { value: 'Keep my typed draft' } });
    await userEvent.click(screen.getByRole('button', { name: 'Dictate message' }));
    await fake.failure('speech_microphone');
    expect(screen.getByText('Check the default input device and microphone access for desktop apps in Windows Settings.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Dictate message' })).toHaveProperty('disabled', false);
    expect(box.value).toBe('Keep my typed draft');
    expect(fake.stop).toHaveBeenCalledOnce();
    expect(fake.speechCalls()).toHaveLength(1);
    expect(fake.ipc.mock.calls.map(([command]) => command)).not.toContain('send_message');
  });
});
