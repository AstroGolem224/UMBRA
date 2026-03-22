import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentCronJob } from "@/interfaces";

export const useCronStore = defineStore("cron", () => {
  const jobs = ref<AgentCronJob[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastSync = ref<string | null>(null);
  let _listenUnlisten: (() => void) | null = null;

  async function loadJobs() {
    loading.value = true;
    error.value = null;
    try {
      jobs.value = await invoke<AgentCronJob[]>("list_agent_cron_jobs");
      lastSync.value = new Date().toISOString();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function setupLiveUpdates() {
    if (_listenUnlisten) return;
    try {
      _listenUnlisten = await listen<AgentCronJob[]>("agent-cron-updated", (event) => {
        jobs.value = event.payload;
        lastSync.value = new Date().toISOString();
      });
    } catch {
      // Tauri event API not available (browser mode)
    }
  }

  return {
    jobs,
    loading,
    error,
    lastSync,
    loadJobs,
    setupLiveUpdates,
  };
});
