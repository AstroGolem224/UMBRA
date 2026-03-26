export type AgentStatus = "online" | "working" | "idle" | "offline" | "error";

export interface Agent {
  id: string;
  name: string;
  role: string;
  status: AgentStatus;
  activeTaskId?: string;
  lastSeen: string;
  allowedTools: string[];
  skills: string[];
}

export interface AgentTask {
  id: string;
  title: string;
  description?: string;
  priority: "critical" | "high" | "medium" | "low";
}

export type NoteCategory = string;

export interface Note {
  id: string;
  title: string;
  content: string;
  category: NoteCategory;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  filePath: string;
}

export interface NoteInput {
  title: string;
  content: string;
  category: NoteCategory;
  tags: string[];
}

export interface CronJob {
  id: string;
  name: string;
  schedule: string;
  command: string;
  enabled: boolean;
  lastRun?: string;
  nextRun?: string;
  lastStatus: "ok" | "error" | "pending";
  lastOutput?: string;
}

export interface AgentCronJob {
  id: string;
  agentId: string;
  agentName: string;
  job: string;
  timing: string;
  recurrence: string;
  timezone?: string | null;
  enabled: boolean;
  lastRun?: string | null;
  nextRun?: string | null;
  lastStatus: string;
  notes?: string | null;
  source?: string | null;
  command?: string | null;
  updatedAt: string;
}

export interface WorkspacePreset {
  id: string;
  name: string;
  rootPath: string;
  writable: boolean;
  allowedProviders: string[];
  allowedAgents: string[];
}

export interface WorkspaceBootstrapResult {
  providerId: string;
  workspaceId: string;
  files: string[];
  summary: string;
}

export interface ProviderCommandConfig {
  providerId: string;
  command: string;
}

export interface PersonaPreset {
  id: string;
  name: string;
  description: string;
  prompt: string;
}

export type DispatchMode = "chat" | "task";
export type DispatchStatus = "draft" | "queued" | "working" | "done" | "error" | "cancelled";
export type RunOutcomeStatus = "succeeded" | "blocked" | "needs_input";

export interface DispatchRun {
  id: string;
  parentRunId?: string | null;
  channelId?: string | null;
  sourceMessageId?: string | null;
  jobId?: string | null;
  sessionId?: string | null;
  mode: DispatchMode;
  agentId: string;
  providerId: string;
  workspaceId: string;
  pmTaskId?: string | null;
  prompt: string;
  personaId?: string | null;
  outcomeStatus?: RunOutcomeStatus | null;
  status: DispatchStatus;
  createdAt: string;
  updatedAt: string;
}

export interface RunEvent {
  id: string;
  runId: string;
  type: "user_message" | "system" | "stdout" | "stderr" | "agent_message";
  body: string;
  createdAt: string;
}

export interface RunEventPage {
  items: RunEvent[];
  nextBefore?: string | null;
  hasMore: boolean;
}

export interface RunArtifact {
  id: string;
  runId: string;
  kind: "summary" | "file" | "test";
  label: string;
  value: string;
  createdAt: string;
}

export type ChannelMessageKind = "user" | "agent" | "system";

export interface OpsChannel {
  id: string;
  name: string;
  description: string;
  workspaceId: string;
  defaultAgentId?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface OpsChannelMessage {
  id: string;
  channelId: string;
  parentMessageId?: string | null;
  runId?: string | null;
  jobId?: string | null;
  sessionId?: string | null;
  agentId?: string | null;
  authorLabel?: string | null;
  kind: ChannelMessageKind;
  body: string;
  createdAt: string;
}

export interface OpsChannelMessagePage {
  items: OpsChannelMessage[];
  nextBefore?: string | null;
  hasMore: boolean;
}

export type OpsJobStatus = "open" | "running" | "blocked" | "done";

export interface OpsJob {
  id: string;
  channelId: string;
  sourceMessageId: string;
  title: string;
  summary: string;
  agentId: string;
  workspaceId: string;
  pmTaskId?: string | null;
  runId?: string | null;
  status: OpsJobStatus;
  createdAt: string;
  updatedAt: string;
}

export type RouteApprovalStatus = "pending" | "approved" | "rejected";

export interface OpsRouteApproval {
  id: string;
  channelId: string;
  messageId: string;
  agentId: string;
  workspaceId: string;
  reason: string;
  status: RouteApprovalStatus;
  createdAt: string;
  updatedAt: string;
}

export interface OpsRule {
  id: string;
  name: string;
  pattern: string;
  targetAgentId?: string | null;
  workspaceId?: string | null;
  enabled: boolean;
  requiresHumanGate: boolean;
  lastTriggeredAt?: string | null;
  createdAt: string;
  updatedAt: string;
}

export type OpsSessionState = "active" | "paused" | "completed";

export interface OpsSessionTemplate {
  id: string;
  name: string;
  workspaceId: string;
  agentIds: string[];
  autoAdvance: boolean;
  requiresHumanGate: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface OpsSession {
  id: string;
  channelId: string;
  templateId: string;
  state: OpsSessionState;
  currentTurnIndex: number;
  awaitingHumanGate: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface Task {
  id: string;
  title: string;
  status: "todo" | "in-progress" | "blocked" | "done" | "cancelled";
  priority: "urgent" | "critical" | "high" | "medium" | "low";
  project: string;
  projectId: string;
  columnId: string;
  columnKind?: "backlog" | "in_progress" | "review" | "done";
  description?: string;
  labels?: string[];
  position?: number;
  deadline?: string | null;
  nextDueDate?: string | null;
  createdAt?: string;
  updatedAt?: string;
  comments?: TaskComment[];
}

export interface TaskComment {
  id: string;
  taskId: string;
  content: string;
  createdAt: string;
  updatedAt: string;
}

export interface PmProject {
  id: string;
  name: string;
}

export interface PmColumn {
  id: string;
  name: string;
  kind: "backlog" | "in_progress" | "review" | "done";
  projectId: string;
}

export interface LaunchTarget {
  id: string;
  name: string;
  path: string;
  icon?: string;
}

export interface GithubOpenTarget {
  id: string;
  name: string;
  owner: string;
  repo: string;
}

export interface RepoInfo {
  id: string;
  name: string;
  owner: string;
  repo: string;
  fullName: string;
  openIssues: number;
  pushedAt?: string;
  htmlUrl: string;
}

export interface SkillInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  category: string;
  agents: string[];
  content: string;
  folder: string;
}

export type NoteQuickLinkKind = "task" | "agent" | "repo" | "launcher";

export interface NoteQuickLinkOption {
  id: string;
  kind: NoteQuickLinkKind;
  label: string;
  description?: string;
  url?: string;
}

export interface NoteQuickLinkGroup {
  id: string;
  label: string;
  options: NoteQuickLinkOption[];
}

export interface SavedNoteAttachment {
  fileName: string;
  absolutePath: string;
  relativePath: string;
  markdown: string;
  isImage: boolean;
}

export interface AgentNote {
  notes: string;
  link: string;
}

export interface CustomAgentConfig {
  id: string;
  name: string;
  role: string;
  skills: string[];
  allowedTools: string[];
}

export interface UpdateCheckResult {
  configured: boolean;
  updateAvailable: boolean;
  currentVersion: string;
  version?: string | null;
}

export interface ProviderProbeResult {
  providerId: string;
  command: string;
  launchable: boolean;
  exitCode?: number | null;
  summary: string;
}

export interface ProviderAuthState {
  providerId: string;
  agentIds: string[];
  provisionedCount: number;
  summary: string;
}

export interface ProviderSetupChecklistItem {
  key: string;
  label: string;
  ready: boolean;
  detail: string;
}

export interface ProviderSetupChecklistResult {
  providerId: string;
  workspaceId: string;
  items: ProviderSetupChecklistItem[];
  summary: string;
}

export interface AppConfig {
  theme: string;
  closeToTray: boolean;
  vaultPath: string;
  notesSubdir: string;
  repoRootPath: string;
  workspacePresets: WorkspacePreset[];
  workspaceGrantRoots: string[];
  defaultWorkspaceId?: string | null;
  personaPresets: PersonaPreset[];
  providerCommands: ProviderCommandConfig[];
  launchTargets?: LaunchTarget[];
  githubTargets?: GithubOpenTarget[];
  pmToolUrl: string;
  pmToolDashboardUrl: string;
  pmToolPollSeconds: number;
  updaterEndpoint: string;
  updaterPublicKey: string;
  autoCheckForUpdates: boolean;
  uapAdvertiseHost: string;
  uapPort: number;
  uapToken: string;
  githubPat?: string;
  taskLanePrefs?: Partial<Record<PmColumn["kind"], boolean>>;
  agentNotes?: Record<string, AgentNote>;
  agentAuthTokens?: Record<string, string>;
  customAgents?: CustomAgentConfig[];
}
