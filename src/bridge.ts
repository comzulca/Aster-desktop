import type { Bridge, StreamEvent } from './types';

async function callHost<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) throw new Error('host_unavailable');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}
export const nativeBridge: Bridge = {
  shellReady: input => callHost('startup_shell_ready', { input }),
  selectAttachments: input => callHost('select_attachments', { input }),
  removeAttachment: input => callHost('remove_attachment', { input }),
  viewProcessedAttachment: input => callHost('review_processed_attachment', { input }),
  windowAction: input => callHost('window_action', { input }),
  async subscribeClose(callback) {
    if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) throw new Error('host_unavailable');
    const { listen } = await import('@tauri-apps/api/event');
    return listen('aster://close-request', () => callback());
  },
  getStatus: () => callHost('get_app_status'),
  async subscribe(callback) {
    if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) throw new Error('host_unavailable');
    const { listen } = await import('@tauri-apps/api/event');
    return listen<StreamEvent>('aster://stream', event => callback(event.payload));
  },
  configureCredential: () => callHost('configure_meta_credential'),
  removeCredential: () => callHost('remove_meta_credential'),
  acceptContributorTerms: () => callHost('accept_contributor_terms'),
  configureBudget: input => callHost('configure_budget', { input }),
  reconcileBudget: input => callHost('reconcile_budget', { input }),
  listConversations: () => callHost('list_conversations'),
  createConversation: () => callHost('create_conversation'),
  loadConversation: input => callHost('load_conversation', { input }),
  deleteConversation: input => callHost('delete_conversation', { input }),
  sendMessage: input => callHost('send_message', { input }),
  cancelMessage: requestId => callHost('cancel_message', { requestId }),
};
