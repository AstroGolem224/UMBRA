// Browser-safe no-op shims for Tauri APIs (used outside Tauri WebView)

const MOCK_DEFAULTS: Record<string, unknown> = {
  get_agents: [],
  list_notes: [],
  get_pm_tasks: [],
  get_config: {},
  save_config: null,
  delete_note: null,
  launch_target: null,
  open_github: null,
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
  add_agent: { id: "mock", name: "Mock", role: "", status: "offline", skills: [], allowedTools: [], lastSeen: new Date().toISOString() },
  remove_agent: null,
  get_pm_projects: [],
  get_pm_columns: [],
  create_pm_task: null,
  move_pm_task: null,
  reorder_pm_tasks: null,
  update_pm_task: null,
  add_pm_comment: null,
  list_user_repos: [],
  open_github_url: null,
};

export const invoke = async (cmd: string, args?: unknown): Promise<unknown> => {
  console.debug(`[tauri-mock] invoke(${cmd}) - not in Tauri`);

  if (cmd === "save_note") {
    const note = (args as { note?: Record<string, unknown> } | undefined)?.note ?? {};
    const now = new Date().toISOString();
    return {
      ...note,
      createdAt: (note.createdAt as string | undefined) ?? now,
      updatedAt: now,
      filePath: (note.filePath as string | undefined) || "mock://UMBRA_Notes/misc/note.md",
    };
  }

  return cmd in MOCK_DEFAULTS ? MOCK_DEFAULTS[cmd] : null;
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
