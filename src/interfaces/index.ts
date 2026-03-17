export type AgentStatus = "online" | "idle" | "offline" | "error";

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

export interface Task {
  id: string;
  slug: string;
  title: string;
  status: "todo" | "in-progress" | "blocked" | "done" | "cancelled";
  priority: "critical" | "high" | "medium" | "low";
  project: string;
  agent: string;
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

export interface AppConfig {
  theme: string;
  vaultPath: string;
  notesSubdir: string;
  launchTargets?: LaunchTarget[];
  githubTargets?: GithubOpenTarget[];
  pmToolUrl: string;
  pmToolPollSeconds: number;
  githubPat?: string;
  agentNotes?: Record<string, AgentNote>;
}
