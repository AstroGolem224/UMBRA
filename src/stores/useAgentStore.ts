import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCache } from "@/composables/useCache";
import type { Agent, AgentTask, CustomAgentConfig } from "@/interfaces";

const { cached, invalidate } = useCache();

export const useAgentStore = defineStore("agents", () => {
  const agents = ref<Agent[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadAgents(force = false) {
    if (force) invalidate("agents");
    loading.value = true;
    error.value = null;
    try {
      agents.value = await cached("agents", () => invoke<Agent[]>("get_agents"), 10_000);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  let _listenUnlisten: (() => void) | null = null;

  async function setupLiveUpdates() {
    if (_listenUnlisten) return;
    try {
      // The UAP server emits full Agent objects on heartbeat.
      _listenUnlisten = await listen<Agent>("agent-status-changed", (event) => {
        const incoming = event.payload;
        const idx = agents.value.findIndex((a) => a.id === incoming.id);
        if (idx >= 0) {
          agents.value[idx] = incoming;
        } else {
          agents.value.push(incoming);
        }
      });
    } catch {
      // Tauri event API not available (browser mode)
    }
  }

  async function addAgent(cfg: CustomAgentConfig): Promise<Agent> {
    const agent = await invoke<Agent>("add_agent", { agent: cfg });
    const idx = agents.value.findIndex((a) => a.id === agent.id);
    if (idx >= 0) agents.value[idx] = agent;
    else agents.value.push(agent);
    return agent;
  }

  async function removeAgent(id: string) {
    await invoke("remove_agent", { id });
    agents.value = agents.value.filter((a) => a.id !== id);
  }

  async function pushTask(agentId: string, task: Omit<AgentTask, "id">) {
    await invoke("push_agent_task", {
      agentId,
      title: task.title,
      description: task.description ?? null,
      priority: task.priority ?? "medium",
    });
  }

  return { agents, loading, error, loadAgents, setupLiveUpdates, addAgent, removeAgent, pushTask };
});
