import { useEffect, useRef, useState } from 'react';
import type { Bridge, RenderTraceSnapshot } from './types';
import { validRenderTrace } from './render-trace-ui';

export function RenderingDiagnostics({ bridge }: { bridge: Bridge }) {
  const [snapshot, setSnapshot] = useState<RenderTraceSnapshot | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const mounted = useRef(true);
  const pending = useRef(false);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  async function inspect() {
    if (!bridge.getRenderTrace || pending.current) return;
    pending.current = true; setBusy(true); setNotice(null); setSnapshot(null);
    try {
      const value: unknown = await bridge.getRenderTrace();
      if (!validRenderTrace(value)) throw new Error('render_trace_unavailable');
      if (mounted.current) setSnapshot(value);
    } catch { if (mounted.current) setNotice('The local rendering observation is unavailable or incompatible. No diagnostic content was uploaded.'); }
    finally { pending.current = false; if (mounted.current) setBusy(false); }
  }
  return <div className="rendering-diagnostics">
    <h4>Rendering diagnostics</h4>
    <p>A content-free observation of this process only: at most 96 events within the first 60 seconds. Native window and WebView2 categories are separate from lower-trust renderer markers. Initial navigation may predate observer attachment. These observations do not establish compositor or painted-pixel completeness, or a flicker cure.</p>
    <button className="secondary-action" disabled={busy || !bridge.getRenderTrace} onClick={() => void inspect()}>{busy ? 'Reading local observation…' : 'Read rendering diagnostics'}</button>
    {!bridge.getRenderTrace && <p>This host does not expose rendering diagnostics.</p>}
    {notice && <p role="status">{notice}</p>}
    {snapshot && <><p role="status">{snapshot.webview2DetailEventsAvailable ? 'The captured host receipt reports the full fixed WebView2 observer set registered.' : 'The captured host receipt reports detailed WebView2 observation unavailable.'} Local observation only. Nothing was sent to Meta, GitHub, tools or an export.</p><pre aria-label="Local rendering observation">{JSON.stringify(snapshot, null, 2)}</pre></>}
  </div>;
}
