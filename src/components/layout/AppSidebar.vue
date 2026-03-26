<template>
  <aside class="sidebar">
    <nav class="nav-shell" aria-label="Primary">
      <ul class="nav-list">
        <li v-for="item in navItems" :key="item.path">
          <RouterLink :to="item.path" class="nav-item" :class="{ active: isActive(item.path) }">
            <span class="nav-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none">
                <path
                  v-for="(segment, index) in iconPaths(item.icon)"
                  :key="`${item.path}-${index}`"
                  :d="segment"
                />
              </svg>
            </span>
            <span class="nav-label">{{ item.label }}</span>
            <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
          </RouterLink>
        </li>
      </ul>
    </nav>

    <div class="sidebar-footer">
      <div class="footer-block">
        <p class="footer-kicker">telemetry</p>
        <div class="connection-status" :class="backendConnected ? 'connected' : 'disconnected'">
          <span class="status-dot" />
          <span class="status-label">{{ backendConnected ? "backend online" : "backend offline" }}</span>
        </div>
      </div>
      <div class="footer-block compact">
        <span>{{ activeTaskCount }} active tasks</span>
        <span>{{ onlineAgentCount }} live agents</span>
      </div>
    </div>
  </aside>
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
const onlineAgentCount = computed(() =>
  agentStore.agents.filter((agent) => ["online", "idle", "working"].includes(agent.status)).length
);

function isActive(path: string) {
  return route.path.startsWith(path);
}

function iconPaths(icon: string) {
  const icons: Record<string, string[]> = {
    dashboard: [
      "M4 4h7v7H4z",
      "M13 4h7v4h-7z",
      "M13 10h7v10h-7z",
      "M4 13h7v7H4z",
    ],
    agents: [
      "M9 11a4 4 0 1 0 0-8a4 4 0 0 0 0 8Z",
      "M17 13a3 3 0 1 0 0-6",
      "M3 20a6 6 0 0 1 12 0",
      "M15 20a5 5 0 0 0-2-4",
    ],
    tasks: [
      "M9 6h11",
      "M9 12h11",
      "M9 18h11",
      "M4 6.5l1.5 1.5L7.8 5.7",
      "M4 12.5l1.5 1.5l2.3-2.3",
      "M4 18.5l1.5 1.5l2.3-2.3",
    ],
    cron: [
      "M12 7v5l3 3",
      "M12 3a9 9 0 1 0 9 9",
      "M21 6v4h-4",
    ],
    notes: [
      "M7 3h7l5 5v13H7z",
      "M14 3v5h5",
      "M10 13h6",
      "M10 17h6",
    ],
    launcher: [
      "M5 19h14",
      "M12 5v10",
      "M8 9l4-4l4 4",
    ],
    workbench: [
      "M4 7h16",
      "M4 12h10",
      "M4 17h7",
      "M17 11l3 3l-3 3",
    ],
    ops: [
      "M5 6h14",
      "M5 12h8",
      "M5 18h10",
      "M16 10l3 2l-3 2",
      "M14 16l3 2l-3 2",
    ],
    plugins: [
      "M10 4.5V9H5.5",
      "M18.5 10H14V5.5",
      "M10 14v4.5H5.5",
      "M18.5 14H14v4.5",
      "M10 9H5.5v5H10z",
      "M18.5 9H14v5h4.5z",
    ],
    skills: [
      "M12 3l1.7 5.2H19l-4.3 3.1l1.6 5.2L12 13.4L7.7 16.5l1.6-5.2L5 8.2h5.3z",
    ],
    settings: [
      "M12 8.5a3.5 3.5 0 1 0 0 7a3.5 3.5 0 0 0 0-7Z",
      "M19.4 15a1 1 0 0 0 .2 1.1l.1.1a1.2 1.2 0 0 1 0 1.7l-1.1 1.1a1.2 1.2 0 0 1-1.7 0l-.1-.1a1 1 0 0 0-1.1-.2a1 1 0 0 0-.6.9V21a1.2 1.2 0 0 1-1.2 1.2h-1.6A1.2 1.2 0 0 1 11.1 21v-.2a1 1 0 0 0-.6-.9a1 1 0 0 0-1.1.2l-.1.1a1.2 1.2 0 0 1-1.7 0l-1.1-1.1a1.2 1.2 0 0 1 0-1.7l.1-.1a1 1 0 0 0 .2-1.1a1 1 0 0 0-.9-.6H5A1.2 1.2 0 0 1 3.8 13v-1.6A1.2 1.2 0 0 1 5 10.2h.2a1 1 0 0 0 .9-.6a1 1 0 0 0-.2-1.1l-.1-.1a1.2 1.2 0 0 1 0-1.7l1.1-1.1a1.2 1.2 0 0 1 1.7 0l.1.1a1 1 0 0 0 1.1.2a1 1 0 0 0 .6-.9V5A1.2 1.2 0 0 1 11.6 3.8h1.6A1.2 1.2 0 0 1 14.4 5v.2a1 1 0 0 0 .6.9a1 1 0 0 0 1.1-.2l.1-.1a1.2 1.2 0 0 1 1.7 0l1.1 1.1a1.2 1.2 0 0 1 0 1.7l-.1.1a1 1 0 0 0-.2 1.1a1 1 0 0 0 .9.6h.2a1.2 1.2 0 0 1 1.2 1.2V13a1.2 1.2 0 0 1-1.2 1.2h-.2a1 1 0 0 0-.9.8Z",
    ],
  };

  return icons[icon] ?? icons.dashboard;
}

const navItems = computed(() => [
  { path: "/dashboard", label: "Dashboard", icon: "dashboard" },
  { path: "/agents", label: "Agents", icon: "agents" },
  { path: "/workbench", label: "Workbench", icon: "workbench" },
  { path: "/ops-room", label: "Ops Room", icon: "ops" },
  {
    path: "/tasks",
    label: "Tasks",
    icon: "tasks",
    badge: activeTaskCount.value > 0 ? String(activeTaskCount.value) : undefined,
  },
  { path: "/cron", label: "Cron", icon: "cron" },
  { path: "/notes", label: "Notes", icon: "notes" },
  { path: "/launcher", label: "Launcher", icon: "launcher" },
  { path: "/plugins", label: "Plugins", icon: "plugins" },
  { path: "/skills", label: "Skills", icon: "skills" },
  { path: "/settings", label: "Settings", icon: "settings" },
]);
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: calc(var(--stage-edge-pad) + var(--stage-inner-pad)) 14px 14px;
  flex-shrink: 0;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--bg-secondary) 94%, transparent), color-mix(in srgb, var(--bg-primary) 92%, transparent));
  border-right: 1px solid color-mix(in srgb, var(--glass-border) 82%, transparent);
}

.sidebar-footer {
  border: 1px solid color-mix(in srgb, var(--glass-border) 90%, transparent);
  border-radius: var(--radius-xl);
  background: color-mix(in srgb, var(--glass-bg) 94%, transparent);
}

.footer-kicker,
.status-label {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.nav-shell {
  flex: 1;
  padding: 0;
}

.nav-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  padding: 12px 14px;
  border-radius: var(--radius-lg);
  border: 1px solid transparent;
  text-decoration: none;
  transition: background 0.16s ease, border-color 0.16s ease, transform 0.16s ease;
}

.nav-item:hover {
  background: color-mix(in srgb, var(--accent) 6%, transparent);
  transform: translateX(1px);
}

.nav-item.active {
  background: color-mix(in srgb, var(--accent) 10%, var(--bg-surface));
  border-color: transparent;
}

.nav-icon {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--text-muted);
  transition: color 0.16s ease;
}

.nav-item:hover .nav-icon,
.nav-item.active .nav-icon {
  color: var(--accent);
}

.nav-item.active .nav-icon {
  filter: drop-shadow(0 0 4px color-mix(in srgb, var(--accent) 40%, transparent));
}

.nav-icon svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.nav-label {
  flex: 1 1 auto;
  color: var(--text-secondary);
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 700;
  line-height: 0.96;
  letter-spacing: 0.03em;
  transition: color 0.16s ease;
}

.nav-item:hover .nav-label,
.nav-item.active .nav-label {
  color: var(--text-primary);
}

.nav-badge {
  margin-left: auto;
  min-width: 22px;
  padding: 3px 7px;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  border: none;
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 10px;
  text-align: center;
}

.sidebar-footer {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
}

.footer-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.footer-block.compact {
  gap: 4px;
  color: var(--text-secondary);
  font-size: 12px;
}

.connection-status {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.connected .status-dot {
  background: var(--accent-success);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent-success) 16%, transparent);
}

.disconnected .status-dot {
  background: var(--accent-error);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent-error) 16%, transparent);
}

@media (max-width: 1080px) {
  .sidebar {
    width: 208px;
    padding: 28px 10px 10px;
  }

  .nav-label {
    font-size: 18px;
  }
}
</style>
