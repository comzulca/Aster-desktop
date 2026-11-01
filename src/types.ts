export const DEFAULT_MODEL = 'muse-spark-1.3-contributor' as const;
export type ModelId = typeof DEFAULT_MODEL;
export type DataClass = 'synthetic' | 'public_eligible';
export type ReasoningEffort = 'minimal' | 'low' | 'medium' | 'high' | 'xhigh';
export type WindowAction = 'minimise' | 'toggle_maximise' | 'toggle_fullscreen' | 'start_dragging' | 'close';
export type TerminalState = 'completed' | 'failed' | 'cancelled' | 'truncated' | 'refused';
export type EntryState = TerminalState | 'interrupted';
export type CredentialValidation = 'missing' | 'unverified' | 'valid' | 'invalid' | 'unavailable';
export interface AttachmentSummary {
  id: string;
  kind: 'text' | 'image' | 'pdf';
  label: string;
  sourceBytes: number;
  processedBytes: number;
  pageCount: number;
}

export interface ConversationEntry {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  requestId: string;
  state: EntryState;
  // Display-only references. These never restore temporary import IDs for
  // sending, original files, filesystem authority or disclosure approval.
  processedAttachments?: AttachmentSummary[];
}
export interface ConversationSummary { id: string; title: string; updatedAt: number; entryCount: number }
export interface Conversation extends Omit<ConversationSummary, 'entryCount'> { entries: ConversationEntry[] }
export interface BudgetIntent {
  accountId: string;
  gbpPerUsdPpm: number;
  marginBps: number;
  taxBps: number;
  paymentBps: number;
  validUntilUnix: number;
}
export interface SendMessageIntent {
  conversationId: string;
  requestId: string;
  modelId: ModelId;
  message: string;
  dataClass: DataClass;
  reasoningEffort: ReasoningEffort;
  attachmentIds?: string[];
}
export interface AppStatus {
  protocolVersion: 4;
  policyVersion: 4;
  configured: boolean;
  credentialAvailable: boolean;
  credentialValidation: CredentialValidation;
  defaultModel: ModelId;
  models: { id: ModelId; label: string }[];
  activeRequest: boolean;
  projectSelected: false;
  projectLabel: null;
  capabilities: { projectRead: false; proposals: false; applyNewFile: false; applyExistingFile: false; attachmentImport: boolean };
  availability: 'needs_terms' | 'needs_credential' | 'needs_budget' | 'attention_required' | 'ready' | 'busy' | 'unsupported';
  contributor: { termsRevision: string; accepted: boolean; eligible: boolean };
  historyAvailable: boolean;
  budget: {
    accountId: string | null;
    pending: { requestId: string; period: string; reservationNanoGbp: string; dispatched: boolean }[];
    configured: boolean;
    period: string | null;
    limitNanoGbp: string;
    spentNanoGbp: string;
    reservedNanoGbp: string;
    remainingNanoGbp: string;
    warning: boolean;
    requiresReconciliation: boolean;
    configurationExpiresAt: number | null;
  };
}
export interface StreamEvent {
  requestId: string;
  conversationId: string;
  type: 'content' | 'terminal';
  text?: string;
  status?: TerminalState;
  errorCode?: string;
}
export interface PersonalizationProfile {
  customInstructions: string;
  nickname: string;
  occupation: string;
  moreAboutYou: string;
}
export interface PersonalizationSnapshot { revision: number; profile: PersonalizationProfile }
export type PackageKind = 'skill' | 'plugin';
export interface CustomizationDraft { kind: PackageKind; slug: string; name: string; description: string; instructions: string }
export interface CustomizationEntry {
  id: string;
  kind: PackageKind;
  name: string;
  description: string;
  enabled: boolean;
  readOnly: boolean;
  availability: 'instructions-only' | 'built-in-connector' | 'reference-unavailable';
  entryRevision: number;
}
export interface CustomizationSnapshot { revision: number; entries: CustomizationEntry[] }
export interface CustomizationPackage extends CustomizationDraft { id: string; enabled: boolean; entryRevision: number; readOnly: boolean }
export interface GenerationBinding { requestId: string; conversationId: string }
export interface GenerationFinished { requestId: string; conversationId: string | null; terminalObserved: boolean }
export interface GitHubRepositoryIdentity { installation_id: number; repository_id: number; owner: string; name: string }
export interface GitHubRepository { identity: GitHubRepositoryIdentity; private: boolean; canRead: boolean; canWrite: boolean }
export interface GitHubStatus {
  enabled: boolean;
  state: 'disabled' | 'disconnected' | 'authorising' | 'connected' | 'reconnect_required';
  account: { id: number; login: string } | null;
  repository: GitHubRepositoryIdentity | null;
  userCode: string | null;
  verificationUri: 'https://github.com/login/device';
  nextPollSeconds: number;
  inFlight: boolean;
  mutationsAvailable: false;
}
export interface GitHubFile { path: string; branch: string; blobSha: string; content: string }
export type RendererTracePhase = 'mounted' | 'assets_ready' | 'hydration_started' | 'hydration_finished' | 'visible' | 'hidden';
export type NativeTracePhase = 'host_created' | 'parent_created' | 'page_started' | 'page_finished' | 'parent_visible' | 'visibility_denied' | 'resized' | 'scale_changed' | 'focused' | 'unfocused' | 'destroyed'
  | 'observer_attached' | 'observer_unavailable' | 'observer_detached' | 'navigation_started' | 'content_loading' | 'dom_content_loaded' | 'navigation_completed' | 'navigation_failed' | 'process_failed'
  | 'controller_visible' | 'controller_hidden' | 'controller_unavailable' | 'background_dark' | 'background_other' | 'background_unavailable' | 'zoom_default' | 'zoom_changed' | 'zoom_unavailable'
  | 'startup_gate_created' | 'startup_shell_accepted' | 'startup_reveal_ready' | 'startup_reveal_deadline' | 'startup_show_started'
  | 'startup_show_visible' | 'startup_show_denied' | 'startup_show_visible_cleanup_failed' | 'startup_show_denied_cleanup_failed' | 'startup_dispatch_denied';
export type RenderTraceEvent = { elapsedMs: number; source: 'renderer'; phase: RendererTracePhase } | { elapsedMs: number; source: 'native'; phase: NativeTracePhase };
export interface RenderTraceSnapshot {
  schemaVersion: 1;
  classification: 'bounded-observation-not-a-rendering-cure';
  visibleAtMs: number | null;
  elapsedMs: number;
  dropped: number;
  events: RenderTraceEvent[];
  webview2DetailEventsAvailable: boolean;
}
export interface Bridge {
  shellReady?(input: { protocolVersion: 1 }): Promise<{ accepted: boolean; visible: boolean }>;
  recordRenderEvent?(input: { protocolVersion: 1; phase: RendererTracePhase }): Promise<boolean>;
  getRenderTrace?(): Promise<RenderTraceSnapshot>;
  selectAttachments?(input: { conversationId: string }): Promise<AttachmentSummary[]>;
  removeAttachment?(input: { conversationId: string; attachmentId: string }): Promise<{ removed: boolean }>;
  viewProcessedAttachment?(input: { conversationId: string; requestId: string; attachmentId: string }): Promise<{ viewed: boolean }>;
  windowAction?(input: { action: WindowAction }): Promise<{ maximised: boolean; fullscreen: boolean }>;
  subscribeClose?(callback: () => void): Promise<() => void>;
  getStatus(): Promise<AppStatus>;
  subscribe(callback: (event: StreamEvent) => void): Promise<() => void>;
  configureCredential(): Promise<{ configured: boolean }>;
  removeCredential(): Promise<{ configured: boolean }>;
  verifyCredential?(): Promise<{ state: CredentialValidation }>;
  getPersonalization?(): Promise<PersonalizationSnapshot>;
  savePersonalization?(input: { expectedRevision: number; profile: PersonalizationProfile }): Promise<unknown>;
  clearPersonalization?(input: { expectedRevision: number }): Promise<unknown>;
  getCustomization?(): Promise<CustomizationSnapshot>;
  getCustomizationPackage?(input: { id: string }): Promise<CustomizationPackage>;
  saveCustomization?(input: { expectedRevision: number; package: CustomizationDraft }): Promise<unknown>;
  setCustomizationEnabled?(input: { expectedRevision: number; id: string; enabled: boolean }): Promise<unknown>;
  importCustomization?(input: { expectedRevision: number }): Promise<unknown>;
  generateCustomization?(input: { requestId: string; kind: PackageKind; description: string; dataClass: DataClass }): Promise<{ conversationId: string; requestId: string; accepted: true }>;
  getGitHubStatus?(): Promise<GitHubStatus>;
  beginGitHubConnection?(): Promise<unknown>;
  pollGitHubConnection?(): Promise<unknown>;
  cancelGitHubConnection?(): Promise<unknown>;
  disconnectGitHub?(): Promise<unknown>;
  listGitHubRepositories?(): Promise<GitHubRepository[]>;
  selectGitHubRepository?(input: { installationId: number; repositoryId: number }): Promise<unknown>;
  readGitHubFile?(input: { path: string; branch: string }): Promise<GitHubFile>;
  acceptContributorTerms(): Promise<{ accepted: boolean }>;
  configureBudget(input: BudgetIntent): Promise<{ configured: boolean }>;
  reconcileBudget(input: { requestId: string; confirmedChargeNanoGbp: string }): Promise<{ reconciled: boolean }>;
  listConversations(): Promise<ConversationSummary[]>;
  createConversation(): Promise<Conversation>;
  loadConversation(input: { conversationId: string }): Promise<Conversation>;
  deleteConversation(input: { conversationId: string }): Promise<{ deleted: boolean }>;
  sendMessage(input: SendMessageIntent): Promise<{ requestId: string; accepted: boolean }>;
  cancelMessage(requestId: string): Promise<{ cancelled: boolean }>;
}
