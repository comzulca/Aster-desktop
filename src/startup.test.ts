import { afterEach, describe, expect, it, vi } from 'vitest';
import nativeConfig from '../src-tauri/tauri.conf.json';
import entryHtml from '../index.html?raw';

// These are bounded source/DOM fixtures, not installed WebView2 launch evidence.
// Vitest intentionally stubs CSS imports; read only these fixed repository
// sources through Node's existing built-in module instead of adding a loader.
const { existsSync, readFileSync } = await vi.importActual<{
  existsSync(path: string): boolean;
  readFileSync(path: string, encoding: 'utf8'): string;
}>('node:fs');
const startupSource = 'public/startup.css';
const startupStyles = existsSync(startupSource) ? readFileSync(startupSource, 'utf8') : '';
const rendererCss = readFileSync('src/styles.css', 'utf8');
const DARK_CANVAS = '#1b1f20';
const DARK_CANVAS_RGB = 'rgb(27, 31, 32)';
const ORIGINAL_CSP = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self' ipc: http://ipc.localhost";
const stylesAdded: HTMLStyleElement[] = [];
const nodesAdded: HTMLElement[] = [];

afterEach(() => {
  stylesAdded.splice(0).forEach(style => style.remove());
  nodesAdded.splice(0).forEach(node => node.remove());
});

function paintUnhydratedRoot() {
  const style = document.createElement('style');
  style.textContent = startupStyles;
  document.head.append(style);
  stylesAdded.push(style);
  const root = document.createElement('div');
  root.id = 'root';
  document.body.append(root);
  nodesAdded.push(root);
  return root;
}

describe('dark native and pre-module startup surfaces (T-073 / T-074)', () => {
  it('uses the existing opaque dark canvas on the main window and WebView, with one zoom owner', () => {
    const main = nativeConfig.app.windows[0] as typeof nativeConfig.app.windows[0] & {
      backgroundColor?: string; theme?: string; zoomHotkeysEnabled?: boolean;
    };
    expect(main.label).toBe('main');
    expect(main.backgroundColor).toBe(DARK_CANVAS);
    expect(main.theme).toBe('Dark');
    expect(main.zoomHotkeysEnabled).toBe(false);
    expect(main).not.toHaveProperty('transparent');
    expect(main).not.toHaveProperty('additionalBrowserArgs');
    expect(main).toHaveProperty('visible', false);
  });

  it('links one local render-blocking stylesheet in the head before the application module', () => {
    const entry = new DOMParser().parseFromString(entryHtml, 'text/html');
    const links = [...entry.head.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')];
    expect(links).toHaveLength(1);
    expect(links[0]?.getAttribute('href')).toBe('/startup.css');
    expect(links[0]?.hasAttribute('media')).toBe(false);
    expect(links[0]?.hasAttribute('disabled')).toBe(false);
    expect(links[0]?.hasAttribute('onload')).toBe(false);
    expect(entry.head.querySelector('meta[name="theme-color"]')?.getAttribute('content')).toBe(DARK_CANVAS);
    expect(entry.head.querySelector('meta[name="color-scheme"]')?.getAttribute('content')).toBe('dark');
    expect(entry.querySelectorAll('style, script:not([src]), [style]')).toHaveLength(0);
    expect(entryHtml.indexOf('href="/startup.css"')).toBeGreaterThan(0);
    expect(entryHtml.indexOf('href="/startup.css"')).toBeLessThan(entryHtml.indexOf('src="/src/main.tsx"'));
  });

  it('keeps html, body and an empty root opaque and visible before module loading or after failed hydration', () => {
    expect(existsSync(startupSource)).toBe(true);
    const root = paintUnhydratedRoot();
    for (const element of [document.documentElement, document.body, root]) {
      const style = getComputedStyle(element);
      expect(style.backgroundColor).toBe(DARK_CANVAS_RGB);
      expect(style.visibility).not.toBe('hidden');
      expect(style.display).not.toBe('none');
      expect(style.opacity).not.toBe('0');
    }
    root.textContent = 'The native host is unavailable. No paid request was made.';
    expect(root.textContent).toContain('unavailable');
    expect(getComputedStyle(root).backgroundColor).toBe(DARK_CANVAS_RGB);
    expect(getComputedStyle(root).display).not.toBe('none');
  });

  it('leaves policy, local-only loading and the existing renderer canvas intact', () => {
    expect(nativeConfig.app.security.csp).toBe(ORIGINAL_CSP);
    expect(nativeConfig.app.security.capabilities).toEqual(['main']);
    expect(rendererCss).toContain(`--canvas:${DARK_CANVAS}`);
    const startup = startupStyles;
    expect(startup).not.toMatch(/@import|url\(|animation|transition|display\s*:\s*none|visibility\s*:\s*hidden|opacity\s*:\s*0/i);
    expect(startup).toMatch(/color-scheme\s*:\s*dark/);
    expect(entryHtml).not.toMatch(/http[s]?:\/\/|onload=|location\.|setTimeout/);
  });
});

describe('bounded host-owned initial presentation (T-077)', () => {
  it('sets one opaque process-local WebView2 colour before the host starts', () => {
    const binary = readFileSync('src-tauri/src/main.rs', 'utf8');
    expect(binary).toContain('"WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "FF1B1F20"');
    expect(binary.indexOf('std::env::set_var(')).toBeLessThan(binary.indexOf('aster_desktop_lib::run()'));
    expect(binary).not.toMatch(/setx|SetEnvironmentVariable|--disable-gpu|additional_browser_args/i);
  });

  it('declares fixed minimum/deadline bounds and no generic hide or focus action', () => {
    const module = 'src-tauri/src/startup.rs';
    const source = existsSync(module) ? readFileSync(module, 'utf8') : '';
    expect(source).toContain('const MIN_REVEAL_MS: u64 = 6_000;');
    expect(source).toContain('const HARD_DEADLINE_MS: u64 = 10_000;');
    expect(source).toContain('const MAX_TRACE_EVENTS: usize = 16;');
    expect(source).toContain('startup_shell_ready');
    expect(source).toContain('deny_unknown_fields');
    expect(source).not.toMatch(/\.hide\(|\.set_focus\(|\.navigate\(|\.reload\(|\.eval\(/);
    expect(readFileSync('src-tauri/capabilities/main.json', 'utf8')).not.toMatch(/core:window:|allow-show|allow-hide/);
  });
  it('keeps the initial timing and shared trace bounds unchanged while adding capture-only native milestones', () => {
    const source = readFileSync('src-tauri/src/startup.rs', 'utf8');
    const trace = readFileSync('src-tauri/src/render_trace.rs', 'utf8');
    expect(source).toContain('const MIN_REVEAL_MS: u64 = 6_000;');
    expect(source).toContain('const HARD_DEADLINE_MS: u64 = 10_000;');
    expect(trace).toContain('const MAX_EVENTS: usize = 96;');
    expect(trace).toContain('const MAX_MS: u64 = 60_000;');
    expect(trace).toContain('const RESERVED_CRITICAL_EVENTS: usize = 9;');
    for (const category of ['StartupGateCreated', 'StartupShellAccepted', 'StartupRevealReady', 'StartupRevealDeadline', 'StartupShowStarted',
      'StartupShowVisible', 'StartupShowDenied', 'StartupShowVisibleCleanupFailed', 'StartupShowDeniedCleanupFailed', 'StartupDispatchDenied']) {
      expect(trace).toContain(category);
    }
    expect(source).not.toMatch(/std::env::var|env::var|additional_browser_args|disable.gpu|\.hide\(|\.set_focus\(/i);
  });
  it('preserves Tao visibility with a bounded owned-UI-thread activation veto, never a global input hook', () => {
    const source = readFileSync('src-tauri/src/startup.rs', 'utf8');
    expect(source).toContain('run_on_main_thread');
    expect(source).toContain('SetWindowsHookExW(WH_CBT');
    expect(source).toContain('HCBT_ACTIVATE'); expect(source).toContain('HCBT_SETFOCUS');
    expect(source).toContain('GetWindowThreadProcessId'); expect(source).toContain('GetCurrentThreadId');
    expect(source).toContain('window.show().is_ok()'); expect(source).toContain('IsWindowVisible(handle)');
    expect(source).toContain('impl Drop for ActivationGuard'); expect(source).toContain('UnhookWindowsHookEx');
    expect(source).toContain('CallNextHookEx'); expect(source).toContain('TARGET.with(|target| target.set(0))');
    expect(source).not.toMatch(/WH_KEYBOARD|WH_MOUSE|SetForegroundWindow|SetActiveWindow|SetFocus\(|SendInput|GetAsyncKeyState/);
  });
});
