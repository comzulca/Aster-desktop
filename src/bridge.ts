import type { Bridge, StreamEvent } from './types';
import { validGenerationIntent, validGenerationReceipt } from './generation-ui';

async function callHost<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) throw new Error('host_unavailable');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}
export const nativeBridge: Bridge = {
  shellReady: input => callHost('startup_shell_ready', { input }),
  recordRenderEvent: input => callHost('record_render_event', { input }),
  getRenderTrace: () => callHost('get_render_trace'),
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
  verifyCredential: () => callHost('verify_meta_credential'),
  getPersonalization: () => callHost('get_personalization'),
  savePersonalization: input => callHost('save_personalization', { input }),
  clearPersonalization: input => callHost('clear_personalization', { input }),
  getCustomization: () => callHost('get_customization'),
  getCustomizationPackage: input => callHost('get_customization_package', { input }),
  saveCustomization: input => callHost('save_customization', { input }),
  setCustomizationEnabled: input => callHost('set_customization_enabled', { input }),
  importCustomization: input => callHost('import_customization', { input }),
  async generateCustomization(input) {
    if (!validGenerationIntent(input)) throw new Error('generation_intent_invalid');
    const receipt: unknown = await callHost('generate_customization', { input });
    if (!validGenerationReceipt(receipt, input.requestId)) throw new Error('generation_receipt_unavailable');
    return receipt;
  },
  getGitHubStatus: () => callHost('get_github_status'),
  beginGitHubConnection: () => callHost('begin_github_connection'),
  pollGitHubConnection: () => callHost('poll_github_connection'),
  cancelGitHubConnection: () => callHost('cancel_github_connection'),
  disconnectGitHub: () => callHost('disconnect_github'),
  listGitHubRepositories: () => callHost('list_github_repositories'),
  selectGitHubRepository: input => callHost('select_github_repository', { input }),
  readGitHubFile: input => callHost('read_github_file', { input }),
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
