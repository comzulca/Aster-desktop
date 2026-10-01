export const DEFAULT_MODEL = 'nvidia/nemotron-3.5-lightning-30b-a3b' as const;
export const MUSE_MODEL = 'meta/muse-glimmer-30b' as const;
export type ModelId = typeof DEFAULT_MODEL | typeof MUSE_MODEL;

export type SendMessageIntent = {
  conversationId: string;
  requestId: string;
  modelId: ModelId;
  message: string;
};

export type FileReceipt = {
  relativePath: string;
  requestId: string;
  sha256: string;
  byteCount: number;
};

export interface AppStatus {
  policyVersion: number;
  configured: boolean;
  defaultModel: ModelId;
  models: { id: ModelId; label: string }[];
  activeRequest: boolean;
  projectSelected: boolean;
  projectLabel: string | null;
  capabilities: {
    projectRead: boolean;
    proposals: boolean;
    applyNewFile: boolean;
    applyExistingFile: boolean;
  };
}

export type StreamEvent = {
  requestId: string;
  conversationId: string;
  type: 'content' | 'terminal';
  text?: string;
  status?: 'completed' | 'failed' | 'cancelled';
  errorCode?: string;
};

export interface Bridge {
  getStatus(): Promise<AppStatus>;
  subscribe(callback: (event: StreamEvent) => void): Promise<() => void>;
  configureCredential(): Promise<{ configured: boolean }>;
  removeCredential(): Promise<{ configured: boolean }>;
  selectProject(): Promise<{ selected: boolean; label: string | null }>;
  selectProjectFile(args: { conversationId: string; requestId: string }): Promise<FileReceipt | null>;
  sendMessage(args: SendMessageIntent): Promise<{ requestId: string; accepted: boolean }>;
  sendProjectMessage(args: SendMessageIntent & { relativePath: string; expectedSha256: string }): Promise<{ requestId: string; accepted: boolean }>;
  reviewProjectProposal(args: { conversationId: string; requestId: string; proposalJson: string }): Promise<{
    status: 'verified' | 'attention_required' | 'denied' | 'proposal_only';
    operation: 'create_new' | 'replace_existing';
    relativePath: string;
    newSha256: string;
  }>;
  cancelMessage(requestId: string): Promise<{ cancelled: boolean }>;
}
