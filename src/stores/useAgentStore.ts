import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Agent } from "@/interfaces";

export const useAgentStore = defineStore("agents", () => {
  const agents = ref<Agent[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadAgents() {
    loading.value = true;
    error.value = null;
    try {
      agents.value = await invoke<Agent[]>("get_agents");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function setupLiveUpdates() {
    listen<{ agentId: string; status: Agent["status"]; activeTaskId?: string }>(
      "agent-status-changed",
      (event) => {
        const agent = agents.value.find((a) => a.id === event.payload.agentId);
        if (agent) {
          agent.status = event.payload.status;
          agent.activeTaskId = event.payload.activeTaskId;
          agent.lastSeen = new Date().toISOString();
        }
      }
    );
  }

  return { agents, loading, error, loadAgents, setupLiveUpdates };
});
