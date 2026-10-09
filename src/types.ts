export const DEFAULT_MODEL = 'muse-spark-1.3-contributor' as const;
export type ModelId = typeof DEFAULT_MODEL;
export type DataClass = 'synthetic' | 'public_eligible';
export type ReasoningEffort = 'minimal' | 'low' | 'medium' | 'high' | 'xhigh';
export type WindowAction = 'minimise' | 'toggle_maximise' | 'toggle_fullscreen' | 'start_dragging' | 'close';
export type TerminalState = 'completed' | 'failed' | 'cancelled' | 'truncated' | 'refused';
export type EntryState = TerminalState | 'interrupted';
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
export interface Bridge {
  shellReady?(input: { protocolVersion: 1 }): Promise<{ accepted: boolean; visible: boolean }>;
  selectAttachments?(input: { conversationId: string }): Promise<AttachmentSummary[]>;
  removeAttachment?(input: { conversationId: string; attachmentId: string }): Promise<{ removed: boolean }>;
  viewProcessedAttachment?(input: { conversationId: string; requestId: string; attachmentId: string }): Promise<{ viewed: boolean }>;
  windowAction?(input: { action: WindowAction }): Promise<{ maximised: boolean; fullscreen: boolean }>;
  subscribeClose?(callback: () => void): Promise<() => void>;
  getStatus(): Promise<AppStatus>;
  subscribe(callback: (event: StreamEvent) => void): Promise<() => void>;
  configureCredential(): Promise<{ configured: boolean }>;
  removeCredential(): Promise<{ configured: boolean }>;
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
