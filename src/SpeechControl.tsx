import { useCallback, useEffect, useRef, useState } from 'react';
import type { Bridge } from './types';

const messages: Record<string, string> = {
  busy: 'Another native operation is in progress. Finish it before dictating.',
  native_dialog_busy: 'Finish the other Windows dialogue before starting dictation.',
  speech_cancelled: 'Dictation cancelled.',
  speech_unavailable: 'The local speech resources could not be verified. Repair the Aster installation before dictating.',
  speech_microphone: 'Check the default input device and microphone access for desktop apps in Windows Settings.',
  speech_runtime: 'The local decoder needs the Microsoft Visual C++ x64 runtime, including OpenMP. Repair the runtime before dictating.',
  speech_timeout: 'Local transcription timed out. Try dictating a shorter passage.',
  speech_no_audio: 'No speech was recognised. Check the selected Windows microphone.',
  speech_failed: 'Local speech recognition failed. Your existing draft is available to edit.',
};

interface Props {
  bridge: Bridge;
  disabled: boolean;
  onActive: (active: boolean) => void;
  onTranscript: (transcript: string) => void;
  onNotice: (message: string) => void;
}
interface CaptureSession { id: string; started: boolean; unsubscribe?: () => void }

export function SpeechControl({ bridge, disabled, onActive, onTranscript, onNotice }: Props) {
  const [phase, setPhase] = useState<'idle' | 'starting' | 'recording' | 'finishing'>('idle');
  const session = useRef<CaptureSession | null>(null);
  const supported = Boolean(bridge.subscribeSpeechPhase && bridge.startSpeechCapture && bridge.finishSpeechCapture && bridge.cancelSpeechCapture);

  const discard = useCallback(() => {
    const current = session.current;
    if (!current) return;
    // Invalidate first. Even a successful late result belongs to the old draft.
    session.current = null;
    current.unsubscribe?.();
    onActive(false);
    setPhase('idle');
    if (current.started) void bridge.cancelSpeechCapture?.(current.id).catch(() => {});
  }, [bridge, onActive]);

  useEffect(() => {
    const hide = () => { if (document.hidden) discard(); };
    document.addEventListener('visibilitychange', hide);
    return () => { document.removeEventListener('visibilitychange', hide); discard(); };
  }, [discard]);
  useEffect(() => { if (disabled) discard(); }, [disabled, discard]);

  async function start() {
    if (disabled || !supported || session.current || !bridge.startSpeechCapture || !bridge.subscribeSpeechPhase) return;
    const id = crypto.randomUUID();
    const current: CaptureSession = { id, started: false };
    session.current = current;
    onActive(true);
    setPhase('starting');
    try {
      const unsubscribe = await bridge.subscribeSpeechPhase(event => {
        if (session.current !== current || !current.started || event.sessionId !== id) return;
        if (event.phase === 'recording') setPhase(previous => previous === 'starting' ? 'recording' : previous);
        if (event.phase === 'transcribing') setPhase('finishing');
      });
      if (session.current !== current) { unsubscribe(); return; }
      current.unsubscribe = unsubscribe;
      current.started = true;
      const transcript = await bridge.startSpeechCapture(id);
      if (session.current === current && !document.hidden) onTranscript(transcript);
    } catch (error) {
      if (session.current === current) {
        const code = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
        onNotice(Object.prototype.hasOwnProperty.call(messages, code) ? messages[code] : messages.speech_failed);
      }
    } finally {
      if (session.current === current) {
        session.current = null;
        current.unsubscribe?.();
        onActive(false);
        setPhase('idle');
      }
    }
  }

  async function finish() {
    const current = session.current;
    if (!current || phase !== 'recording' || !bridge.finishSpeechCapture) return;
    setPhase('finishing');
    try { await bridge.finishSpeechCapture(current.id); }
    catch {
      if (session.current === current) {
        discard();
        onNotice('The microphone stop could not be confirmed. Check the Windows microphone indicator.');
      }
    }
  }

  const label = { idle: 'Dictate message', starting: 'Preparing dictation', recording: 'Finish dictation', finishing: 'Transcribing speech' }[phase];
  return <div className="speech-control">
    <button type="button" className="speech-button" aria-label={label}
      title={phase === 'idle' ? 'Speech to text with local whisper.cpp' : label}
      disabled={disabled || !supported || phase === 'starting' || phase === 'finishing'}
      onClick={() => { if (session.current) void finish(); else void start(); }}>
      <svg aria-hidden="true" width="23" height="23" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
        {phase === 'idle' ? <><rect x="8" y="2" width="8" height="13" rx="4"/><path d="M4 11a8 8 0 0 0 16 0M12 19v3m-3 0h6"/></> : <rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor"/>}
      </svg>
    </button>
    {phase !== 'idle' && <div className="speech-status">
      <span role="status">{phase === 'starting' ? 'Confirm in Windows, then wait for the listening indicator.' : phase === 'recording' ? 'Listening… Recording stops automatically after 30 seconds.' : 'Transcribing locally…'}</span>
      <button type="button" onClick={discard}>Cancel dictation</button>
    </div>}
  </div>;
}
