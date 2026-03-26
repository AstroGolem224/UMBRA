<template>
  <div class="dashboard-view">
    <ViewHero
      kicker="operational integrity // phase 04"
      title="Umbra Dashboard Overview"
      subtitle="agent status, task signals, and cron telemetry at a glance."
    >
      <template #meta>
        <div class="view-hero-pill" :class="{ 'is-stale': !taskStore.lastSync }">
          {{ taskStore.lastSync ? `pm ${formatRelativeTime(taskStore.lastSync)}` : "pm sync pending" }}
        </div>
        <div class="view-hero-pill" :class="{ 'is-stale': !cronStore.lastSync }">
          {{ cronStore.lastSync ? `cron ${formatRelativeTime(cronStore.lastSync)}` : "cron awaiting agent push" }}
        </div>
        <div class="view-hero-pill">live sync</div>
        <div class="view-hero-pill nominal-pill">{{ attentionItems.length ? "attention" : "nominal" }}</div>
      </template>
    </ViewHero>

    <div v-if="isLoading" class="dashboard-loading">
      <div class="skeleton-grid">
        <SkeletonBlock variant="card" height="320px" />
        <div class="skeleton-col">
          <SkeletonBlock variant="card" height="150px" />
          <SkeletonBlock variant="card" height="80px" />
        </div>
      </div>
      <SkeletonBlock variant="card" height="200px" />
      <div class="skeleton-row">
        <SkeletonBlock variant="card" height="180px" />
        <SkeletonBlock variant="card" height="180px" />
        <SkeletonBlock variant="card" height="180px" />
      </div>
    </div>

    <section v-else class="dashboard-stage">
      <div class="metrics-grid">
        <GlassCard class="hero-card">
          <div class="card-head">
            <div>
              <p class="eyebrow">task volume</p>
              <h2 class="card-title">Total Tasks</h2>
            </div>
            <span class="card-icon">⌁</span>
          </div>

          <div class="hero-metric">
            <strong>{{ statusMetrics[0]?.value ?? 0 }}</strong>
            <span class="metric-unit">live</span>
          </div>
          <p class="metric-delta">{{ topSuggestion ? `route to ${topSuggestion.agent.name} next` : "waiting for broker signal" }}</p>

          <div class="chart-shell" aria-label="task volume chart">
            <div v-for="metric in statusMetrics" :key="metric.label" class="chart-column">
              <div class="chart-bar-track">
                <div class="chart-bar" :style="{ height: `${metric.height}%` }" />
              </div>
              <div class="chart-meta">
                <strong>{{ metric.value }}</strong>
                <span>{{ metric.label }}</span>
              </div>
            </div>
          </div>
        </GlassCard>

        <GlassCard class="mini-card">
          <div class="card-head compact">
            <div>
              <p class="eyebrow">compute load</p>
              <h3 class="card-title minor">{{ loadMetric }}%</h3>
            </div>
            <div class="orbital-ring">
              <div class="orbital-ring__inner">{{ onlineAgents }}</div>
            </div>
          </div>
          <p class="support-copy">{{ onlineAgents }} live agents / {{ staleAgents.length }} stale</p>
        </GlassCard>

        <GlassCard class="cron-card">
          <p class="eyebrow light">next cron cycle</p>
          <h3>{{ nextCronJob ? formatDeadline(nextCronJob.nextRun) : "none" }}</h3>
          <p>{{ nextCronJob ? `${nextCronJob.job} / ${nextCronJob.agentName}` : "no cron jobs scheduled" }}</p>
        </GlassCard>
      </div>

      <GlassCard class="registry-card">
        <div class="section-head">
          <div>
            <h3 class="section-title">Deployment Registry</h3>
            <p class="section-copy">recent tasks from the PM Tool with project, status and activity.</p>
          </div>
          <div class="section-actions">
            <span class="ghost-action">{{ registryRows.length }} of {{ taskStore.tasks.length }}</span>
          </div>
        </div>

        <div class="registry-table">
          <div class="registry-row registry-row--head">
            <span>task</span>
            <span>project</span>
            <span>status</span>
            <span>priority</span>
            <span>activity</span>
          </div>

          <article v-for="row in registryRows" :key="row.id" class="registry-row">
            <div class="asset-cell">
              <span class="asset-dot" :class="row.dotTone" />
              <div>
                <strong>{{ row.name }}</strong>
                <p>{{ row.meta }}</p>
              </div>
            </div>
            <span class="mono">{{ row.region }}</span>
            <span class="status-chip" :class="`status-chip--${row.statusTone}`">{{ row.statusLabel }}</span>
            <span class="status-chip status-chip--neutral">{{ row.priority }}</span>
            <span class="mono strong">{{ row.threads }}</span>
          </article>
        </div>
      </GlassCard>

      <div class="summary-grid">
        <GlassCard class="summary-card">
          <div class="section-head compact">
            <div>
              <p class="eyebrow">attention rail</p>
              <h3 class="section-title minor">Priority Signals</h3>
            </div>
          </div>
          <div v-if="attentionItems.length" class="summary-list">
            <article v-for="item in attentionItems" :key="item.id" class="summary-row">
              <div>
                <strong>{{ item.title }}</strong>
                <p>{{ item.detail }}</p>
              </div>
              <span class="status-chip" :class="`status-chip--${item.level}`">{{ item.kind }}</span>
            </article>
          </div>
          <div v-else class="empty-state">no overdue tasks, failing cron jobs or stale agents right now.</div>
        </GlassCard>

        <GlassCard class="summary-card">
          <div class="section-head compact">
            <div>
              <p class="eyebrow">timeline</p>
              <h3 class="section-title minor">Upcoming Deadlines</h3>
            </div>
          </div>
          <div v-if="upcomingDeadlines.length" class="summary-list">
            <article v-for="task in upcomingDeadlines" :key="task.id" class="summary-row">
              <div>
                <strong>{{ task.title }}</strong>
                <p>{{ task.project }} / {{ formatDeadline(task.deadline ?? task.nextDueDate) }}</p>
              </div>
              <span class="status-chip status-chip--neutral">{{ task.priority }}</span>
            </article>
          </div>
          <div v-else class="empty-state">no due dates visible in synced PM data.</div>
        </GlassCard>

        <GlassCard class="summary-card">
          <div class="section-head compact">
            <div>
              <p class="eyebrow">telemetry</p>
              <h3 class="section-title minor">Recent Activity</h3>
            </div>
          </div>
          <div v-if="recentActivity.length" class="summary-list">
            <article v-for="activity in recentActivity" :key="activity.id" class="summary-row">
              <div>
                <strong>{{ activity.title }}</strong>
                <p>{{ activity.project }} / {{ activity.kind }}</p>
              </div>
              <span class="status-chip status-chip--info">{{ formatRelativeTime(activity.timestamp) }}</span>
            </article>
          </div>
          <div v-else class="empty-state">no recent task activity yet.</div>
        </GlassCard>
      </div>

      <div class="telemetry-strip">
        <span>pm sync: {{ taskStore.lastSync ? formatRelativeTime(taskStore.lastSync) : "pending" }}</span>
        <span>agents: {{ onlineAgents }} online / {{ agentStore.agents.length }} total</span>
        <span>tasks: {{ taskStore.tasks.length }} tracked</span>
        <span>attention: {{ attentionItems.length }} signal{{ attentionItems.length !== 1 ? "s" : "" }}</span>
        <span>cron: {{ cronStore.jobs.length }} job{{ cronStore.jobs.length !== 1 ? "s" : "" }}</span>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import ViewHero from "@/components/layout/ViewHero.vue";
import GlassCard from "@/components/ui/GlassCard.vue";
import SkeletonBlock from "@/components/ui/SkeletonBlock.vue";
import { buildAssignmentSuggestions } from "@/lib/assignment-broker";
import type { Task } from "@/interfaces";
import { useAgentStore } from "@/stores/useAgentStore";
import { useCronStore } from "@/stores/useCronStore";
import { useNotesStore } from "@/stores/useNotesStore";
import { useTaskStore } from "@/stores/useTaskStore";

type ActivityItem = {
  id: string;
  title: string;
  project: string;
  kind: string;
  timestamp: string;
};

type AttentionItem = {
  id: string;
  kind: string;
  title: string;
  detail: string;
  level: "danger" | "warn" | "info";
};

type RegistryRow = {
  id: string;
  name: string;
  meta: string;
  region: string;
  statusLabel: string;
  statusTone: "success" | "warn" | "danger" | "info";
  dotTone: "success" | "warn" | "danger" | "info";
  priority: string;
  threads: string;
};

const agentStore = useAgentStore();
const cronStore = useCronStore();
const notesStore = useNotesStore();
const taskStore = useTaskStore();
const isLoading = ref(true);

const priorityWeight: Record<Task["priority"], number> = {
  urgent: 5,
  critical: 4,
  high: 3,
  medium: 2,
  low: 1,
};

const onlineAgents = computed(() => agentStore.agents.filter((agent) => ["online", "working", "idle"].includes(agent.status)).length);
const staleAgents = computed(() => agentStore.agents.filter((agent) => Date.now() - new Date(agent.lastSeen).getTime() > 30 * 60_000));
const overdueTasks = computed(() =>
  taskStore.tasks.filter((task) => {
    const due = task.deadline ?? task.nextDueDate;
    return Boolean(due) && !["done", "cancelled"].includes(task.status) && new Date(due!).getTime() < Date.now();
  }),
);

const suggestions = computed(() => buildAssignmentSuggestions(taskStore.tasks, agentStore.agents, 4));
const topSuggestion = computed(() => suggestions.value[0]);

const nextCronJob = computed(() =>
  [...cronStore.jobs]
    .filter((job) => Boolean(job.nextRun))
    .sort((a, b) => new Date(a.nextRun ?? "").getTime() - new Date(b.nextRun ?? "").getTime())[0],
);

const upcomingDeadlines = computed(() =>
  [...taskStore.tasks]
    .filter((task) => Boolean(task.deadline ?? task.nextDueDate))
    .sort((a, b) => {
      const aTime = new Date(a.deadline ?? a.nextDueDate ?? "").getTime();
      const bTime = new Date(b.deadline ?? b.nextDueDate ?? "").getTime();
      return aTime - bTime || priorityWeight[b.priority] - priorityWeight[a.priority];
    })
    .slice(0, 4),
);

const attentionItems = computed<AttentionItem[]>(() => {
  const items: AttentionItem[] = [
    ...cronStore.jobs.filter((job) => job.lastStatus === "error").slice(0, 2).map((job) => ({
      id: `cron-${job.id}`,
      kind: "cron",
      title: job.job,
      detail: `${job.agentName} / ${job.notes || job.lastStatus}`,
      level: "danger" as const,
    })),
    ...overdueTasks.value.slice(0, 2).map((task) => ({
      id: `task-${task.id}`,
      kind: "task",
      title: task.title,
      detail: `${task.project} / overdue since ${formatDeadline(task.deadline ?? task.nextDueDate)}`,
      level: "warn" as const,
    })),
    ...staleAgents.value.slice(0, 2).map((agent) => ({
      id: `agent-${agent.id}`,
      kind: "agent",
      title: agent.name,
      detail: `${agent.role || "no role"} / last seen ${formatRelativeTime(agent.lastSeen)}`,
      level: "info" as const,
    })),
  ];

  return items;
});

const statusMetrics = computed(() => {
  const metrics = [
    { label: "Total Tasks", value: taskStore.tasks.length },
    { label: "In Progress", value: taskStore.tasks.filter((task) => task.status === "in-progress").length },
    { label: "Completed", value: taskStore.tasks.filter((task) => task.status === "done").length },
    { label: "Overdue", value: overdueTasks.value.length },
  ];
  const max = Math.max(...metrics.map((metric) => metric.value), 1);
  return metrics.map((metric) => ({
    ...metric,
    height: Math.max(18, Math.round((metric.value / max) * 100)),
  }));
});

const loadMetric = computed(() => {
  if (!agentStore.agents.length) return 0;
  return Math.round((onlineAgents.value / agentStore.agents.length) * 100);
});

const registryRows = computed<RegistryRow[]>(() =>
  taskStore.tasks.slice(0, 6).map((task) => {
    const statusMap: Record<string, RegistryRow["statusTone"]> = {
      done: "success",
      blocked: "warn",
      cancelled: "danger",
      "in-progress": "info",
      todo: "info",
    };

    const statusLabelMap: Record<string, string> = {
      done: "done",
      blocked: "blocked",
      cancelled: "cancelled",
      "in-progress": "in progress",
      todo: "todo",
    };

    return {
      id: task.id,
      name: task.title,
      meta: `id: ${task.id.slice(0, 8)}`,
      region: task.project,
      statusLabel: statusLabelMap[task.status] ?? task.status,
      statusTone: statusMap[task.status] ?? "info",
      dotTone: statusMap[task.status] ?? "info",
      priority: task.priority,
      threads: `${task.comments?.length ?? 0} comment${(task.comments?.length ?? 0) !== 1 ? "s" : ""}`,
    };
  }),
);

const recentActivity = computed<ActivityItem[]>(() => {
  const activity = taskStore.tasks.flatMap((task) => {
    const items: ActivityItem[] = [];
    if (task.createdAt) {
      items.push({ id: `${task.id}-created`, title: task.title, project: task.project, kind: "created", timestamp: task.createdAt });
    }
    if (task.updatedAt && task.createdAt && task.updatedAt !== task.createdAt) {
      items.push({ id: `${task.id}-updated`, title: task.title, project: task.project, kind: task.status === "done" ? "completed" : "updated", timestamp: task.updatedAt });
    }
    for (const comment of task.comments ?? []) {
      items.push({ id: comment.id, title: task.title, project: task.project, kind: "commented", timestamp: comment.createdAt });
    }
    return items;
  });

  return activity.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()).slice(0, 4);
});

function formatDeadline(value?: string | null) {
  if (!value) return "no date";
  return new Intl.DateTimeFormat("de-DE", {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

function formatRelativeTime(value: string) {
  const deltaMs = new Date(value).getTime() - Date.now();
  const minutes = Math.round(deltaMs / 60_000);
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  if (Math.abs(minutes) < 60) return formatter.format(minutes, "minute");
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) return formatter.format(hours, "hour");
  const days = Math.round(hours / 24);
  return formatter.format(days, "day");
}

onMounted(async () => {
  await Promise.all([
    agentStore.loadAgents(),
    agentStore.setupLiveUpdates(),
    cronStore.loadJobs(),
    cronStore.setupLiveUpdates(),
    notesStore.loadNotes(),
    taskStore.fetchTasks(),
    taskStore.setupLiveUpdates(),
  ]);
  isLoading.value = false;
});
</script>

<style scoped>
.dashboard-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.dashboard-stage {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding-bottom: 20px;
  position: relative;
}

.dashboard-stage::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image: radial-gradient(color-mix(in srgb, var(--glass-border) 90%, transparent) 0.8px, transparent 0.8px);
  background-size: 26px 26px;
  opacity: 0.55;
}

.dashboard-stage > * {
  position: relative;
  z-index: 1;
}

.metrics-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(280px, 0.75fr);
  gap: 18px;
}

.hero-card,
.registry-card,
.summary-card,
.mini-card,
.cron-card {
  border-radius: 20px;
  box-shadow: var(--glow-primary);
}

.hero-card {
  min-height: 320px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.card-head,
.section-head,
.registry-row,
.summary-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.card-head.compact {
  align-items: center;
}

.eyebrow {
  margin: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.eyebrow.light,
.cron-card p {
  color: rgba(255, 255, 255, 0.82);
}

:global([data-theme="light"]) .eyebrow.light,
:global([data-theme="light"]) .cron-card .eyebrow.light,
:global([data-theme="light"]) .cron-card p {
  color: rgba(255, 255, 255, 0.88);
}

.card-title,
.section-title,
.hero-metric strong,
.mini-card h3,
.cron-card h3 {
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-display);
}

.card-title {
  font-size: 24px;
  letter-spacing: 0.03em;
}

.card-title.minor,
.section-title.minor {
  font-size: 20px;
}

.card-icon {
  color: var(--accent);
  font-size: 22px;
}

.hero-metric {
  display: flex;
  align-items: flex-end;
  gap: 8px;
}

.hero-metric strong {
  font-size: 74px;
  line-height: 0.88;
  letter-spacing: -0.04em;
}

.metric-unit,
.metric-delta,
.section-copy,
.summary-row p,
.asset-cell p,
.support-copy,
.telemetry-strip {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.metric-unit {
  color: var(--rune-glow);
  font-family: var(--font-mono);
  font-size: 18px;
  text-transform: uppercase;
}

.chart-shell {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  align-items: end;
  min-height: 150px;
  margin-top: auto;
}

.chart-column {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: stretch;
}

.chart-bar-track {
  height: 116px;
  display: flex;
  align-items: flex-end;
  padding: 8px;
  border-radius: 16px 16px 8px 8px;
  background: color-mix(in srgb, var(--bg-secondary) 82%, white 18%);
}

.chart-bar {
  width: 100%;
  border-radius: 10px 10px 4px 4px;
  background: linear-gradient(180deg, var(--rune-glow), var(--accent));
  box-shadow: 0 16px 28px color-mix(in srgb, var(--accent) 18%, transparent);
}

.chart-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.chart-meta strong,
.asset-cell strong,
.summary-row strong {
  color: var(--text-primary);
  font-size: 14px;
}

.chart-meta span,
.asset-cell p,
.summary-row p,
.telemetry-strip span {
  color: var(--text-muted);
  font-size: 11px;
}

.mini-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 150px;
}

.orbital-ring {
  width: 72px;
  height: 72px;
  border-radius: 999px;
  border: 4px solid color-mix(in srgb, var(--glass-border) 72%, transparent);
  border-top-color: var(--rune-glow);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  font-family: var(--font-mono);
}

.orbital-ring__inner {
  width: 44px;
  height: 44px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--accent) 10%, white 90%);
  font-size: 12px;
}

.cron-card {
  margin-top: 18px;
  padding: 22px;
  background: linear-gradient(135deg, color-mix(in srgb, var(--accent) 92%, black 8%), color-mix(in srgb, var(--rune-glow) 18%, var(--accent)));
}

.cron-card h3,
.cron-card p {
  color: white;
}

[data-theme="light"] .cron-card {
  background: linear-gradient(135deg, #005a7d, #00718e);
}

[data-theme="light"] .cron-card h3,
[data-theme="light"] .cron-card p {
  color: #ffffff;
}

.registry-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.ghost-action {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.registry-table {
  display: flex;
  flex-direction: column;
}

.registry-row {
  align-items: center;
  padding: 14px 0;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 62%, transparent);
}

.registry-row--head {
  padding-top: 0;
  border-top: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.registry-row > span,
.registry-row > div {
  flex: 1 1 0;
  min-width: 0;
}

.asset-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.asset-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  flex-shrink: 0;
}

.asset-dot.success { background: #22c55e; }
.asset-dot.warn { background: #f59e0b; }
.asset-dot.danger { background: #ef4444; }
.asset-dot.info { background: var(--rune-glow); }

.status-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  padding: 4px 9px;
  border-radius: 999px;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  border: none;
}

.status-chip--success {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.12);
}

.status-chip--warn {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.12);
}

.status-chip--danger {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.10);
}

.status-chip--info {
  color: var(--accent);
  background: var(--accent-dim);
}

.status-chip--neutral {
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--bg-surface) 80%, transparent);
}

.mono {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
}

.mono.strong {
  color: var(--text-primary);
  font-weight: 600;
}

.summary-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.summary-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-head.compact {
  align-items: flex-end;
}

.summary-list {
  display: flex;
  flex-direction: column;
}

.summary-row {
  align-items: flex-start;
  padding: 12px 0;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 62%, transparent);
}

.summary-row:first-child {
  padding-top: 0;
  border-top: 0;
}

.empty-state {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.telemetry-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
  padding: 10px 14px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--glass-border) 72%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 78%, transparent);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

@media (max-width: 1260px) {
  .metrics-grid,
  .summary-grid {
    grid-template-columns: 1fr;
  }
}

.dashboard-loading {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 0 2px;
}

.skeleton-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(280px, 0.75fr);
  gap: 18px;
}

.skeleton-col {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.skeleton-row {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 18px;
}

@media (max-width: 1260px) {
  .skeleton-grid,
  .skeleton-row {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 860px) {
  .chart-shell {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .registry-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
