// Browser-safe mock shims for Tauri APIs used by preview and browser e2e runs.

type DispatchRun = {
  id: string;
  parentRunId?: string | null;
  channelId?: string | null;
  sourceMessageId?: string | null;
  jobId?: string | null;
  sessionId?: string | null;
  mode: "chat" | "task";
  agentId: string;
  providerId: string;
  workspaceId: string;
  pmTaskId?: string | null;
  prompt: string;
  personaId?: string | null;
  outcomeStatus?: "succeeded" | "blocked" | "needs_input" | null;
  status: "draft" | "queued" | "working" | "done" | "error" | "cancelled";
  createdAt: string;
  updatedAt: string;
};

type RunEvent = {
  id: string;
  runId: string;
  type: "user_message" | "system" | "stdout" | "stderr" | "agent_message";
  body: string;
  createdAt: string;
};

type RunArtifact = {
  id: string;
  runId: string;
  kind: "summary" | "file" | "test";
  label: string;
  value: string;
  createdAt: string;
};

type OpsMessage = {
  id: string;
  channelId: string;
  parentMessageId?: string | null;
  runId?: string | null;
  jobId?: string | null;
  sessionId?: string | null;
  agentId?: string | null;
  authorLabel?: string | null;
  kind: "user" | "agent" | "system";
  body: string;
  createdAt: string;
};

type MockState = {
  runs: DispatchRun[];
  eventsByRun: Record<string, RunEvent[]>;
  artifactsByRun: Record<string, RunArtifact[]>;
  channels: Array<{
    id: string;
    name: string;
    description: string;
    workspaceId: string;
    defaultAgentId?: string | null;
    createdAt: string;
    updatedAt: string;
  }>;
  messagesByChannel: Record<string, OpsMessage[]>;
  jobsByChannel: Record<string, unknown[]>;
  approvalsByChannel: Record<string, unknown[]>;
  sessionsByChannel: Record<string, unknown[]>;
};

const now = "2026-03-26T08:00:00.000Z";

function buildInitialState(): MockState {
  const runs: DispatchRun[] = [
    {
      id: "run-1",
      parentRunId: null,
      mode: "task",
      agentId: "forge",
      providerId: "codex",
      workspaceId: "umbra",
      pmTaskId: "pm-1",
      prompt: "implement the workbench retry flow",
      personaId: "implementer",
      outcomeStatus: "needs_input",
      status: "error",
      createdAt: "2026-03-26T07:40:00.000Z",
      updatedAt: "2026-03-26T07:44:00.000Z",
    },
    {
      id: "run-0",
      parentRunId: null,
      mode: "chat",
      agentId: "prism",
      providerId: "claude",
      workspaceId: "umbra",
      pmTaskId: null,
      prompt: "review the ops room rollout",
      personaId: "reviewer",
      outcomeStatus: "succeeded",
      status: "done",
      createdAt: "2026-03-26T07:00:00.000Z",
      updatedAt: "2026-03-26T07:05:00.000Z",
    },
  ];

  return {
    runs,
    eventsByRun: {
      "run-1": [
        {
          id: "run-1-e1",
          runId: "run-1",
          type: "user_message",
          body: "implement the workbench retry flow",
          createdAt: "2026-03-26T07:40:00.000Z",
        },
        {
          id: "run-1-e2",
          runId: "run-1",
          type: "system",
          body: "launch attempt via codex exec",
          createdAt: "2026-03-26T07:40:01.000Z",
        },
        {
          id: "run-1-e3",
          runId: "run-1",
          type: "agent_message",
          body: "i can patch the runner, but i need the recovery path clarified.",
          createdAt: "2026-03-26T07:40:03.000Z",
        },
        {
          id: "run-1-e4",
          runId: "run-1",
          type: "system",
          body: "run recovered after app restart; previous provider process is no longer attached",
          createdAt: "2026-03-26T07:44:00.000Z",
        },
      ],
      "run-0": [
        {
          id: "run-0-e1",
          runId: "run-0",
          type: "user_message",
          body: "review the ops room rollout",
          createdAt: "2026-03-26T07:00:00.000Z",
        },
        {
          id: "run-0-e2",
          runId: "run-0",
          type: "agent_message",
          body: "the rollout is coherent. phase 6 still needed recovery and pagination.",
          createdAt: "2026-03-26T07:02:00.000Z",
        },
      ],
    },
    artifactsByRun: {
      "run-1": [
        {
          id: "run-1-a1",
          runId: "run-1",
          kind: "summary",
          label: "agent summary",
          value: "retry this run after wiring the recovery command",
          createdAt: "2026-03-26T07:44:00.000Z",
        },
      ],
      "run-0": [],
    },
    channels: [
      {
        id: "channel-1",
        name: "delivery",
        description: "",
        workspaceId: "umbra",
        defaultAgentId: "forge",
        createdAt: now,
        updatedAt: now,
      },
    ],
    messagesByChannel: {
      "channel-1": [
        {
          id: "msg-1",
          channelId: "channel-1",
          kind: "user",
          body: "@forge please finish phase 6 and report back",
          authorLabel: "you",
          createdAt: "2026-03-26T07:41:00.000Z",
        },
        {
          id: "msg-2",
          channelId: "channel-1",
          kind: "system",
          body: "dispatched to @forge",
          authorLabel: "umbra",
          runId: "run-1",
          createdAt: "2026-03-26T07:41:02.000Z",
        },
        {
          id: "msg-3",
          channelId: "channel-1",
          kind: "agent",
          body: "the retry surface is in place; recovery now writes audit events.",
          authorLabel: "forge",
          agentId: "forge",
          runId: "run-1",
          createdAt: "2026-03-26T07:44:05.000Z",
        },
      ],
    },
    jobsByChannel: {
      "channel-1": [
        {
          id: "job-1",
          channelId: "channel-1",
          sourceMessageId: "msg-1",
          title: "finish phase 6",
          summary: "close recovery, pagination and retry",
          agentId: "forge",
          workspaceId: "umbra",
          pmTaskId: "pm-1",
          runId: "run-1",
          status: "blocked",
          createdAt: now,
          updatedAt: now,
        },
      ],
    },
    approvalsByChannel: { "channel-1": [] },
    sessionsByChannel: { "channel-1": [] },
  };
}

let state = buildInitialState();
let idCounter = 20;

function nextId(prefix: string) {
  idCounter += 1;
  return `${prefix}-${idCounter}`;
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function browserPageItems<T extends { createdAt: string }>(items: T[], before?: string | null) {
  const sorted = [...items].sort((left, right) => left.createdAt.localeCompare(right.createdAt));
  if (!before) {
    const slice = sorted.slice(-2);
    return {
      items: clone(slice),
      nextBefore: sorted.length > slice.length ? slice[0]?.createdAt ?? null : null,
      hasMore: sorted.length > slice.length,
    };
  }
  const older = sorted.filter((item) => item.createdAt < before);
  return {
    items: clone(older),
    nextBefore: null,
    hasMore: false,
  };
}

const MOCK_DEFAULTS: Record<string, unknown> = {
  get_agents: [
    {
      id: "forge",
      name: "Forge",
      role: "builder",
      status: "working",
      skills: ["rust", "vue"],
      allowedTools: ["shell", "git"],
      lastSeen: now,
    },
    {
      id: "prism",
      name: "Prism",
      role: "reviewer",
      status: "idle",
      skills: ["review", "testing"],
      allowedTools: ["shell", "git"],
      lastSeen: now,
    },
  ],
  get_pm_tasks: [
    {
      id: "pm-1",
      title: "finish phase 6 hardening",
      status: "in-progress",
      priority: "high",
      project: "UMBRA",
      projectId: "umbra-project",
      columnId: "review",
    },
  ],
  get_config: {
    theme: "ember",
    closeToTray: true,
    vaultPath: "D:/Obsidian/2nd-brain/2nd-brain",
    notesSubdir: "UMBRA_Notes",
    repoRootPath: "C:/Users/matth/OneDrive/Dokumente/GitHub",
    workspacePresets: [
      {
        id: "umbra",
        name: "UMBRA",
        rootPath: "C:/Users/matth/OneDrive/Dokumente/GitHub/UMBRA",
        writable: true,
        allowedProviders: [],
        allowedAgents: [],
      },
    ],
    workspaceGrantRoots: ["C:/Users/matth/OneDrive/Dokumente/GitHub/UMBRA"],
    defaultWorkspaceId: "umbra",
    personaPresets: [
      {
        id: "implementer",
        name: "implementer",
        description: "ship code",
        prompt: "implement the requested change",
      },
      {
        id: "reviewer",
        name: "reviewer",
        description: "review code",
        prompt: "review the requested change",
      },
    ],
    providerCommands: [
      { providerId: "codex", command: "codex" },
      { providerId: "claude", command: "claude" },
      { providerId: "gemini", command: "gemini" },
      { providerId: "kimi", command: "kimi" },
    ],
    launchTargets: [],
    githubTargets: [],
    pmToolUrl: "",
    pmToolDashboardUrl: "",
    pmToolPollSeconds: 30,
    updaterEndpoint: "",
    updaterPublicKey: "",
    autoCheckForUpdates: false,
    uapAdvertiseHost: "127.0.0.1",
    uapPort: 8765,
    uapToken: "mock-uap-token",
    taskLanePrefs: {},
    agentAuthTokens: { forge: "forge-token", prism: "prism-token" },
  },
  save_config: null,
  delete_note: null,
  launch_target: null,
  open_github: null,
  list_notes: [],
  list_cron_jobs: [],
  list_agent_cron_jobs: [],
  create_cron_job: null,
  toggle_cron_job: true,
  delete_cron_job: null,
  run_cron_job_now: "",
  get_github_repos: [],
  get_obsidian_stats: { connected: false, totalNotes: 0, recentNotes: [], vaultPath: "" },
  get_tmlite_tasks: [],
  list_skills: [],
  push_agent_task: null,
  add_agent: null,
  remove_agent: null,
  create_tmlite_task: { id: "task-001", title: "mock task", status: "todo", project: "", priority: "medium" },
  get_pm_projects: [],
  get_pm_columns: [],
  create_pm_task: null,
  move_pm_task: null,
  reorder_pm_tasks: null,
  update_pm_task: null,
  add_pm_comment: null,
  list_user_repos: [],
  open_github_url: null,
  reveal_run_path: null,
  open_workspace_folder: null,
  open_workspace_terminal: null,
};

// ---------- PM Tool HTTP proxy (browser mode) ----------

// In browser mode, Vite proxies /pm-api → PM Tool to avoid CORS

function kindToStatus(kind: string): string {
  if (kind === "in_progress" || kind === "review") return "in-progress";
  if (kind === "done") return "done";
  return "todo";
}

async function pmFetch(path: string, init?: RequestInit): Promise<unknown> {
  const url = `/pm-api${path.replace(/^\/api/, "")}`;
  const headers: Record<string, string> = {};
  if (init?.method && init.method !== "GET") headers["Content-Type"] = "application/json";
  const resp = await fetch(url, { ...init, headers: { ...headers, ...(init?.headers as Record<string, string> ?? {}) } });
  if (!resp.ok) throw new Error(`PM Tool ${resp.status}: ${await resp.text()}`);
  const text = await resp.text();
  return text ? JSON.parse(text) : null;
}

type PmProject = { id: string; name: string; columns: Array<{ id: string; kind: string }> };

async function pmGetTasks(): Promise<unknown> {
  const projects = (await pmFetch("/api/projects")) as PmProject[];
  const colKind: Record<string, string> = {};
  const projName: Record<string, string> = {};
  for (const p of projects) {
    projName[p.id] = p.name;
    for (const c of p.columns ?? []) colKind[c.id] = c.kind;
  }
  const allTasks: unknown[] = [];
  for (const p of projects) {
    const tasks = (await pmFetch(`/api/projects/${p.id}/tasks`)) as Array<Record<string, unknown>>;
    allTasks.push(...tasks);
  }
  return (allTasks as Array<Record<string, unknown>>).map((t) => {
    const colId = String(t.column_id ?? "");
    const projId = String(t.project_id ?? "");
    const kind = colKind[colId] ?? "backlog";
    return {
      ...t,
      status: kindToStatus(kind),
      project: projName[projId] ?? "",
      projectId: projId,
      columnId: colId,
      columnKind: kind,
      nextDueDate: t.next_due_date,
      createdAt: t.created_at,
      updatedAt: t.updated_at,
      comments: ((t.comments as Array<Record<string, unknown>> | undefined) ?? []).map((c) => ({
        id: c.id,
        taskId: c.task_id,
        content: c.content,
        createdAt: c.created_at,
        updatedAt: c.updated_at,
      })),
    };
  });
}

// ---------- End PM Tool proxy ----------

export const invoke = async (cmd: string, args?: unknown): Promise<unknown> => {
  console.debug(`[tauri-mock] invoke(${cmd})`);
  const payload = (args ?? {}) as Record<string, unknown>;

  // PM Tool commands — proxy to real API in browser mode
  switch (cmd) {
    case "get_pm_tasks":
      try { return await pmGetTasks(); } catch (e) { console.warn("[tauri-mock] PM Tool unreachable, using fallback", e); break; }
    case "get_pm_projects":
      try { return await pmFetch("/api/projects"); } catch { break; }
    case "get_pm_columns":
      try {
        const proj = (await pmFetch(`/api/projects/${String(payload.projectId)}`)) as PmProject;
        return proj.columns ?? [];
      } catch { break; }
    case "create_pm_task":
      try {
        return await pmFetch("/api/tasks", {
          method: "POST",
          body: JSON.stringify(payload),
        });
      } catch { break; }
    case "move_pm_task":
      try {
        return await pmFetch(`/api/tasks/${String(payload.taskId)}/move`, {
          method: "PATCH",
          body: JSON.stringify({ column_id: payload.columnId, position: payload.position ?? 0 }),
        });
      } catch { break; }
    case "update_pm_task":
      try {
        return await pmFetch(`/api/tasks/${String(payload.taskId)}`, {
          method: "PATCH",
          body: JSON.stringify(payload),
        });
      } catch { break; }
    case "add_pm_comment":
      try {
        return await pmFetch(`/api/tasks/${String(payload.taskId)}/comments`, {
          method: "POST",
          body: JSON.stringify({ content: payload.content }),
        });
      } catch { break; }
    case "reorder_pm_tasks":
      try {
        return await pmFetch(`/api/tasks/reorder`, {
          method: "POST",
          body: JSON.stringify(payload),
        });
      } catch { break; }
    case "list_notes":
      try {
        const resp = await fetch("/notes-api/list");
        if (resp.ok) return await resp.json();
      } catch { /* fall through to mock default */ }
      break;
    case "list_skills":
      try {
        const resp = await fetch("/skills-api/list");
        if (resp.ok) return await resp.json();
      } catch { /* fall through to mock default */ }
      break;
  }

  switch (cmd) {
    case "list_dispatch_runs":
      return clone(state.runs);
    case "list_channel_dispatch_runs":
      return clone(state.runs.filter((run) => run.channelId === String(payload.channelId)));
    case "list_run_events_page":
      return browserPageItems(
        state.eventsByRun[String(payload.runId)] ?? [],
        payload.before as string | null,
      );
    case "list_run_events":
      return clone(state.eventsByRun[String(payload.runId)] ?? []);
    case "list_run_artifacts":
      return clone(state.artifactsByRun[String(payload.runId)] ?? []);
    case "retry_dispatch_run": {
      const sourceId = String(payload.runId);
      const source = state.runs.find((run) => run.id === sourceId);
      if (!source) return null;
      const retryId = nextId("run");
      const createdAt = "2026-03-26T08:15:00.000Z";
      const retryRun: DispatchRun = {
        ...source,
        id: retryId,
        parentRunId: source.id,
        status: "queued",
        outcomeStatus: null,
        createdAt,
        updatedAt: createdAt,
      };
      state.runs = [retryRun, ...state.runs];
      state.eventsByRun[retryId] = [
        {
          id: nextId("event"),
          runId: retryId,
          type: "user_message",
          body: retryRun.prompt,
          createdAt,
        },
        {
          id: nextId("event"),
          runId: retryId,
          type: "system",
          body: `continued from run ${source.id}`,
          createdAt: "2026-03-26T08:15:01.000Z",
        },
      ];
      state.artifactsByRun[retryId] = [];
      state.eventsByRun[source.id] = [
        ...(state.eventsByRun[source.id] ?? []),
        {
          id: nextId("event"),
          runId: source.id,
          type: "system",
          body: `retry dispatched as run ${retryId}`,
          createdAt: "2026-03-26T08:15:02.000Z",
        },
      ];
      return clone(retryRun);
    }
    case "create_dispatch_run":
      return clone(state.runs[0]);
    case "get_dispatch_run":
      return clone(state.runs.find((run) => run.id === String(payload.runId)) ?? null);
    case "cancel_dispatch_run":
      return clone(state.runs[0]);
    case "list_ops_channels":
      return clone(state.channels);
    case "check_provider_setup":
      return {
        providerId: String(payload.providerId ?? "codex"),
        workspaceId: String(payload.workspaceId ?? "umbra"),
        summary: "4/4 checklist item(s) ready",
        items: [
          { key: "command", label: "provider command configured", ready: true, detail: "codex configured" },
          { key: "auth", label: "per-agent auth provisioned", ready: true, detail: "1/1 token ready" },
          { key: "workspace", label: "workspace inside grant roots", ready: true, detail: "UMBRA is allowed" },
          { key: "instructions", label: "provider instruction file present", ready: true, detail: "found AGENTS.md" },
        ],
      };
    case "get_provider_auth_state":
      return {
        providerId: String(payload.providerId ?? "codex"),
        agentIds: ["forge"],
        provisionedCount: 1,
        summary: "1/1 agent token(s) provisioned for codex: forge",
      };
    case "smoke_test_provider_command":
      return {
        providerId: String(payload.providerId ?? "codex"),
        command: "codex",
        launchable: true,
        exitCode: 0,
        summary: "codex --version completed successfully",
      };
    case "bootstrap_provider_workspace":
      return {
        providerId: String(payload.providerId ?? "codex"),
        workspaceId: String(payload.workspaceId ?? "umbra"),
        files: ["C:/Users/matth/OneDrive/Dokumente/GitHub/UMBRA/AGENTS.md", "C:/Users/matth/OneDrive/Dokumente/GitHub/UMBRA/.umbra/worker.env.example"],
        summary: "wrote 2 bootstrap file(s)",
      };
    case "list_ops_channel_messages_page":
      return browserPageItems(
        state.messagesByChannel[String(payload.channelId)] ?? [],
        payload.before as string | null,
      );
    case "list_ops_channel_messages":
      return clone(state.messagesByChannel[String(payload.channelId)] ?? []);
    case "list_ops_jobs":
      return clone(state.jobsByChannel[String(payload.channelId)] ?? []);
    case "list_ops_route_approvals":
      return clone(state.approvalsByChannel[String(payload.channelId)] ?? []);
    case "list_ops_sessions":
      return clone(state.sessionsByChannel[String(payload.channelId)] ?? []);
    case "list_ops_rules":
      return [];
    case "list_ops_session_templates":
      return [];
    case "send_ops_channel_message": {
      const input = (payload.input ?? {}) as Record<string, unknown>;
      const channelId = String(input.channelId);
      const message: OpsMessage = {
        id: nextId("msg"),
        channelId,
        kind: "user",
        body: String(input.body ?? ""),
        authorLabel: "you",
        agentId: typeof input.agentId === "string" ? input.agentId : null,
        parentMessageId: typeof input.parentMessageId === "string" ? input.parentMessageId : null,
        createdAt: "2026-03-26T08:20:00.000Z",
      };
      state.messagesByChannel[channelId] = [...(state.messagesByChannel[channelId] ?? []), message];
      return clone(message);
    }
    case "save_config": {
      const nextConfig = clone((payload.config ?? payload) as Record<string, unknown>);
      MOCK_DEFAULTS.get_config = nextConfig;
      return null;
    }
    case "save_note": {
      const note = (args as { note?: Record<string, unknown> } | undefined)?.note ?? {};
      const savedAt = new Date().toISOString();
      return {
        ...note,
        createdAt: (note.createdAt as string | undefined) ?? savedAt,
        updatedAt: savedAt,
        filePath: (note.filePath as string | undefined) || "mock://UMBRA_Notes/misc/note.md",
      };
    }
    case "__reset_tauri_mock__":
      state = buildInitialState();
      idCounter = 20;
      return null;
    default:
      return cmd in MOCK_DEFAULTS ? clone(MOCK_DEFAULTS[cmd]) : null;
  }
};

export const listen = async (_event: string, _handler: unknown): Promise<() => void> => {
  return () => {};
};

export const emit = async (_event: string, _payload?: unknown): Promise<void> => {};

export const getCurrentWindow = () => ({
  minimize: async () => {},
  maximize: async () => {},
  unmaximize: async () => {},
  close: async () => {},
  isMaximized: async () => false,
});

export class Channel<T = unknown> {
  id = 0;
  onmessage: (response: T) => void = () => {};
}
