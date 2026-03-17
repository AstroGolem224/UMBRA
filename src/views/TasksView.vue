<template>
  <div class="tasks-view">
    <header class="page-header">
      <div class="header-left">
        <h1 class="page-title">TASKS</h1>
        <span class="page-subtitle">// PM Tool</span>
      </div>
      <div class="header-right">
        <span v-if="taskStore.lastSync" class="sync-time">
          synced {{ relativeSync }}
        </span>
        <NeonButton size="sm" :loading="taskStore.loading" @click="taskStore.fetchTasks()">
          SYNC
        </NeonButton>
      </div>
    </header>

    <div v-if="taskStore.error" class="error-bar">{{ taskStore.error }}</div>

    <div v-if="taskStore.loading && taskStore.tasks.length === 0" class="loading-state">
      Loading tasks...
    </div>

    <div v-else class="kanban">
      <div v-for="col in columns" :key="col.status" class="kanban-col">
        <div class="col-header">
          <span class="col-dot" :class="col.status" />
          <span class="col-title">{{ col.label }}</span>
          <span class="col-count">{{ col.tasks.length }}</span>
        </div>

        <div class="col-body">
          <div v-if="col.tasks.length === 0" class="col-empty">no tasks</div>
          <GlassCard
            v-for="task in col.tasks"
            :key="task.id"
            class="task-card"
            :variant="task.status === 'blocked' ? 'danger' : 'default'"
          >
            <div class="task-title">{{ task.title }}</div>
            <div class="task-meta">
              <span class="task-project">{{ task.project }}</span>
              <span class="task-priority" :class="task.priority">{{ task.priority }}</span>
            </div>
            <div v-if="task.agent" class="task-agent">{{ task.agent }}</div>
          </GlassCard>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useTaskStore } from "@/stores/useTaskStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import type { Task } from "@/interfaces";

const taskStore = useTaskStore();

const COLUMNS: { status: Task["status"]; label: string }[] = [
  { status: "in-progress", label: "IN PROGRESS" },
  { status: "todo",        label: "TODO" },
  { status: "blocked",     label: "BLOCKED" },
  { status: "done",        label: "DONE" },
];

const columns = computed(() =>
  COLUMNS.map((col) => ({
    ...col,
    tasks: taskStore.tasks.filter((t) => t.status === col.status),
  }))
);

const relativeSync = computed(() => {
  if (!taskStore.lastSync) return "";
  const diff = Date.now() - new Date(taskStore.lastSync).getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return `${secs}s ago`;
  return `${Math.floor(secs / 60)}m ago`;
});

onMounted(() => {
  taskStore.fetchTasks();
  taskStore.setupLiveUpdates();
});
</script>

<style scoped>
.tasks-view {
  max-width: 1400px;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.page-title {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-primary);
  margin: 0;
}

.page-subtitle {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  letter-spacing: 0.08em;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sync-time {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}

.error-bar {
  margin-bottom: 16px;
  padding: 10px 14px;
  border-radius: 6px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid var(--accent-error);
  color: var(--accent-error);
  font-family: var(--font-mono);
  font-size: 12px;
}

.loading-state {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-muted);
  padding: 32px 0;
}

/* ─── Kanban Board ────────────────────────── */
.kanban {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  align-items: start;
}

@media (max-width: 1100px) {
  .kanban { grid-template-columns: repeat(2, 1fr); }
}

.kanban-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.col-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 2px 8px;
  border-bottom: 1px solid var(--glass-border);
  margin-bottom: 4px;
}

.col-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--text-muted);
}

.col-dot.in-progress {
  background: var(--accent);
  box-shadow: 0 0 6px var(--accent-dim);
  animation: pulse-dot 2.5s ease-in-out infinite;
}

.col-dot.todo     { background: var(--text-secondary); }
.col-dot.blocked  { background: var(--accent-error); box-shadow: 0 0 6px rgba(239,68,68,0.4); }
.col-dot.done     { background: var(--neon-green); }
.col-dot.cancelled{ background: var(--text-muted); }

@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.45; }
}

.col-title {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--text-secondary);
  text-transform: uppercase;
  flex: 1;
}

.col-count {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-surface);
  border: 1px solid var(--glass-border);
  border-radius: 4px;
  padding: 1px 6px;
}

.col-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.col-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  padding: 12px 4px;
}

/* ─── Task Card ───────────────────────────── */
.task-card {
  cursor: default;
}

.task-title {
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 1.4;
  margin-bottom: 6px;
}

.task-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.task-project {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-priority {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
  border: 1px solid;
}

.task-priority.critical {
  color: var(--accent-error);
  border-color: rgba(239, 68, 68, 0.3);
  background: rgba(239, 68, 68, 0.08);
}
.task-priority.high {
  color: var(--neon-amber);
  border-color: rgba(245, 158, 11, 0.3);
  background: rgba(245, 158, 11, 0.08);
}
.task-priority.medium {
  color: var(--accent);
  border-color: var(--glass-border);
  background: var(--bg-surface);
}
.task-priority.low {
  color: var(--text-muted);
  border-color: var(--glass-border);
  background: transparent;
}

.task-agent {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--accent);
  margin-top: 4px;
  opacity: 0.7;
}
</style>
