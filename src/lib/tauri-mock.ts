// Browser-safe no-op shims for Tauri APIs (used outside Tauri WebView)

const MOCK_DEFAULTS: Record<string, unknown> = {
  get_agents: [],
  list_notes: [],
  get_pm_tasks: [],
  get_config: {},
  save_config: null,
  save_note: null,
  delete_note: null,
  launch_target: null,
  open_github: null,
  list_cron_jobs: [],
  create_cron_job: null,
  toggle_cron_job: true,
  delete_cron_job: null,
  run_cron_job_now: "",
};

export const invoke = async (_cmd: string, _args?: unknown): Promise<unknown> => {
  console.debug(`[tauri-mock] invoke(${_cmd}) — not in Tauri`);
  return _cmd in MOCK_DEFAULTS ? MOCK_DEFAULTS[_cmd] : null;
};

export const listen = async (
  _event: string,
  _handler: unknown
): Promise<() => void> => {
  return () => {};
};

export const emit = async (_event: string, _payload?: unknown): Promise<void> => {};
