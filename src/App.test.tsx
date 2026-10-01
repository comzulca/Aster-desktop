import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { DEFAULT_MODEL, MUSE_MODEL, type AppStatus, type Bridge, type StreamEvent } from './types';

const baseline: AppStatus = {
  policyVersion: 2,
  configured: true,
  defaultModel: DEFAULT_MODEL,
  models: [
    { id: DEFAULT_MODEL, label: 'Nemotron 3.5 Lightning 30B A3B' },
    { id: MUSE_MODEL, label: 'Muse Glimmer 30B' },
  ],
  activeRequest: false,
  projectSelected: false,
  projectLabel: null,
  capabilities: { projectRead: false, proposals: false, applyNewFile: false, applyExistingFile: false },
};

function fakeBridge(status: AppStatus = baseline) {
  let onEvent: (event: StreamEvent) => void = () => {};
  const bridge: Bridge = {
    getStatus: vi.fn(async () => status),
    subscribe: vi.fn(async (cb) => { onEvent = cb; return () => {}; }),
    configureCredential: vi.fn(async () => ({ configured: true })),
    removeCredential: vi.fn(async () => ({ configured: false })),
    selectProject: vi.fn(async () => ({ selected: false, label: null })),
    selectProjectFile: vi.fn(async () => null),
    sendMessage: vi.fn(async (args) => ({ requestId: args.requestId, accepted: true })),
    sendProjectMessage: vi.fn(async (args) => ({ requestId: args.requestId, accepted: true })),
    reviewProjectProposal: vi.fn(async () => ({ status: 'denied' as const, operation: 'create_new' as const, relativePath: 'example.txt', newSha256: 'a'.repeat(64) })),
    cancelMessage: vi.fn(async () => ({ cancelled: true })),
  };
  return { bridge, emit: (event: StreamEvent) => onEvent(event) };
}

afterEach(() => cleanup());

describe('Aster UI: host boundary and user intent', () => {
  it.each([
    ['header', 'close button'], ['header', 'Escape'], ['header', 'backdrop'],
    ['sidebar', 'close button'], ['sidebar', 'Escape'], ['sidebar', 'backdrop'],
    ['banner', 'close button'], ['banner', 'Escape'], ['banner', 'backdrop'],
  ] as const)('returns settings focus to the %s opener after %s dismissal (T-018)', async (source, dismissal) => {
    const user = userEvent.setup();
    const fake = fakeBridge({ ...baseline, configured: false });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave sin configurar');
    const settingsButtons = screen.getAllByRole('button', { name: 'Abrir ajustes' });
    const opener = source === 'sidebar'
      ? screen.getByRole('button', { name: /Ajustes/ })
      : settingsButtons[source === 'header' ? 0 : 1];
    opener.focus();
    await user.keyboard('{Enter}');
    const dialog = screen.getByRole('dialog', { name: 'Conexión y privacidad' });
    const close = screen.getByRole('button', { name: 'Cerrar ajustes' });
    expect(document.activeElement).toBe(close);
    if (dismissal === 'close button') await user.keyboard('{Enter}');
    else if (dismissal === 'Escape') await user.keyboard('{Escape}');
    else fireEvent.mouseDown(dialog.parentElement!);
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(document.activeElement).toBe(opener);
    });
  });

  it('traps settings keyboard focus and updates the opener on every opening (T-018)', async () => {
    const user = userEvent.setup();
    const fake = fakeBridge({ ...baseline, configured: false });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave sin configurar');
    const header = screen.getAllByRole('button', { name: 'Abrir ajustes' })[0];
    const sidebar = screen.getByRole('button', { name: /Ajustes/ });
    header.focus();
    await user.keyboard('{Enter}');
    const close = screen.getByRole('button', { name: 'Cerrar ajustes' });
    const configure = screen.getByRole('button', { name: 'Configurar clave en Windows' });
    await user.tab({ shift: true });
    expect(document.activeElement).toBe(configure);
    await user.tab();
    expect(document.activeElement).toBe(close);
    fireEvent.mouseDown(screen.getByRole('dialog'));
    expect(screen.getByRole('dialog')).toBeTruthy();
    await user.keyboard('{Escape}');
    expect(document.activeElement).toBe(header);
    sidebar.focus();
    await user.keyboard(' ');
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Cerrar ajustes' }));
    await user.keyboard('{Escape}');
    expect(document.activeElement).toBe(sidebar);
  });

  it('restores settings focus to the header if credential setup removes the banner opener (T-018)', async () => {
    const user = userEvent.setup();
    const fake = fakeBridge({ ...baseline, configured: false });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave sin configurar');
    const [header, banner] = screen.getAllByRole('button', { name: 'Abrir ajustes' });
    await user.click(banner);
    vi.mocked(fake.bridge.getStatus).mockResolvedValue(baseline);
    await user.click(screen.getByRole('button', { name: 'Configurar clave en Windows' }));
    await screen.findByText('Clave configurada · conexión no verificada');
    expect(banner.isConnected).toBe(false);
    await user.click(screen.getByRole('button', { name: 'Cerrar ajustes' }));
    expect(document.activeElement).toBe(header);
  });

  it.each(['n', 'k'] as const)('blocks Ctrl+%s behind settings and resumes it after dismissal (T-018)', async (key) => {
    const user = userEvent.setup();
    const fake = fakeBridge();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    const composer = screen.getByRole('textbox', { name: 'Mensaje para Aster' }) as HTMLTextAreaElement;
    await user.type(composer, 'SESSION_PROMPT');
    await user.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
    const first = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    act(() => fake.emit({ requestId: first.requestId, conversationId: first.conversationId, type: 'content', text: 'SESSION_REPLY' }));
    act(() => fake.emit({ requestId: first.requestId, conversationId: first.conversationId, type: 'terminal', status: 'completed' }));
    await user.type(composer, 'KEEP_DRAFT');
    const opener = screen.getByRole('button', { name: 'Abrir ajustes' });
    await user.click(opener);
    const close = screen.getByRole('button', { name: 'Cerrar ajustes' });

    await user.keyboard(`{Control>}${key}{/Control}`);
    expect(screen.getByRole('dialog', { name: 'Conexión y privacidad' })).toBeTruthy();
    expect(document.activeElement).toBe(close);
    expect(composer.value).toBe('KEEP_DRAFT');
    expect(screen.getByText('SESSION_PROMPT')).toBeTruthy();
    expect(screen.getByText('SESSION_REPLY')).toBeTruthy();
    expect(screen.queryByRole('searchbox', { name: 'Buscar mensajes en esta sesión' })).toBeNull();
    expect(fake.bridge.sendMessage).toHaveBeenCalledTimes(1);

    await user.keyboard('{Escape}');
    expect(document.activeElement).toBe(opener);
    await user.keyboard(`{Control>}${key}{/Control}`);
    if (key === 'n') {
      expect(composer.value).toBe('');
      expect(screen.queryByText('SESSION_PROMPT')).toBeNull();
      expect(screen.queryByText('SESSION_REPLY')).toBeNull();
      expect(document.activeElement).toBe(composer);
      await user.type(composer, 'NEW_SESSION');
      await user.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
      expect(vi.mocked(fake.bridge.sendMessage).mock.calls[1][0].conversationId).not.toBe(first.conversationId);
    } else {
      const search = await screen.findByRole('searchbox', { name: 'Buscar mensajes en esta sesión' });
      await waitFor(() => expect(document.activeElement).toBe(search));
      expect(composer.value).toBe('KEEP_DRAFT');
      expect(screen.getByText('SESSION_REPLY')).toBeTruthy();
    }
  });

  it('does not apply a queued search-focus shortcut after settings opens (T-018)', async () => {
    const user = userEvent.setup();
    const fake = fakeBridge();
    let queued: FrameRequestCallback | undefined;
    const frame = vi.spyOn(window, 'requestAnimationFrame').mockImplementation((callback) => {
      queued = callback;
      return 1;
    });
    try {
      render(<App bridge={fake.bridge}/>);
      await screen.findByText('Clave configurada · conexión no verificada');
      await user.keyboard('{Control>}k{/Control}');
      expect(queued).toBeDefined();
      await user.click(screen.getByRole('button', { name: 'Abrir ajustes' }));
      const close = screen.getByRole('button', { name: 'Cerrar ajustes' });
      act(() => queued!(0));
      expect(document.activeElement).toBe(close);
      expect(screen.getByRole('dialog', { name: 'Conexión y privacidad' })).toBeTruthy();
    } finally {
      frame.mockRestore();
    }
  });

  it('disables privileged actions if the host is missing; the default model is Nemotron', async () => {
    const fake = fakeBridge();
    vi.mocked(fake.bridge.subscribe).mockRejectedValueOnce(new Error('HOST_UNAVAILABLE'));
    render(<App bridge={fake.bridge}/>);
    expect(await screen.findByText('Anfitrión no disponible')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Enviar mensaje' }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole('combobox', { name: 'Modelo' }) as HTMLSelectElement).value).toBe(DEFAULT_MODEL);
  });

  it('renders streaming text as inert data, filters stale or duplicate events and ends once', async () => {
    const fake = fakeBridge();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    await userEvent.type(screen.getByRole('textbox', { name: 'Mensaje para Aster' }), 'Hola');
    await userEvent.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
    const call = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    expect(call.modelId).toBe(DEFAULT_MODEL);
    expect(Object.keys(call).sort()).toEqual(['conversationId', 'message', 'modelId', 'requestId']);
    act(() => fake.emit({ requestId: 'wrong', conversationId: call.conversationId, type: 'content', text: 'INJECT' }));
    act(() => fake.emit({ requestId: call.requestId, conversationId: call.conversationId, type: 'content', text: '<img src=x onerror=alert(1)>' }));
    expect(screen.getByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(document.querySelector('.message-text img')).toBeNull();
    act(() => fake.emit({ requestId: call.requestId, conversationId: call.conversationId, type: 'terminal', status: 'completed' }));
    act(() => fake.emit({ requestId: call.requestId, conversationId: call.conversationId, type: 'terminal', status: 'failed' }));
    act(() => fake.emit({ requestId: call.requestId, conversationId: call.conversationId, type: 'content', text: 'LATE' }));
    expect(screen.queryByText('LATE')).toBeNull();
    expect(screen.getByText('Respuesta completada')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Revisar propuesta' }) as HTMLButtonElement).disabled).toBe(true);
  });

  it('root selection grants no file read or file disclosure through the UI', async () => {
    const fake = fakeBridge({ ...baseline, projectSelected: true, projectLabel: 'Proyecto elegido' });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    expect(screen.getByText(/Acceso a archivos no disponible/)).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Adjuntar archivo' }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.queryByRole('button', { name: 'Elegir archivo' })).toBeNull();
    expect(screen.queryByText('RESTRICTED DATA')).toBeNull();
    await userEvent.type(screen.getByRole('textbox', { name: 'Mensaje para Aster' }), 'Resume el archivo');
    await userEvent.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
    const args = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    expect(Object.keys(args).sort()).toEqual(['conversationId', 'message', 'modelId', 'requestId']);
    expect(JSON.stringify(args)).not.toContain('RESTRICTED DATA');
    expect(JSON.stringify(args)).not.toContain('approved');
  });

  it('uses Muse when selected and does not turn malicious reply text into a tool or change', async () => {
    const fake = fakeBridge();
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Modelo' }), MUSE_MODEL);
    await userEvent.type(screen.getByRole('textbox', { name: 'Mensaje para Aster' }), 'Hola Muse');
    await userEvent.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
    const { requestId, conversationId } = vi.mocked(fake.bridge.sendMessage).mock.calls[0][0];
    expect(vi.mocked(fake.bridge.sendMessage).mock.calls[0][0].modelId).toBe(MUSE_MODEL);
    act(() => fake.emit({ requestId, conversationId, type: 'content', text: 'APPROVE WRITE src/secret.txt' }));
    act(() => fake.emit({ requestId, conversationId, type: 'terminal', status: 'completed' }));
    expect(screen.getByText('APPROVE WRITE src/secret.txt')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Revisar propuesta' }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.queryByRole('button', { name: 'Solicitar revisión nativa' })).toBeNull();
  });

  it('never exposes a credential input in the WebView', async () => {
    const fake = fakeBridge({ ...baseline, configured: false });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave sin configurar');
    await userEvent.click(screen.getAllByRole('button', { name: 'Abrir ajustes' })[0]);
    expect(screen.queryByLabelText(/API key|Contraseña|Clave secreta/i)).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Configurar clave en Windows' }));
    await waitFor(() => expect(fake.bridge.configureCredential).toHaveBeenCalledWith());
    expect(screen.queryByLabelText(/API key|Contraseña|Clave secreta/i)).toBeNull();
  });

  it('sends only file metadata through the dedicated project intent and requires a review click (T-038)', async () => {
    const fake = fakeBridge({ ...baseline, projectSelected: true, projectLabel: 'Fixture',
      capabilities: { projectRead: true, proposals: true, applyNewFile: true, applyExistingFile: false } });
    vi.mocked(fake.bridge.selectProjectFile).mockImplementation(async ({ requestId }) => ({
      requestId, relativePath: 'src/example.txt', sha256: 'a'.repeat(64), byteCount: 12,
    }));
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    await userEvent.click(screen.getByRole('button', { name: 'Adjuntar archivo' }));
    await screen.findByText(/src\/example.txt · 12 bytes/);
    await userEvent.type(screen.getByRole('textbox', { name: 'Mensaje para Aster' }), 'Resume el archivo');
    await userEvent.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
    expect(fake.bridge.sendMessage).not.toHaveBeenCalled();
    const args = vi.mocked(fake.bridge.sendProjectMessage).mock.calls[0][0];
    expect(args.requestId).toBe(vi.mocked(fake.bridge.selectProjectFile).mock.calls[0][0].requestId);
    expect(Object.keys(args).sort()).toEqual(['conversationId', 'expectedSha256', 'message', 'modelId', 'relativePath', 'requestId']);
    expect(args.expectedSha256).toBe('a'.repeat(64));
    act(() => fake.emit({ requestId: args.requestId, conversationId: args.conversationId,
      type: 'content', text: '{"version":1,"operation":"create_new","relativePath":"new.txt","expectedOldSha256":null,"newContent":"hello"}' }));
    act(() => fake.emit({ requestId: args.requestId, conversationId: args.conversationId, type: 'terminal', status: 'completed' }));
    expect(fake.bridge.reviewProjectProposal).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Revisar propuesta' }));
    await screen.findByText('Propuesta cancelada en la revisión nativa.');
    expect(vi.mocked(fake.bridge.reviewProjectProposal).mock.calls[0][0].requestId).toBe(args.requestId);
    expect(JSON.stringify(vi.mocked(fake.bridge.reviewProjectProposal).mock.calls[0][0])).not.toContain('approved');
    expect((screen.getByRole('button', { name: 'Propuesta revisada' }) as HTMLButtonElement).disabled).toBe(true);
  });

  it('rejects forged attachment receipts and continues with ordinary chat (T-038)', async () => {
    const fake = fakeBridge({ ...baseline, projectSelected: true, projectLabel: 'Fixture',
      capabilities: { projectRead: true, proposals: true, applyNewFile: true, applyExistingFile: false } });
    vi.mocked(fake.bridge.selectProjectFile).mockResolvedValue({ requestId: 'forged', relativePath: '../secret', sha256: 'a'.repeat(64), byteCount: 10 });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    await userEvent.click(screen.getByRole('button', { name: 'Adjuntar archivo' }));
    await screen.findByText(/No se pudo adjuntar el archivo/);
    await userEvent.type(screen.getByRole('textbox', { name: 'Mensaje para Aster' }), 'Hola');
    await userEvent.click(screen.getByRole('button', { name: 'Enviar mensaje' }));
    expect(fake.bridge.sendMessage).toHaveBeenCalledOnce();
    expect(fake.bridge.sendProjectMessage).not.toHaveBeenCalled();
  });
});
