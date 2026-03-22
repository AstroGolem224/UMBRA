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

export type NoteCategory = "prompts" | "cli" | "agents" | "skills" | "misc";

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

export interface AppConfig {
  theme: string;
  vaultPath: string;
  notesSubdir: string;
  repoRootPath: string;
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
  customAgents?: CustomAgentConfig[];
}
