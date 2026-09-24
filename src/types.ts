export const DEFAULT_MODEL = 'nvidia/nemotron-3.5-lightning-30b-a3b' as const;
export const MUSE_MODEL = 'meta/muse-glimmer-30b' as const;
export type ModelId = typeof DEFAULT_MODEL | typeof MUSE_MODEL;

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
  sendMessage(args: {
    conversationId: string;
    requestId: string;
    modelId: ModelId;
    message: string;
  }): Promise<{ requestId: string; accepted: boolean }>;
  cancelMessage(requestId: string): Promise<{ cancelled: boolean }>;
}
