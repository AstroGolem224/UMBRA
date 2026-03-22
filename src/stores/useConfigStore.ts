import { defineStore } from "pinia";
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "@/interfaces";

const defaults: AppConfig = {
  theme: "ember",
  vaultPath: "",
  notesSubdir: "UMBRA_Notes",
  repoRootPath: "",
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
