import { useEffect, useRef, useState } from 'react';
import type { Bridge, PersonalizationProfile, PersonalizationSnapshot } from './types';
import { utf8Bytes } from './policy';
import { emptyProfile, PROFILE_LIMITS, scalarCount, validPersonalization, validProfile } from './personalization-ui';

export function Personalization({ bridge, locked, onNavigationBlockedChange }: { bridge: Bridge; locked: boolean; onNavigationBlockedChange?: (blocked: boolean) => void }) {
  const [snapshot, setSnapshot] = useState<PersonalizationSnapshot | null>(null);
  const [profile, setProfile] = useState(emptyProfile);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('Loading the protected local profile…');
  const mounted = useRef(false);
  const work = useRef(false);
  const draftBaseline = useRef(emptyProfile());
  const available = Boolean(bridge.getPersonalization && bridge.savePersonalization && bridge.clearPersonalization);
  const changed = JSON.stringify(profile) !== JSON.stringify(snapshot?.profile ?? draftBaseline.current);
  useEffect(() => { onNavigationBlockedChange?.(busy || changed); }, [busy, changed, onNavigationBlockedChange]);
  useEffect(() => () => { onNavigationBlockedChange?.(false); }, [onNavigationBlockedChange]);
  useEffect(() => {
    mounted.current = true;
    if (!available) { setMessage('Personalization is unavailable in this host. Nothing is saved or sent.'); return () => { mounted.current = false; }; }
    let disposed = false;
    void bridge.getPersonalization!().then(value => {
      if (disposed) return;
      if (!validPersonalization(value)) throw new Error('invalid_profile');
      draftBaseline.current = value.profile; setSnapshot(value); setProfile(value.profile); setMessage('');
    }).catch(() => { if (!disposed) setMessage('The protected local profile is unavailable. Existing data was not overwritten.'); });
    return () => { disposed = true; mounted.current = false; };
  }, [bridge, available]);
  async function mutate(clear: boolean) {
    if (locked || work.current || !snapshot || !available || (!clear && !validProfile(profile))) return;
    work.current = true; setBusy(true); setMessage('');
    try {
      if (clear) await bridge.clearPersonalization!({ expectedRevision: snapshot.revision });
      else await bridge.savePersonalization!({ expectedRevision: snapshot.revision, profile: { ...profile } });
      const value: unknown = await bridge.getPersonalization!();
      if (!validPersonalization(value) || value.revision <= snapshot.revision || (clear ? Object.values(value.profile).some(Boolean) : (Object.keys(PROFILE_LIMITS) as (keyof PersonalizationProfile)[]).some(key => value.profile[key] !== profile[key]))) throw new Error('profile_receipt_invalid');
      if (mounted.current) { draftBaseline.current = value.profile; setSnapshot(value); setProfile(value.profile); setMessage(clear ? 'The active local profile was cleared. Backups and recovery remnants are not erased.' : 'Personalization saved locally. It is never sent to Meta or GitHub.'); }
    } catch {
      if (mounted.current) { setSnapshot(null); setMessage('The local change could not be verified. Existing data was not reset; close and reopen Settings before trying again.'); }
    } finally { work.current = false; if (mounted.current) setBusy(false); }
  }
  const totalCharacters = Object.values(profile).reduce((sum, text) => sum + scalarCount(text), 0);
  const totalBytes = Object.values(profile).reduce((sum, text) => sum + utf8Bytes(text), 0);
  const disabled = locked || busy || !snapshot || !available;
  return <section className="personalization-page" aria-labelledby="personalization-heading">
    <h3 id="personalization-heading">Personalization</h3>
    <p>Keep optional personal details and preferences on this Windows account only.</p>
    <p>Choose Save to persist changes. Closing Settings discards unsaved profile edits.</p>
    <p className="privacy-callout">This profile is local-only. Aster never includes it in Meta requests, context summaries, generated skills or plugins, GitHub operations, exports or diagnostics. It does not grant file, tool, web or security authority.</p>
    <h4>Custom instructions</h4><p>Record local preferences. They are not instructions sent to the model.</p>
    {(Object.keys(PROFILE_LIMITS) as (keyof PersonalizationProfile)[]).map((key, index) => {
      const limit = PROFILE_LIMITS[key], multiline = key === 'customInstructions' || key === 'moreAboutYou';
      const characters = scalarCount(profile[key]), bytes = utf8Bytes(profile[key]);
      const invalid = characters > limit.characters || bytes > limit.bytes || !validProfile({ ...emptyProfile(), [key]: profile[key] });
      const inputProps = { id: `profile-${key}`, value: profile[key], disabled, maxLength: limit.characters * 2, 'aria-label': limit.label, 'aria-describedby': `profile-${key}-limit`, 'aria-invalid': invalid, onChange: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => setProfile(current => ({ ...current, [key]: event.target.value })) };
      return <div className={`profile-field ${invalid ? 'over-limit' : ''}`} key={key}>
        {index === 1 && <><h4>About you</h4><p>These optional details remain local and do not tailor provider responses.</p></>}
        <label htmlFor={inputProps.id}><span>{limit.label}</span><span className="optional-label">Optional</span></label>
        {multiline ? <textarea {...inputProps} rows={key === 'customInstructions' ? 7 : 5}/> : <input {...inputProps}/>}
        <div id={`profile-${key}-limit`} className="profile-counter"><span>Maximum {limit.characters.toLocaleString('en-GB')} Unicode characters and {limit.bytes.toLocaleString('en-GB')} UTF-8 bytes. Current value uses {bytes.toLocaleString('en-GB')} bytes.</span><span>{characters.toLocaleString('en-GB')} / {limit.characters.toLocaleString('en-GB')} characters</span></div>
      </div>;
    })}
    <p className={validProfile(profile) ? 'profile-total' : 'profile-total over-limit'}>Complete profile: {totalCharacters.toLocaleString('en-GB')} / 3,200 characters · {totalBytes.toLocaleString('en-GB')} / 12,800 UTF-8 bytes</p>
    <p className="privacy-callout">Aster stores this profile in a separate encrypted current-user local domain. A normal uninstall retains it. Clear removes only Aster's active copy; it is not forensic erasure of disk remnants, recovery records, backups or third-party copies.</p>
    <div className="settings-actions profile-actions"><button className="secondary-action" disabled={disabled || !Object.values(snapshot?.profile ?? {}).some(Boolean)} onClick={() => void mutate(true)}>Clear Personalization</button><button className="primary-action" disabled={disabled || !changed || !validProfile(profile)} onClick={() => void mutate(false)}>Save Personalization</button></div>
    {message && <p className="inline-state" role="status">{message}</p>}
  </section>;
}
