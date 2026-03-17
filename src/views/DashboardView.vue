<template>
  <div class="dashboard">
    <header class="page-header">
      <h1 class="page-title">DASHBOARD</h1>
      <span class="page-subtitle">System Overview</span>
    </header>

    <div class="widget-grid">
      <GlassCard class="widget">
        <h3 class="widget-title">AGENTS</h3>
        <div class="agent-summary">
          <div v-for="agent in agentStore.agents" :key="agent.id" class="agent-row">
            <StatusBadge :status="agent.status" />
            <span class="agent-name">{{ agent.name }}</span>
          </div>
          <div v-if="agentStore.loading" class="loading-text">Loading...</div>
        </div>
      </GlassCard>

      <GlassCard class="widget">
        <h3 class="widget-title">NOTES</h3>
        <div class="stat-row">
          <span class="stat-value">{{ notesStore.notes.length }}</span>
          <span class="stat-label">total notes</span>
        </div>
        <div class="category-pills">
          <span v-for="cat in noteCategories" :key="cat" class="category-pill">
            {{ cat }} ({{ countByCategory(cat) }})
          </span>
        </div>
      </GlassCard>

      <GlassCard class="widget">
        <h3 class="widget-title">TASKS</h3>
        <div v-if="taskStore.tasks.length === 0" class="empty-text">No tasks loaded</div>
        <div v-else class="task-list">
          <div v-for="task in taskStore.tasks.slice(0, 5)" :key="task.id" class="task-row">
            <span class="task-status-dot" :class="task.status" />
            <span class="task-title">{{ task.title }}</span>
          </div>
        </div>
      </GlassCard>

      <GlassCard class="widget">
        <h3 class="widget-title">SYSTEM</h3>
        <div class="sys-rows">
          <div class="sys-row">
            <span class="sys-label">Platform</span>
            <span class="sys-val">Windows 11</span>
          </div>
          <div class="sys-row">
            <span class="sys-label">App Version</span>
            <span class="sys-val">0.1.0</span>
          </div>
          <div class="sys-row">
            <span class="sys-label">Theme</span>
            <span class="sys-val">{{ configStore.config.theme }}</span>
          </div>
        </div>
      </GlassCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useAgentStore } from "@/stores/useAgentStore";
import { useNotesStore } from "@/stores/useNotesStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useConfigStore } from "@/stores/useConfigStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import StatusBadge from "@/components/ui/StatusBadge.vue";
import type { NoteCategory } from "@/interfaces";

const agentStore = useAgentStore();
const notesStore = useNotesStore();
const taskStore = useTaskStore();
const configStore = useConfigStore();

onMounted(() => {
  agentStore.loadAgents();
  notesStore.loadNotes();
  taskStore.fetchTasks();
  taskStore.setupLiveUpdates();
});

const noteCategories: NoteCategory[] = ["prompts", "cli", "agents", "skills", "misc"];

function countByCategory(cat: NoteCategory) {
  return notesStore.notes.filter((n) => n.category === cat).length;
}
</script>

<style scoped>
.dashboard {
  max-width: 1200px;
}

.page-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 24px;
}

.page-title {
  font-family: "Iceland", monospace;
  font-size: 24px;
  letter-spacing: 0.2em;
  color: var(--text-primary);
  margin: 0;
}

.page-subtitle {
  font-size: 12px;
  color: var(--text-muted);
  letter-spacing: 0.1em;
}

.widget-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.widget-title {
  font-family: "Iceland", monospace;
  font-size: 11px;
  letter-spacing: 0.2em;
  color: var(--text-muted);
  margin: 0 0 12px;
  text-transform: uppercase;
}

.agent-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.agent-name {
  font-size: 12px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.stat-row {
  display: flex;
  align-items: baseline;
  gap: 6px;
  margin-bottom: 10px;
}

.stat-value {
  font-family: "Iceland", monospace;
  font-size: 32px;
  color: var(--accent);
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted);
}

.category-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.category-pill {
  font-size: 10px;
  font-family: "Iceland", monospace;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--bg-surface);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
}

.empty-text,
.loading-text {
  font-size: 12px;
  color: var(--text-muted);
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.task-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--text-muted);
}
.task-status-dot.in-progress { background: var(--accent); }
.task-status-dot.done { background: var(--accent-success); }
.task-status-dot.blocked { background: var(--accent-error); }

.task-title {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sys-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sys-row {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.sys-label {
  color: var(--text-muted);
}

.sys-val {
  color: var(--text-secondary);
  font-family: "Iceland", monospace;
  letter-spacing: 0.06em;
}
</style>
