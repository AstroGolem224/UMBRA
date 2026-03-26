import { defineStore } from "pinia";
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "@/interfaces";

const defaults: AppConfig = {
  theme: "ember",
  closeToTray: true,
  vaultPath: "",
  notesSubdir: "UMBRA_Notes",
  repoRootPath: "",
  workspacePresets: [],
  workspaceGrantRoots: [],
  defaultWorkspaceId: null,
  personaPresets: [
    {
      id: "implementer",
      name: "implementer",
      description: "prefer shipping code, tests, and a concise result summary.",
      prompt: "you are the implementation persona. prefer concrete code changes over discussion. finish with changed files, checks, and blockers.",
    },
    {
      id: "reviewer",
      name: "reviewer",
      description: "prioritize bugs, regressions, missing tests, and trust boundaries.",
      prompt: "you are the review persona. prioritize findings, regressions, missing tests, trust boundaries, and operational risk over summaries.",
    },
    {
      id: "architect",
      name: "architect",
      description: "plan with clear tradeoffs, rollout phases, and failure modes.",
      prompt: "you are the architecture persona. respond with a concrete plan, tradeoffs, rollout order, and critical failure modes before implementation details.",
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
  uapToken: "",
  taskLanePrefs: {},
  agentAuthTokens: {},
};

export const useConfigStore = defineStore("config", () => {
  const config = reactive<AppConfig>({ ...defaults });
  const loaded = ref(false);

  async function load() {
    try {
      const cfg = await invoke<AppConfig>("get_config");
      Object.assign(config, cfg);
    } catch {
      // First run — keep defaults
    }
    loaded.value = true;
    document.documentElement.setAttribute("data-theme", config.theme);
  }

  async function saveConfig(updates: AppConfig) {
    await invoke("save_config", { config: updates });
    Object.assign(config, updates);
    document.documentElement.setAttribute("data-theme", config.theme);
  }

  function setTheme(t: string) {
    config.theme = t;
    document.documentElement.setAttribute("data-theme", t);
    saveConfig({ ...config });
  }

  return { config, loaded, load, saveConfig, setTheme };
});
