<template>
  <nav class="sidebar glass-panel">
    <ul class="nav-list">
      <li v-for="item in navItems" :key="item.path">
        <RouterLink :to="item.path" class="nav-item" :class="{ active: isActive(item.path) }">
          <span class="nav-icon">{{ item.icon }}</span>
          <span class="nav-label">{{ item.label }}</span>
          <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
        </RouterLink>
      </li>
    </ul>

    <div class="sidebar-footer">
      <div class="connection-status" :class="backendConnected ? 'connected' : 'disconnected'">
        <span class="status-dot" />
        <span class="status-label">{{ backendConnected ? "ONLINE" : "OFFLINE" }}</span>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useAgentStore } from "@/stores/useAgentStore";
import { useTaskStore } from "@/stores/useTaskStore";

const route = useRoute();
const agentStore = useAgentStore();
const taskStore = useTaskStore();

const backendConnected = computed(() => (agentStore.agents?.length ?? 0) > 0 || !agentStore.loading);
const activeTaskCount = computed(() => taskStore.tasks.filter((t) => t.status === "in-progress").length);

function isActive(path: string) {
  return route.path.startsWith(path);
}

const navItems = computed(() => [
  { path: "/dashboard", icon: "◈", label: "DASHBOARD" },
  { path: "/agents",    icon: "◉", label: "AGENTS" },
  { path: "/tasks",     icon: "▣", label: "TASKS", badge: activeTaskCount.value > 0 ? String(activeTaskCount.value) : undefined },
  { path: "/cron",      icon: "⏱", label: "CRON" },
  { path: "/notes",     icon: "◎", label: "NOTES" },
  { path: "/launcher",  icon: "▶", label: "LAUNCHER" },
  { path: "/plugins",   icon: "⊞", label: "PLUGINS" },
  { path: "/skills",    icon: "⚡", label: "SKILLS" },
  { path: "/settings",  icon: "⚙", label: "SETTINGS" },
]);
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 12px 0;
  flex-shrink: 0;
}

.nav-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 8px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 8px;
  text-decoration: none;
  color: var(--text-secondary);
  font-family: "Iceland", monospace;
  font-size: 12px;
  letter-spacing: 0.1em;
  transition: all 0.15s;
  border-left: 2px solid transparent;
}

.nav-item:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-dim);
  color: var(--accent);
  border-left-color: var(--accent);
  text-shadow: var(--glow-neon);
}

.nav-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}

.nav-label {
  flex: 1;
}

.nav-badge {
  background: var(--accent);
  color: var(--bg-primary);
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 4px;
  font-weight: 700;
}

.sidebar-footer {
  padding: 12px 20px 4px;
  border-top: 1px solid var(--glass-border);
  margin-top: 8px;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.connected .status-dot {
  background: var(--accent-success);
  box-shadow: 0 0 6px var(--accent-success);
}

.disconnected .status-dot {
  background: var(--accent-error);
}

.status-label {
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
}
</style>
