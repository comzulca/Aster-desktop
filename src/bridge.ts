import type { AppStatus, Bridge, StreamEvent } from './types';

function isNative(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function callHost<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isNative()) throw new Error('HOST_UNAVAILABLE');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export const nativeBridge: Bridge = {
  getStatus: () => callHost<AppStatus>('get_app_status'),
  async subscribe(callback: (event: StreamEvent) => void) {
    if (!isNative()) throw new Error('HOST_UNAVAILABLE');
    const { listen } = await import('@tauri-apps/api/event');
    return listen<StreamEvent>('aster://stream', (event) => callback(event.payload));
  },
  configureCredential: () => callHost('configure_nvidia_credential'),
  removeCredential: () => callHost('remove_nvidia_credential'),
  selectProject: () => callHost('select_project_root'),
  selectProjectFile: (args) => callHost('select_project_file', { input: args }),
  sendMessage: (args) => callHost('send_message', { input: args }),
  sendProjectMessage: (args) => callHost('send_project_message', { input: args }),
  reviewProjectProposal: (args) => callHost('review_project_proposal', { input: args }),
  cancelMessage: (requestId) => callHost('cancel_message', { requestId }),
};
