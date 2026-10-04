#!/usr/bin/env node
/**
 * Deterministic synthetic QA of the compiled renderer, not the native application.
 * Uses bundled Playwright and signed Edge with an ephemeral headless profile.
 * No dependency installation, owner profile, keys, stores, native dialogues or Meta.
 * Run: node scripts/qa/Test-Renderer-v023.mjs --source-p1 <40-hex-commit>
 */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const dist = path.join(root, 'dist');
const edge = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const bundledPlaywright = path.join(os.homedir(), '.cache', 'codex-runtimes', 'codex-primary-runtime', 'dependencies', 'node', 'node_modules', 'playwright');
const args = process.argv.slice(2);
assert.equal(args.length, 2, 'expected-source-p1-argument');
assert.equal(args[0], '--source-p1', 'expected-source-p1-argument');
assert.match(args[1], /^[a-f0-9]{40}$/, 'invalid-source-p1');
const sourceP1 = args[1];
const requiredChecks = [
  'T-083-early-dark-compiled-surface',
  'T-077-shell-signal-independent-of-hydration',
  'T-075-plus-uniform-surface-and-geometry',
  'T-076-independent-medium-and-child-classification',
  'T-075-complete-batch-metadata-draft-and-removal',
  'T-075-selector-cancellation-replaces-batch-only',
  'T-075-malformed-receipt-fails-closed',
  'T-075-chat-switch-revokes-only-current-import-ids',
  'T-075-long-text-scroll-without-truncation',
  'T-073-zoom-keyboard-wheel-and-cleanable-hud',
  'T-075-modal-lock-and-default-cancel',
  'T-075-synthetic-response-stop-and-busy-denials',
  'T-083-compact-1366x768-composer-and-popup',
  'T-083-no-external-or-unregistered-host-operation',
];
const git = (...arguments_) => execFileSync('git', arguments_, { cwd: root, windowsHide: true, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }).trim();
assert.equal(git('cat-file', '-t', sourceP1), 'commit', 'source-p1-not-commit');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const runtimePaths = ['src', 'src-tauri', 'public', 'index.html', 'package.json', 'pnpm-lock.yaml', 'vite.config.ts', 'tsconfig.json'];
const runtimeDiff = git('diff', sourceP1, '--', ...runtimePaths);
const rendererPaths = ['src', 'public', 'index.html', 'package.json', 'pnpm-lock.yaml', 'vite.config.ts', 'tsconfig.json', 'src-tauri/icons', 'src-tauri/tauri.conf.json'];
const rendererDiff = git('diff', sourceP1, '--', ...rendererPaths);
const startedUtc = new Date().toISOString();
const outputDirectory = path.join(root, 'reports', `renderer-v023-${startedUtc.replace(/[:.]/g, '-')}`);
await fs.mkdir(outputDirectory, { recursive: true });
const tauriConfig = JSON.parse(await fs.readFile(path.join(root, 'src-tauri', 'tauri.conf.json'), 'utf8'));
const files = new Map();
async function collect(directory) {
  for (const item of await fs.readdir(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, item.name);
    const stat = await fs.lstat(absolute);
    assert.ok(!stat.isSymbolicLink(), 'dist-link-denied');
    if (stat.isDirectory()) await collect(absolute);
    else {
      assert.ok(stat.isFile(), 'dist-special-file-denied');
      assert.ok(stat.size <= 16 * 1024 * 1024, 'dist-file-limit');
      const relative = path.relative(dist, absolute).split(path.sep).join('/');
      assert.ok(!relative.startsWith('../') && !path.isAbsolute(relative), 'dist-root-escape');
      const bytes = await fs.readFile(absolute);
      files.set(`/${relative}`, { bytes, sha256: sha256(bytes) });
    }
  }
}
await collect(dist);
assert.ok(files.has('/index.html') && files.has('/startup.css'), 'compiled-renderer-missing');
const distFingerprint = sha256([...files].sort(([a], [b]) => a.localeCompare(b, 'en-GB')).map(([name, item]) => `${name}\0${item.sha256}\n`).join(''));
const report = {
  schemaVersion: 1,
  classification: 'synthetic-headless-renderer-only',
  startedUtc,
  sourceP1,
  sourceP1Tree: git('rev-parse', `${sourceP1}^{tree}`),
  checkoutHead: git('rev-parse', 'HEAD'),
  checkoutHeadTree: git('rev-parse', 'HEAD^{tree}'),
  runtimeMatchesP1: runtimeDiff.length === 0,
  runtimeDiffSha256: sha256(runtimeDiff),
  unmatchedNativeFiles: git('diff', '--name-only', sourceP1, '--', 'src-tauri').split('\n').filter(Boolean),
  rendererInputsMatchP1: rendererDiff.length === 0,
  rendererDiffSha256: sha256(rendererDiff),
  sourceBindingScope: 'Compiled renderer/config/icons match P1; native source differences, if present, are not qualified by this check.',
  workingTreeDirty: git('status', '--porcelain').length > 0,
  workingTreeStatusSha256: sha256(git('status', '--porcelain')),
  scriptSha256: sha256(await fs.readFile(fileURLToPath(import.meta.url))),
  compiledDistFingerprint: distFingerprint,
  compiledDistFiles: [...files].map(([name, item]) => ({ name, sha256: item.sha256, bytes: item.bytes.length })),
  environment: {
    node: process.version,
    platform: process.platform,
    release: os.release(),
    architecture: process.arch,
    principal: execFileSync('C:\\Windows\\System32\\whoami.exe', ['/user', '/fo', 'csv', '/nh'], { windowsHide: true, encoding: 'utf8' }).trim(),
  },
  browser: { executable: edge, headless: true, chromiumSandbox: true, profile: 'Playwright-managed ephemeral; no persistent context', version: null },
  limits: { loopbackOnly: true, serviceWorkersBlocked: true, externalRequestsDenied: true, ownerProfileUsed: false, realHostUsed: false, liveRequestsMade: false },
  nativeQualification: 'NOT RUN',
  installedStartupFlashAcceptance: 'NOT RUN',
  accountOrParserOrBackendGrantQualification: 'NOT RUN',
  productionQualification: 'NOT RUN',
  checks: [], screenshots: [], deniedExternalRequests: [], unexpectedFakeCommands: [], pageErrors: [],
  allPassed: false,
};
let origin;
const served = [];
const server = http.createServer((request, response) => {
  try {
    assert.ok(['GET', 'HEAD'].includes(request.method), 'method-denied');
    assert.equal(request.headers.host, new URL(origin).host, 'host-denied');
    const url = new URL(request.url, origin);
    assert.equal(url.origin, origin, 'origin-denied');
    assert.ok(!url.search && !url.hash, 'query-denied');
    const pathname = decodeURIComponent(url.pathname);
    assert.ok(!pathname.includes('\\') && !pathname.includes('\0') && !pathname.includes(':') && !pathname.split('/').includes('..'), 'path-denied');
    const name = pathname === '/' ? '/index.html' : pathname;
    const item = files.get(name);
    if (!item) { response.writeHead(404); response.end(); return; }
    const extension = path.extname(name);
    const contentType = { '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.png': 'image/png', '.svg': 'image/svg+xml' }[extension] ?? 'application/octet-stream';
    response.writeHead(200, { 'Content-Type': contentType, 'Content-Length': item.bytes.length, 'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff', 'Content-Security-Policy': tauriConfig.app.security.csp });
    served.push(name);
    response.end(request.method === 'HEAD' ? undefined : item.bytes);
  } catch { response.writeHead(400); response.end(); }
});
await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
origin = `http://127.0.0.1:${server.address().port}`;

// This fake is deliberately closed, synthetic and installed before any module.
// It never delegates an IPC command to Tauri or the operating system.
function installFake({ slowHydration = false }) {
  const calls = [];
  const unknown = [];
  const callbacks = new Map();
  const listeners = new Map();
  const records = new Map();
  let callbackSequence = 0, listenerSequence = 0, conversationSequence = 0;
  let selection = 'full', batch = [], active = null;
  let release;
  const hydration = slowHydration ? new Promise(resolve => { release = resolve; }) : Promise.resolve();
  let frames = 0;
  const originalFrame = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = callback => originalFrame(timestamp => { frames++; callback(timestamp); });
  const attachments = [
    { id: 'a'.repeat(32), kind: 'pdf', label: 'Synthetic pages.pdf', sourceBytes: 1024, processedBytes: 2048, pageCount: 2 },
    { id: 'b'.repeat(32), kind: 'image', label: 'Synthetic image.png', sourceBytes: 128, processedBytes: 256, pageCount: 1 },
    { id: 'c'.repeat(32), kind: 'text', label: 'Synthetic notes.txt', sourceBytes: 40, processedBytes: 40, pageCount: 0 },
    { id: 'd'.repeat(32), kind: 'image', label: 'Synthetic picture.jpg', sourceBytes: 160, processedBytes: 300, pageCount: 1 },
  ];
  const status = () => ({
    protocolVersion: 4, policyVersion: 4, configured: true, credentialAvailable: true,
    defaultModel: 'muse-spark-1.3-contributor', models: [{ id: 'muse-spark-1.3-contributor', label: 'Muse Spark 1.3 · Contributor' }],
    activeRequest: Boolean(active), projectSelected: false, projectLabel: null,
    capabilities: { projectRead: false, proposals: false, applyNewFile: false, applyExistingFile: false, attachmentImport: true },
    availability: active ? 'busy' : 'ready', contributor: { termsRevision: '2026-10-02', accepted: true, eligible: true }, historyAvailable: true,
    budget: { accountId: 'e'.repeat(32), pending: [], configured: true, period: '2026-10', limitNanoGbp: '89000000000', spentNanoGbp: '0', reservedNanoGbp: '0', remainingNanoGbp: '89000000000', warning: false, requiresReconciliation: false, configurationExpiresAt: 1790964000 },
  });
  const copy = value => structuredClone(value);
  const exact = (value, keys) => value && Object.keys(value).sort().join('|') === [...keys].sort().join('|');
  const emit = (event, payload) => {
    for (const [id, listener] of listeners) if (listener.event === event) callbacks.get(listener.handler)?.({ event, id, payload });
  };
  const api = {
    get calls() { return copy(calls); },
    get unknown() { return [...unknown]; },
    get batch() { return copy(batch); },
    setSelection(value) { if (!['full', 'cancel', 'malformed', 'deny'].includes(value)) throw new Error('qa-scenario-denied'); selection = value; },
    releaseHydration() { release?.(); },
    emitClose() { emit('aster://close-request', null); },
  };
  Object.defineProperty(window, '__ASTER_SYNTHETIC_QA__', { value: api });
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener(_event, id) { listeners.delete(id); } };
  window.__TAURI_INTERNALS__ = {
    transformCallback(callback, once = false) { const id = ++callbackSequence; callbacks.set(id, value => { if (once) callbacks.delete(id); callback(value); }); return id; },
    unregisterCallback(id) { callbacks.delete(id); },
    async invoke(command, arguments_ = {}) {
      const input = arguments_.input;
      const trace = { command, atMillis: performance.now() };
      if (command === 'startup_shell_ready') Object.assign(trace, { frameCallbacks: frames, allImagesSettled: [...document.querySelectorAll('.app-shell img')].every(image => image.complete), protocolVersion: input?.protocolVersion });
      if (command === 'send_message') Object.assign(trace, { dataClass: input?.dataClass, reasoningEffort: input?.reasoningEffort, attachmentCount: input?.attachmentIds?.length, syntheticMessage: input?.message === 'Synthetic response cancellation fixture' });
      if (command === 'remove_attachment') trace.attachmentId = input?.attachmentId;
      calls.push(trace);
      switch (command) {
        case 'plugin:event|listen': {
          if (!['aster://stream', 'aster://close-request'].includes(arguments_.event) || !callbacks.has(arguments_.handler)) throw new Error('qa-event-denied');
          const id = ++listenerSequence; listeners.set(id, { event: arguments_.event, handler: arguments_.handler }); return id;
        }
        case 'plugin:event|unlisten': listeners.delete(arguments_.eventId); return null;
        case 'startup_shell_ready': if (!exact(input, ['protocolVersion']) || input.protocolVersion !== 1) throw new Error('qa-startup-denied'); return { accepted: true, visible: false };
        case 'get_app_status': await hydration; return status();
        case 'list_conversations': return [...records.values()].map(item => ({ id: item.id, title: item.title, updatedAt: item.updatedAt, entryCount: item.entries.length }));
        case 'create_conversation': {
          const id = `synthetic_chat_${++conversationSequence}`;
          const value = { id, title: `Synthetic chat ${conversationSequence}`, updatedAt: conversationSequence, entries: [] };
          records.set(id, value); return copy(value);
        }
        case 'load_conversation': if (!exact(input, ['conversationId']) || !records.has(input.conversationId)) throw new Error('qa-conversation-denied'); return copy(records.get(input.conversationId));
        case 'select_attachments': {
          if (!exact(input, ['conversationId']) || !records.has(input.conversationId) || active) throw new Error('qa-selection-denied');
          batch = [];
          await new Promise(resolve => setTimeout(resolve, 100));
          if (selection === 'deny') throw new Error('attachment_denied');
          if (selection === 'malformed') return [{ ...attachments[0], path: 'synthetic-forbidden-field' }];
          batch = selection === 'cancel' ? [] : copy(attachments);
          return copy(batch);
        }
        case 'remove_attachment': {
          if (!exact(input, ['conversationId', 'attachmentId']) || !records.has(input.conversationId)) throw new Error('qa-removal-denied');
          const removed = batch.some(item => item.id === input.attachmentId); batch = batch.filter(item => item.id !== input.attachmentId); return { removed };
        }
        case 'send_message': {
          if (!exact(input, ['conversationId', 'requestId', 'modelId', 'message', 'dataClass', 'reasoningEffort', 'attachmentIds']) || input.modelId !== 'muse-spark-1.3-contributor' || input.dataClass !== 'synthetic' || input.message !== 'Synthetic response cancellation fixture' || !records.has(input.conversationId) || active) throw new Error('qa-send-denied');
          active = copy(input);
          records.get(input.conversationId).entries = [{ id: 'synthetic_u', role: 'user', content: input.message, requestId: input.requestId, state: 'completed' }, { id: 'synthetic_a', role: 'assistant', content: '', requestId: input.requestId, state: 'cancelled' }];
          return { requestId: input.requestId, accepted: true };
        }
        case 'cancel_message': {
          if (!active || arguments_.requestId !== active.requestId) throw new Error('qa-cancel-denied');
          const value = active; active = null; batch = [];
          emit('aster://stream', { requestId: value.requestId, conversationId: value.conversationId, type: 'terminal', status: 'cancelled' });
          return { cancelled: true };
        }
        case 'window_action': if (!exact(input, ['action']) || !['minimise', 'toggle_maximise', 'toggle_fullscreen', 'start_dragging', 'close'].includes(input.action)) throw new Error('qa-window-denied'); return { maximised: false, fullscreen: false };
        default: unknown.push(command); throw new Error('qa-command-denied');
      }
    },
  };
}

let browser;
let context;
let page;
let releaseHeldModule = () => {};
let moduleGateTimer;
const capture = async name => {
  assert.match(name, /^[a-z0-9-]+$/);
  const filename = `${name}.png`;
  await page.screenshot({ path: path.join(outputDirectory, filename), fullPage: true });
  report.screenshots.push({ filename, sha256: sha256(await fs.readFile(path.join(outputDirectory, filename))), viewport: page.viewportSize() });
};
async function check(code, operation) {
  process.stderr.write(`[renderer-qa] ${code} START\n`);
  try { const evidence = await operation(); report.checks.push({ code, status: 'PASS', ...(evidence === undefined ? {} : { evidence }) }); process.stderr.write(`[renderer-qa] ${code} PASS\n`); }
  catch (error) { report.checks.push({ code, status: 'FAIL', reason: String(error.message).slice(0, 400) }); process.stderr.write(`[renderer-qa] ${code} FAIL\n`); }
}
async function bounded(promise, code, milliseconds = 6000) {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(code)), milliseconds); })]);
  } finally { clearTimeout(timer); }
}
const getTrace = async () => page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.calls);
async function classification(value = 'Synthetic') {
  await page.getByRole('button', { name: 'Reasoning effort', exact: true }).click();
  await page.getByRole('menuitem', { name: 'Classify content', exact: true }).click();
  await page.getByRole('menuitemradio', { name: value, exact: true }).click();
}
async function waitEnabled(locator) { await locator.waitFor({ state: 'visible' }); await page.waitForFunction(selector => !document.querySelector(selector)?.disabled, '.attachment-button'); }
try {
  assert.ok(report.rendererInputsMatchP1, 'renderer-inputs-do-not-match-source-p1');
  const require = createRequire(import.meta.url);
  const { chromium } = require(bundledPlaywright);
  process.stderr.write('[renderer-qa] isolated-headless-launch START\n');
  browser = await chromium.launch({ executablePath: edge, headless: true, chromiumSandbox: true, timeout: 15000 });
  process.stderr.write('[renderer-qa] isolated-headless-launch COMPLETE\n');
  report.browser.version = browser.version();
  process.stderr.write('[renderer-qa] ephemeral-context START\n');
  context = await bounded(browser.newContext({ viewport: { width: 1917, height: 1137 }, locale: 'en-GB', colorScheme: 'dark', deviceScaleFactor: 1, acceptDownloads: false, serviceWorkers: 'block' }), 'ephemeral-context-timeout');
  process.stderr.write('[renderer-qa] ephemeral-context COMPLETE\n');
  context.setDefaultTimeout(6000);
  await bounded(context.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin === origin) await route.continue();
    else { report.deniedExternalRequests.push({ origin: url.origin, type: route.request().resourceType() }); await route.abort('blockedbyclient'); }
  }), 'request-denial-setup-timeout');
  process.stderr.write('[renderer-qa] fake-host-injection START\n');
  await bounded(context.addInitScript(installFake, { slowHydration: true }), 'fake-host-setup-timeout');
  page = await bounded(context.newPage(), 'ephemeral-page-timeout');
  process.stderr.write('[renderer-qa] fake-host-injection COMPLETE\n');
  page.on('pageerror', error => report.pageErrors.push({ name: error.name, message: error.message.slice(0, 400) }));
  let releaseModule;
  const moduleGate = new Promise(resolve => { releaseModule = resolve; });
  releaseHeldModule = releaseModule;
  moduleGateTimer = setTimeout(releaseHeldModule, 12000);
  await bounded(page.route('**/assets/index-*.js', async route => { await moduleGate; await route.continue(); }), 'module-gate-setup-timeout');
  process.stderr.write('[renderer-qa] loopback-navigation START\n');
  await bounded(page.goto(origin, { waitUntil: 'commit', timeout: 6000 }), 'loopback-navigation-timeout');
  process.stderr.write('[renderer-qa] loopback-navigation COMPLETE\n');
  await check('T-083-early-dark-compiled-surface', async () => {
    await page.waitForFunction(() => document.querySelector('#root') && getComputedStyle(document.body).backgroundColor === 'rgb(27, 31, 32)');
    const colours = await page.evaluate(() => [document.documentElement, document.body, document.querySelector('#root')].map(element => ({ background: getComputedStyle(element).backgroundColor, visibility: getComputedStyle(element).visibility, display: getComputedStyle(element).display, opacity: getComputedStyle(element).opacity })));
    assert.ok(colours.every(item => item.background === 'rgb(27, 31, 32)' && item.visibility === 'visible' && item.display !== 'none' && item.opacity !== '0'), 'early-surface-is-not-opaque-dark');
    assert.equal(await page.locator('.app-shell').count(), 0, 'module-was-not-held');
    // CDP captures the already rendered surface without waiting for fonts.ready,
    // which can depend on DOMContentLoaded while a module is intentionally held.
    const session = await bounded(context.newCDPSession(page), 'early-capture-session-timeout');
    try {
      const result = await bounded(session.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false, fromSurface: true }), 'early-capture-timeout');
      const filename = 'pre-module-dark-surface.png';
      const bytes = Buffer.from(result.data, 'base64');
      await fs.writeFile(path.join(outputDirectory, filename), bytes);
      report.screenshots.push({ filename, sha256: sha256(bytes), viewport: page.viewportSize(), method: 'CDP surface capture while application module is held' });
    } finally { await bounded(session.detach(), 'capture-session-detach-timeout'); }
    return colours;
  });
  releaseModule();
  clearTimeout(moduleGateTimer);
  await page.waitForSelector('.app-shell');
  await check('T-077-shell-signal-independent-of-hydration', async () => {
    await page.waitForFunction(() => window.__ASTER_SYNTHETIC_QA__.calls.some(item => item.command === 'startup_shell_ready'));
    const calls = (await getTrace()).filter(item => item.command === 'startup_shell_ready');
    assert.equal(calls.length, 1, 'shell-ready-not-one-shot');
    assert.ok(calls[0].frameCallbacks >= 2 && calls[0].allImagesSettled && calls[0].protocolVersion === 1, 'shell-ready-scheduling-mismatch');
    assert.equal(await page.getByRole('button', { name: 'Add attachments', exact: true }).isDisabled(), true, 'hydration-was-not-held');
    return { ...calls[0], nativeRevealOrPaintProven: false };
  });
  await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.releaseHydration());
  const plus = page.getByRole('button', { name: 'Add attachments', exact: true });
  const draft = page.getByRole('textbox', { name: 'Message for Aster', exact: true });
  await waitEnabled(plus);
  await check('T-075-plus-uniform-surface-and-geometry', async () => {
    const geometry = await plus.evaluate(button => {
      const style = getComputedStyle(button), composer = button.closest('.composer'), model = composer.querySelector('.fixed-model');
      const r = button.getBoundingClientRect(), m = model.getBoundingClientRect(), c = composer.getBoundingClientRect();
      return { background: style.backgroundColor, border: style.borderWidth, radius: style.borderRadius, width: r.width, height: r.height, glyphWidth: button.querySelector('svg').getAttribute('width'), modelX: m.x, right: r.right, composerBackground: getComputedStyle(composer).backgroundColor, composerContains: r.left >= c.left && r.right <= c.right && r.top >= c.top && r.bottom <= c.bottom };
    });
    assert.equal(geometry.background, 'rgba(0, 0, 0, 0)', 'plus-has-background-tile');
    assert.equal(geometry.border, '0px', 'plus-has-border');
    assert.equal(geometry.radius, '0px', 'plus-has-frame');
    assert.equal(geometry.width, 30, 'plus-width-changed'); assert.equal(geometry.height, 34, 'plus-height-changed'); assert.equal(geometry.glyphWidth, '22', 'plus-glyph-changed');
    assert.equal(geometry.composerBackground, 'rgb(36, 38, 39)', 'composer-colour-mismatch');
    assert.ok(geometry.composerContains && geometry.right <= geometry.modelX, 'plus-placement-mismatch');
    await capture('empty-renderer-1917x1137'); return geometry;
  });
  await check('T-076-independent-medium-and-child-classification', async () => {
    assert.match(await page.getByRole('button', { name: 'Reasoning effort', exact: true }).innerText(), /Medium/);
    assert.equal(await page.getByLabel('Content classification', { exact: true }).textContent(), 'Not classified');
    await page.getByRole('button', { name: 'Reasoning effort', exact: true }).click();
    const popup = page.getByRole('menu', { name: 'Reasoning effort options', exact: true });
    const order = await popup.locator('button').allTextContents();
    assert.deepEqual(order.map(item => item.replace('›', '').trim()), ['Minimal', 'Low', 'Medium', 'High', 'Extra high', 'Classify content'], 'popup-order-mismatch');
    const extra = await page.getByRole('menuitemradio', { name: 'Extra high', exact: true }).boundingBox();
    const child = page.getByRole('menuitem', { name: 'Classify content', exact: true });
    const childBox = await child.boundingBox(); assert.ok(childBox.y > extra.y + extra.height, 'classification-not-below-extra-high');
    await child.click();
    await page.getByRole('menu', { name: 'Content classification options', exact: true }).waitFor();
    await capture('classification-child-popup');
    await page.getByRole('menuitemradio', { name: 'Synthetic', exact: true }).click();
    assert.match(await page.getByRole('button', { name: 'Reasoning effort', exact: true }).innerText(), /Medium/);
    assert.equal(await page.getByLabel('Content classification', { exact: true }).textContent(), 'Synthetic');
    await page.getByRole('button', { name: 'Reasoning effort', exact: true }).click();
    await page.getByRole('menuitemradio', { name: 'High', exact: true }).click();
    assert.equal(await page.getByLabel('Content classification', { exact: true }).textContent(), 'Synthetic');
    await page.getByRole('button', { name: 'Reasoning effort', exact: true }).click();
    await page.getByRole('menuitemradio', { name: 'Medium', exact: true }).click();
    return { order, classificationDoesNotChangeEffort: true, effortDoesNotChangeClassification: true };
  });
  await draft.fill('Synthetic draft retained through selection');
  await check('T-075-complete-batch-metadata-draft-and-removal', async () => {
    await plus.click(); await page.getByRole('list', { name: 'Temporary attachments', exact: true }).waitFor(); await waitEnabled(plus);
    assert.equal(await draft.inputValue(), 'Synthetic draft retained through selection', 'selection-lost-draft');
    const rows = page.getByRole('list', { name: 'Temporary attachments', exact: true }).locator('li');
    assert.equal(await rows.count(), 4, 'batch-not-complete');
    const details = await rows.allTextContents();
    assert.ok(details[0].includes('2 pages') && details[0].includes('2,048 bytes') && details[1].includes('Image') && details[2].includes('Text'), 'batch-metadata-mismatch');
    assert.equal(await page.getByLabel('Content classification', { exact: true }).textContent(), 'Not classified', 'selection-retained-classification');
    await capture('complete-synthetic-attachments');
    await classification();
    await page.getByRole('button', { name: 'Remove attachment Synthetic notes.txt', exact: true }).click(); await waitEnabled(plus);
    assert.equal(await rows.count(), 3, 'attachment-removal-not-reflected');
    assert.equal(await draft.inputValue(), 'Synthetic draft retained through selection', 'removal-lost-draft');
    assert.equal(await page.getByLabel('Content classification', { exact: true }).textContent(), 'Not classified', 'removal-retained-classification');
    return { initialCount: 4, afterRemoval: 3, displayedMetadata: details, textDraftRetained: true };
  });
  await check('T-075-selector-cancellation-replaces-batch-only', async () => {
    await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.setSelection('cancel'));
    await plus.click(); await waitEnabled(plus);
    assert.equal(await page.getByRole('list', { name: 'Temporary attachments', exact: true }).count(), 0, 'cancel-restored-old-batch');
    assert.equal(await draft.inputValue(), 'Synthetic draft retained through selection', 'cancel-lost-draft');
    assert.ok(await page.getByText('No replacement attachments were selected.', { exact: false }).isVisible(), 'cancel-notice-missing');
  });
  await check('T-075-malformed-receipt-fails-closed', async () => {
    await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.setSelection('malformed'));
    await plus.click(); await waitEnabled(plus);
    assert.equal(await page.getByRole('list', { name: 'Temporary attachments', exact: true }).count(), 0, 'malformed-receipt-was-admitted');
    assert.equal(await draft.inputValue(), 'Synthetic draft retained through selection', 'malformed-receipt-lost-draft');
    assert.ok(await page.getByText('The host returned invalid attachment metadata.', { exact: false }).isVisible(), 'metadata-error-not-safe');
  });
  await check('T-075-chat-switch-revokes-only-current-import-ids', async () => {
    await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.setSelection('full'));
    await plus.click(); await page.getByRole('list', { name: 'Temporary attachments', exact: true }).waitFor(); await waitEnabled(plus);
    await page.getByRole('button', { name: 'New conversation', exact: true }).click(); await waitEnabled(plus);
    assert.equal(await page.getByRole('list', { name: 'Temporary attachments', exact: true }).count(), 0, 'chat-switch-restored-imports');
    assert.equal(await draft.inputValue(), '', 'new-chat-inherited-draft');
    await page.getByRole('button', { name: /Synthetic chat 1/ }).click(); await waitEnabled(plus);
    assert.equal(await draft.inputValue(), 'Synthetic draft retained through selection', 'prior-chat-draft-not-restored');
    assert.equal(await page.getByRole('list', { name: 'Temporary attachments', exact: true }).count(), 0, 'prior-chat-restored-imports');
    return { restoredTextOnly: true, freshAttachmentSelectionRequired: true };
  });
  await check('T-075-long-text-scroll-without-truncation', async () => {
    const synthetic = Array.from({ length: 120 }, (_, i) => `Synthetic line ${i}: unchanged draft.`).join('\n');
    await draft.fill(synthetic);
    const geometry = await draft.evaluate(element => ({ clientHeight: element.clientHeight, scrollHeight: element.scrollHeight, overflowY: getComputedStyle(element).overflowY, maxHeight: getComputedStyle(element).maxHeight, characters: element.value.length }));
    assert.equal(await draft.inputValue(), synthetic, 'long-draft-truncated');
    assert.ok(geometry.scrollHeight > geometry.clientHeight && geometry.clientHeight <= 220 && geometry.overflowY === 'auto', 'long-draft-scroll-missing');
    await capture('long-draft-scroll'); return geometry;
  });
  await draft.fill('Synthetic draft retained through selection');
  await check('T-073-zoom-keyboard-wheel-and-cleanable-hud', async () => {
    const zoom = page.getByLabel('Interface zoom', { exact: true });
    const hud = page.getByRole('status', { name: 'Zoom level', exact: true });
    assert.equal(await zoom.textContent(), '100%');
    await page.keyboard.press('Control+='); await hud.waitFor({ state: 'visible' });
    assert.equal(await zoom.textContent(), '110%'); assert.equal(await hud.textContent(), '110%');
    const geometry = await hud.evaluate(element => { const r = element.getBoundingClientRect(); return { right: r.right, top: r.top, width: r.width, pointerEvents: getComputedStyle(element).pointerEvents, parentIsBody: element.parentElement.id === 'root', scaledShellContainsHud: Boolean(element.closest('.app-shell')), viewport: { width: innerWidth, height: innerHeight } }; });
    assert.ok(geometry.right <= geometry.viewport.width && geometry.right >= geometry.viewport.width - 20 && geometry.top < 130 && geometry.pointerEvents === 'none' && !geometry.scaledShellContainsHud, 'hud-layout-or-pointer-mismatch');
    await page.waitForTimeout(700); assert.equal(await hud.isVisible(), true, 'hud-expired-before-one-second');
    await page.keyboard.press('Control+='); assert.equal(await hud.textContent(), '120%');
    await page.waitForTimeout(700); assert.equal(await hud.isVisible(), true, 'latest-zoom-did-not-reset-timer');
    await page.waitForTimeout(450); assert.equal(await hud.isVisible(), false, 'hud-did-not-expire');
    await page.keyboard.press('Control+-'); assert.equal(await zoom.textContent(), '110%');
    await page.keyboard.press('Control+0'); assert.equal(await zoom.textContent(), '100%');
    await page.evaluate(() => window.dispatchEvent(new WheelEvent('wheel', { deltaY: -100, ctrlKey: true, cancelable: true, bubbles: true })));
    assert.equal(await zoom.textContent(), '110%');
    await page.evaluate(() => window.dispatchEvent(new WheelEvent('wheel', { deltaY: 100, ctrlKey: true, cancelable: true, bubbles: true })));
    assert.equal(await zoom.textContent(), '100%');
    for (let i = 0; i < 7; i++) await page.keyboard.press('Control+='); assert.equal(await zoom.textContent(), '150%');
    for (let i = 0; i < 9; i++) await page.keyboard.press('Control+-'); assert.equal(await zoom.textContent(), '80%');
    await page.keyboard.press('Control+0'); assert.equal(await zoom.textContent(), '100%');
    return { range: '80–150%', default: '100%', noticeDurationMilliseconds: 1000, geometry, wheelDelivery: 'synthetic DOM WheelEvent; no native zoom proof' };
  });
  await check('T-075-modal-lock-and-default-cancel', async () => {
    await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.getByRole('dialog').waitFor();
    assert.equal(await plus.isDisabled(), true, 'behind-dialogue-import-enabled');
    await page.keyboard.press('Control+='); assert.equal(await page.getByLabel('Interface zoom', { exact: true }).textContent(), '100%', 'behind-dialogue-zoom-changed');
    await page.getByRole('button', { name: 'Close dialogue', exact: true }).click(); await waitEnabled(plus);
    await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.emitClose());
    await page.getByRole('dialog', { name: 'Unsent drafts', exact: true }).waitFor();
    assert.equal(await page.getByRole('button', { name: 'Keep working', exact: true }).evaluate(element => element === document.activeElement), true, 'quit-not-default-cancel');
    await page.getByRole('button', { name: 'Keep working', exact: true }).click(); await waitEnabled(plus);
    assert.equal(await draft.inputValue(), 'Synthetic draft retained through selection', 'default-cancel-lost-draft');
  });
  await check('T-075-synthetic-response-stop-and-busy-denials', async () => {
    await draft.fill('Synthetic response cancellation fixture'); await classification();
    const send = page.getByRole('button', { name: 'Send message', exact: true }); await send.click();
    const stop = page.getByRole('button', { name: 'Stop response', exact: true }); await stop.waitFor();
    assert.equal(await plus.isDisabled(), true, 'busy-import-enabled');
    assert.equal(await page.getByRole('button', { name: 'Reasoning effort', exact: true }).isDisabled(), true, 'busy-classification-enabled');
    const colour = await stop.evaluate(element => ({ colour: getComputedStyle(element).backgroundColor, width: element.getBoundingClientRect().width, height: element.getBoundingClientRect().height }));
    assert.deepEqual(colour, { colour: 'rgb(100, 195, 253)', width: 44, height: 44 }, 'stop-geometry-or-colour-changed');
    await stop.click(); await waitEnabled(plus);
    assert.equal(await page.getByRole('button', { name: 'Stop response', exact: true }).count(), 0, 'stop-did-not-terminate-fake-request');
    const traces = await getTrace(); assert.equal(traces.filter(item => item.command === 'send_message').length, 1, 'fake-send-repeated');
    assert.equal(traces.filter(item => item.command === 'cancel_message').length, 1, 'fake-cancel-repeated');
    assert.ok(traces.find(item => item.command === 'send_message').syntheticMessage, 'non-synthetic-send');
    return { fakeHostOnly: true, noNetworkRequest: true, stop: colour };
  });
  await check('T-083-compact-1366x768-composer-and-popup', async () => {
    await page.setViewportSize({ width: 1366, height: 768 }); await draft.fill('');
    const composer = await page.locator('.composer').boundingBox();
    assert.ok(composer.x >= 0 && composer.y >= 0 && composer.x + composer.width <= 1366 && composer.y + composer.height <= 768, 'compact-composer-clipped');
    await page.getByRole('button', { name: 'Reasoning effort', exact: true }).click();
    await page.getByRole('menuitem', { name: 'Classify content', exact: true }).click();
    const popup = await page.getByRole('menu', { name: 'Content classification options', exact: true }).boundingBox();
    assert.ok(popup.x >= 0 && popup.y >= 0 && popup.x + popup.width <= 1366 && popup.y + popup.height <= 768, 'compact-child-popup-clipped');
    await capture('compact-renderer-1366x768');
    await page.keyboard.press('Escape'); await page.keyboard.press('Escape'); return { composer, classificationPopup: popup };
  });
  await check('T-083-no-external-or-unregistered-host-operation', async () => {
    const trace = await getTrace();
    report.fakeHostCalls = trace;
    report.unexpectedFakeCommands = await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__.unknown);
    assert.equal(report.unexpectedFakeCommands.length, 0, 'unregistered-fake-host-command');
    assert.equal(report.deniedExternalRequests.length, 0, 'renderer-attempted-external-network');
    assert.equal(report.pageErrors.length, 0, 'renderer-page-error');
    assert.equal(trace.filter(item => item.command === 'startup_shell_ready').length, 1, 'shell-ready-repeated');
    return { servedLocalResources: [...new Set(served)].sort(), fakeCommands: [...new Set(trace.map(item => item.command))].sort() };
  });
} catch (error) {
  report.checks.push({ code: 'renderer-harness-execution', status: 'FAIL', reason: String(error.message).slice(0, 600) });
} finally {
  releaseHeldModule();
  clearTimeout(moduleGateTimer);
  if (page && !page.isClosed()) {
    try { report.fakeHostCalls ??= await getTrace(); report.unexpectedFakeCommands = await page.evaluate(() => window.__ASTER_SYNTHETIC_QA__?.unknown ?? []); } catch { /* Retain the earlier execution failure. */ }
  }
  try { if (context) await bounded(context.close(), 'context-close-timeout'); }
  catch (error) { report.checks.push({ code: 'context-teardown', status: 'FAIL', reason: error.message }); }
  try { if (browser) await bounded(browser.close(), 'browser-close-timeout'); }
  catch (error) { report.checks.push({ code: 'browser-teardown', status: 'FAIL', reason: error.message }); }
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
  // Bind to the exact bytes served, then independently detect a changed dist.
  const after = [];
  for (const [name] of files) after.push(`${name}\0${sha256(await fs.readFile(path.join(dist, name.slice(1))))}\n`);
  const afterFingerprint = sha256(after.sort((a, b) => a.localeCompare(b, 'en-GB')).join(''));
  report.compiledDistUnchanged = afterFingerprint === distFingerprint;
  if (!report.compiledDistUnchanged) report.checks.push({ code: 'compiled-dist-stability', status: 'FAIL', reason: 'compiled-dist-changed-during-run' });
  report.completedUtc = new Date().toISOString();
  const failedExecution = report.checks.find(item => item.code === 'renderer-harness-execution');
  for (const code of requiredChecks) if (!report.checks.some(item => item.code === code)) report.checks.push({ code, status: 'NOT RUN', reason: failedExecution?.reason ?? 'earlier-harness-failure' });
  report.allPassed = report.checks.length > 0 && report.checks.every(item => item.status === 'PASS') && report.compiledDistUnchanged;
  await fs.writeFile(path.join(outputDirectory, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify({ classification: report.classification, sourceP1, allPassed: report.allPassed, checks: report.checks.map(({ code, status, reason }) => ({ code, status, ...(reason ? { reason } : {}) })), report: path.join(outputDirectory, 'report.json') }, null, 2)}\n`);
  process.exitCode = report.allPassed ? 0 : 1;
  // Only this harness process is terminated if Playwright's owned transport has
  // an outstanding request after bounded close. Never enumerate or kill Edge.
  setTimeout(() => process.exit(process.exitCode), 1000).unref();
}
