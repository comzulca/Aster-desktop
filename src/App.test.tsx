import { act, cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { DEFAULT_MODEL, MUSE_MODEL, type AppStatus, type Bridge, type StreamEvent } from './types';

const baseline: AppStatus = {
  policyVersion: 1,
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
    sendMessage: vi.fn(async (args) => ({ requestId: args.requestId, accepted: true })),
    cancelMessage: vi.fn(async () => ({ cancelled: true })),
  };
  return { bridge, emit: (event: StreamEvent) => onEvent(event) };
}

afterEach(() => cleanup());

describe('Aster UI: host boundary and user intent', () => {
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
    expect((screen.getByRole('button', { name: 'Proponer cambio (no disponible)' }) as HTMLButtonElement).disabled).toBe(true);
  });

  it('root selection grants no file read or file disclosure through the UI', async () => {
    const fake = fakeBridge({ ...baseline, projectSelected: true, projectLabel: 'Proyecto elegido' });
    render(<App bridge={fake.bridge}/>);
    await screen.findByText('Clave configurada · conexión no verificada');
    expect(screen.getByText(/Lectura y cambios de archivos: pendientes/)).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Adjuntar archivo (no disponible)' }) as HTMLButtonElement).disabled).toBe(true);
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
    expect((screen.getByRole('button', { name: 'Proponer cambio (no disponible)' }) as HTMLButtonElement).disabled).toBe(true);
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
});
