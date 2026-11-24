import { useCallback, useEffect, useLayoutEffect, useRef } from 'react';
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
  draftId: string | null;
  phase: SpeechPhase;
  onPhase: (phase: SpeechPhase) => void;
  onTranscript: (transcript: string) => void;
  onNotice: (message: string) => void;
}
export const speechStatus = {
  starting: 'Confirm in Windows, then wait for the listening indicator.',
  recording: 'Listening… Recording stops automatically after 30 seconds.',
  finishing: 'Waiting for Windows to finish recording…',
  transcribing: 'Transcribing locally…',
  stopping: 'Stopping dictation. Waiting for confirmation from Windows…',
  unconfirmed: 'The stop request could not be confirmed. Check the Windows microphone indicator while Aster waits for capture to end.',
  blocked: 'Aster could not confirm microphone release. Check the Windows microphone indicator. Restart Aster before dictating again.',
};
export type SpeechPhase = 'idle' | keyof typeof speechStatus;
export const capturePending = (phase: SpeechPhase) => phase !== 'idle' && phase !== 'blocked';
const labels: Record<SpeechPhase, string> = {
  idle: 'Dictate message', starting: 'Preparing dictation', recording: 'Finish dictation',
  finishing: 'Finishing recording', transcribing: 'Transcribing speech', stopping: 'Stopping dictation',
  unconfirmed: 'Microphone stop unconfirmed', blocked: 'Microphone release unconfirmed',
};
interface CaptureSession {
  id: string;
  draftId: string | null;
  started: boolean;
  phase: SpeechPhase;
  unsubscribe?: () => void;
}
const stopping = (phase: SpeechPhase) => phase === 'stopping' || phase === 'unconfirmed';

export function SpeechControl({ bridge, disabled, draftId, phase, onPhase, onTranscript, onNotice }: Props) {
  const session = useRef<CaptureSession | null>(null);
  const supported = Boolean(bridge.subscribeSpeechPhase && bridge.startSpeechCapture && bridge.finishSpeechCapture && bridge.cancelSpeechCapture);

  const transition = useCallback((current: CaptureSession, next: SpeechPhase) => {
    current.phase = next;
    onPhase(next);
  }, [onPhase]);

  const discard = useCallback(() => {
    const current = session.current;
    if (!current || stopping(current.phase)) return;
    if (!current.started) {
      session.current = null;
      onPhase('idle');
      return;
    }
    // A cancel reply acknowledges the signal. Only the original capture can
    // settle ownership, and its transcript is no longer eligible for insertion.
    transition(current, 'stopping');
    current.unsubscribe?.();
    current.unsubscribe = undefined;
    void (async () => {
      try {
        await bridge.cancelSpeechCapture?.(current.id);
      } catch {
        if (session.current === current) transition(current, 'unconfirmed');
      }
    })();
  }, [bridge, onPhase, transition]);

  useEffect(() => {
    const hide = () => { if (document.hidden) discard(); };
    document.addEventListener('visibilitychange', hide);
    return () => { document.removeEventListener('visibilitychange', hide); discard(); };
  }, [discard]);
  useLayoutEffect(() => {
    if (disabled || (session.current && session.current.draftId !== draftId)) discard();
  }, [disabled, discard, draftId]);

  async function start() {
    if (phase !== 'idle' || disabled || !supported || session.current || !bridge.startSpeechCapture || !bridge.subscribeSpeechPhase) return;
    const id = crypto.randomUUID();
    const current: CaptureSession = { id, draftId, started: false, phase: 'starting' };
    session.current = current;
    onPhase('starting');
    let terminal: SpeechPhase = 'idle';
    try {
      const unsubscribe = await bridge.subscribeSpeechPhase(event => {
        if (session.current !== current || !current.started || stopping(current.phase) || event.sessionId !== id) return;
        if (event.phase === 'recording' && current.phase === 'starting') transition(current, 'recording');
        if (event.phase === 'transcribing') transition(current, 'transcribing');
      });
      if (session.current !== current) { unsubscribe(); return; }
      current.unsubscribe = unsubscribe;
      current.started = true;
      const transcript = await bridge.startSpeechCapture(id);
      if (typeof transcript !== 'string') terminal = 'blocked';
      else if (session.current === current && !stopping(current.phase) && !document.hidden) onTranscript(transcript);
    } catch (error) {
      if (session.current === current) {
        const code = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
        const known = Object.hasOwn(messages, code);
        // This also covers the host's speech_release_unconfirmed result. An
        // unknown transport error cannot establish that Windows released input.
        if (current.started && !known) terminal = 'blocked';
        else if (!stopping(current.phase)) onNotice(known ? messages[code] : messages.speech_failed);
      }
    } finally {
      if (session.current === current) {
        session.current = null;
        current.unsubscribe?.();
        onPhase(terminal);
      }
    }
  }

  async function finish() {
    const current = session.current;
    if (!current || current.phase !== 'recording' || !bridge.finishSpeechCapture) return;
    transition(current, 'finishing');
    try { await bridge.finishSpeechCapture(current.id); }
    catch {
      if (session.current === current) discard();
    }
  }

  const label = labels[phase];
  return <div className="speech-control">
    <button type="button" className="speech-button" aria-label={label}
      title={phase === 'idle' ? 'Speech to text with local whisper.cpp' : label}
      disabled={disabled || !supported || (phase !== 'idle' && phase !== 'recording')}
      onClick={() => { if (session.current) void finish(); else void start(); }}>
      <svg aria-hidden="true" width="23" height="23" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
        {phase === 'idle' ? <><rect x="8" y="2" width="8" height="13" rx="4"/><path d="M4 11a8 8 0 0 0 16 0M12 19v3m-3 0h6"/></> : <rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor"/>}
      </svg>
    </button>
    {phase !== 'idle' && <div className="speech-status">
      <span role="status">{speechStatus[phase]}</span>
      {!stopping(phase) && phase !== 'blocked' && <button type="button" onClick={discard}>Cancel dictation</button>}
    </div>}
  </div>;
}
