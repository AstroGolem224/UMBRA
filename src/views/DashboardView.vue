<template>
  <div class="dashboard-view">
    <ViewHero
      kicker="mission control"
      title="Dashboard"
      subtitle="calm overview for routing, deadlines and live system state."
    >
      <template #meta>
        <div class="view-hero-pill" :class="{ 'is-stale': !taskStore.lastSync }">
          {{ taskStore.lastSync ? `pm ${formatRelativeTime(taskStore.lastSync)}` : "pm sync pending" }}
        </div>
        <div class="view-hero-pill" :class="{ 'is-stale': !cronStore.lastSync }">
          {{ cronStore.lastSync ? `cron ${formatRelativeTime(cronStore.lastSync)}` : "cron awaiting agent push" }}
        </div>
      </template>
    </ViewHero>

    <section class="layout-grid">
      <GlassCard class="panel slice-panel">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">dashboard slice</p>
            <h2 class="panel-title">Umbra Slice</h2>
            <p class="panel-copy">the main working slice: what matters now, where it should go and what needs attention.</p>
          </div>
        </div>

        <div class="focus-card">
          <div class="focus-head">
            <div>
              <p class="focus-kicker">next best move / assignment broker</p>
              <h3 class="focus-title">{{ topSuggestion ? topSuggestion.task.title : "waiting for enough signal" }}</h3>
            </div>
            <span v-if="topSuggestion" class="focus-score">{{ topSuggestion.score }}</span>
          </div>
          <p v-if="topSuggestion" class="focus-copy">
            route to <strong>{{ topSuggestion.agent.name }}</strong> / {{ topSuggestion.agent.role || "generalist" }}
          </p>
          <p v-else class="focus-copy">
            once live agents and open PM tasks are both present, the broker will recommend a route here.
          </p>
          <div v-if="topSuggestion" class="reason-row">
            <span v-for="reason in topSuggestion.reasons" :key="reason" class="reason-pill">{{ reason }}</span>
          </div>
        </div>

        <div class="slice-columns">
          <div class="slice-block">
            <div class="block-head">
              <p class="block-kicker">attention rail</p>
              <span class="status-pill" :class="{ danger: attentionItems.length > 0 }">
                {{ attentionItems.length ? `${attentionItems.length} signals` : "all clear" }}
              </span>
            </div>

            <div v-if="attentionItems.length" class="list-rows">
              <article v-for="item in attentionItems" :key="item.id" class="list-row">
                <div>
                  <p class="row-title">{{ item.title }}</p>
                  <p class="row-subtitle">{{ item.detail }}</p>
                </div>
                <span class="row-badge" :class="item.level">{{ item.kind }}</span>
              </article>
            </div>
            <div v-else class="empty-state">no overdue tasks, failing cron jobs or stale agents right now</div>
          </div>

          <div class="slice-block">
            <div class="block-head">
              <p class="block-kicker">system snapshot</p>
              <span class="status-pill">live state</span>
            </div>

            <div class="snapshot-list">
              <article class="snapshot-row">
                <span class="snapshot-label">automation pulse</span>
                <div class="snapshot-copy">
                  <strong>{{ enabledCronJobs }} enabled jobs</strong>
                  <small>{{ failingCronJobs.length }} errors / {{ cronAgentCount }} agents</small>
                </div>
              </article>
              <article class="snapshot-row">
                <span class="snapshot-label">next cron</span>
                <div class="snapshot-copy">
                  <strong>{{ nextCronJob ? nextCronJob.job : "not reported" }}</strong>
                  <small>{{ nextCronJob ? formatDeadline(nextCronJob.nextRun) : "waiting" }}</small>
                </div>
              </article>
              <article class="snapshot-row">
                <span class="snapshot-label">next deadline</span>
                <div class="snapshot-copy">
                  <strong>{{ nextDeadline ? nextDeadline.title : "no dated tasks" }}</strong>
                  <small>{{ nextDeadline ? formatDeadline(nextDeadline.deadline ?? nextDeadline.nextDueDate) : "clear" }}</small>
                </div>
              </article>
              <article class="snapshot-row">
                <span class="snapshot-label">latest note</span>
                <div class="snapshot-copy">
                  <strong>{{ latestNote ? latestNote.title : "no note sync data yet" }}</strong>
                  <small>{{ latestNote ? formatRelativeTime(latestNote.updatedAt) : "waiting" }}</small>
                </div>
              </article>
            </div>
          </div>
        </div>

        <div class="roster-strip">
          <div class="roster-head">
            <p class="block-kicker">live roster</p>
            <span class="status-pill">{{ onlineAgents }} online</span>
          </div>
          <div v-if="agentStore.agents.length" class="roster-list">
            <article v-for="agent in sortedAgents.slice(0, 4)" :key="agent.id" class="roster-card">
              <div class="roster-line">
                <StatusBadge :status="agent.status" />
                <strong>{{ agent.name }}</strong>
              </div>
              <p>{{ agent.role || "no role set" }}</p>
            </article>
          </div>
          <div v-else class="empty-state">no agent telemetry yet</div>
        </div>
      </GlassCard>

      <div class="stats-grid">
        <GlassCard v-for="card in summaryCards" :key="card.label" class="panel stat-card">
          <div class="stat-head">
            <div>
              <p class="stat-label">{{ card.label }}</p>
              <p class="stat-copy">{{ card.copy }}</p>
            </div>
          </div>
          <p class="stat-value">{{ card.value }}</p>
        </GlassCard>
      </div>

      <GlassCard class="panel list-panel">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">timeline</p>
            <h3 class="panel-title minor">Upcoming Deadlines</h3>
          </div>
        </div>

        <div v-if="upcomingDeadlines.length" class="list-rows">
          <article v-for="task in upcomingDeadlines" :key="task.id" class="list-row">
            <div>
              <p class="row-title">{{ task.title }}</p>
              <p class="row-subtitle">{{ task.project }} / {{ formatDeadline(task.deadline ?? task.nextDueDate) }}</p>
            </div>
            <span class="row-badge info">{{ task.priority }}</span>
          </article>
        </div>
        <div v-else class="empty-state">no due dates visible in synced PM data</div>
      </GlassCard>

      <GlassCard class="panel list-panel">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">telemetry</p>
            <h3 class="panel-title minor">Recent Activity</h3>
          </div>
        </div>

        <div v-if="recentActivity.length" class="list-rows">
          <article v-for="activity in recentActivity" :key="activity.id" class="list-row">
            <div>
              <p class="row-title">{{ activity.title }}</p>
              <p class="row-subtitle">{{ activity.project }} / {{ activity.kind }}</p>
            </div>
            <span class="row-badge info">{{ formatRelativeTime(activity.timestamp) }}</span>
          </article>
        </div>
        <div v-else class="empty-state">no recent task activity yet</div>
      </GlassCard>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import ViewHero from "@/components/layout/ViewHero.vue";
import GlassCard from "@/components/ui/GlassCard.vue";
import StatusBadge from "@/components/ui/StatusBadge.vue";
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

const agentStore = useAgentStore();
const cronStore = useCronStore();
const notesStore = useNotesStore();
const taskStore = useTaskStore();

const priorityWeight: Record<Task["priority"], number> = {
  urgent: 5,
  critical: 4,
  high: 3,
  medium: 2,
  low: 1,
};

const onlineAgents = computed(() => agentStore.agents.filter((agent) => ["online", "working", "idle"].includes(agent.status)).length);
const staleAgents = computed(() =>
  agentStore.agents.filter((agent) => Date.now() - new Date(agent.lastSeen).getTime() > 30 * 60_000)
);
const overdueTasks = computed(() =>
  taskStore.tasks.filter((task) => {
    const due = task.deadline ?? task.nextDueDate;
    return Boolean(due) && !["done", "cancelled"].includes(task.status) && new Date(due!).getTime() < Date.now();
  })
);
const failingCronJobs = computed(() => cronStore.jobs.filter((job) => job.lastStatus === "error"));
const cronAgentCount = computed(() => new Set(cronStore.jobs.map((job) => job.agentId)).size);
const enabledCronJobs = computed(() => cronStore.jobs.filter((job) => job.enabled).length);

const suggestions = computed(() => buildAssignmentSuggestions(taskStore.tasks, agentStore.agents, 4));
const topSuggestion = computed(() => suggestions.value[0]);

const nextCronJob = computed(() =>
  [...cronStore.jobs]
    .filter((job) => Boolean(job.nextRun))
    .sort((a, b) => new Date(a.nextRun ?? "").getTime() - new Date(b.nextRun ?? "").getTime())[0]
);

const upcomingDeadlines = computed(() =>
  [...taskStore.tasks]
    .filter((task) => Boolean(task.deadline ?? task.nextDueDate))
    .sort((a, b) => {
      const aTime = new Date(a.deadline ?? a.nextDueDate ?? "").getTime();
      const bTime = new Date(b.deadline ?? b.nextDueDate ?? "").getTime();
      return aTime - bTime || priorityWeight[b.priority] - priorityWeight[a.priority];
    })
    .slice(0, 5)
);

const nextDeadline = computed(() => upcomingDeadlines.value[0]);
const recentNotes = computed(() =>
  [...notesStore.notes]
    .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
    .slice(0, 5)
);
const latestNote = computed(() => recentNotes.value[0]);

const sortedAgents = computed(() =>
  [...agentStore.agents].sort((a, b) => {
    const rank = (status: string) => ["working", "idle", "online", "offline", "error"].indexOf(status);
    return rank(a.status) - rank(b.status) || a.name.localeCompare(b.name);
  })
);

const attentionItems = computed<AttentionItem[]>(() => {
  const items: AttentionItem[] = [
    ...failingCronJobs.value.slice(0, 3).map((job) => ({
      id: `cron-${job.id}`,
      kind: "cron",
      title: job.job,
      detail: `${job.agentName} / ${job.notes || job.lastStatus}`,
      level: "danger" as const,
    })),
    ...overdueTasks.value.slice(0, 3).map((task) => ({
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

const summaryCards = computed(() => [
  {
    label: "Total Tasks",
    value: taskStore.tasks.length,
    copy: "all lanes",
  },
  {
    label: "In Progress",
    value: taskStore.tasks.filter((task) => task.status === "in-progress").length,
    copy: "active throughput",
  },
  {
    label: "Completed",
    value: taskStore.tasks.filter((task) => task.status === "done").length,
    copy: "done state",
  },
  {
    label: "Overdue",
    value: overdueTasks.value.length,
    copy: "requires action",
  },
]);

const recentActivity = computed<ActivityItem[]>(() => {
  const activity = taskStore.tasks.flatMap((task) => {
    const items: ActivityItem[] = [];

    if (task.createdAt) {
      items.push({
        id: `${task.id}-created`,
        title: task.title,
        project: task.project,
        kind: "created",
        timestamp: task.createdAt,
      });
    }

    if (task.updatedAt && task.createdAt && task.updatedAt !== task.createdAt) {
      items.push({
        id: `${task.id}-updated`,
        title: task.title,
        project: task.project,
        kind: task.status === "done" ? "completed" : "updated",
        timestamp: task.updatedAt,
      });
    }

    for (const comment of task.comments ?? []) {
      items.push({
        id: comment.id,
        title: task.title,
        project: task.project,
        kind: "commented",
        timestamp: comment.createdAt,
      });
    }

    return items;
  });

  return activity
    .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
    .slice(0, 6);
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
});
</script>

<style scoped>
.dashboard-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  --hero-border: color-mix(in srgb, var(--glass-border) 90%, transparent);
  --hero-bg:
    radial-gradient(circle at top right, color-mix(in srgb, var(--accent) 10%, transparent), transparent 36%),
    linear-gradient(135deg, color-mix(in srgb, var(--bg-secondary) 93%, transparent), color-mix(in srgb, var(--bg-primary) 96%, transparent));
  --hero-shadow: 0 18px 36px rgba(0, 0, 0, 0.12);
  --slice-surface: color-mix(in srgb, var(--bg-elevated) 18%, transparent);
  --slice-border: color-mix(in srgb, var(--glass-border) 78%, transparent);
  --line-divider: color-mix(in srgb, var(--glass-border) 68%, transparent);
}

.hero,
.panel-head,
.block-head,
.list-row,
.snapshot-row,
.stat-head,
.roster-line {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.hero {
  align-items: center;
  padding: 22px 24px;
  border-radius: var(--radius-2xl);
  border: 1px solid var(--hero-border);
  background: var(--hero-bg);
  box-shadow: var(--hero-shadow);
}

.hero-copy {
  max-width: 620px;
}

.hero-kicker,
.panel-kicker,
.block-kicker,
.stat-label,
.snapshot-label,
.focus-kicker {
  margin: 0;
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--text-muted) 86%, var(--text-secondary));
}

.hero-title,
.panel-title,
.focus-title,
.stat-value {
  margin: 0;
  font-family: var(--font-display);
  color: var(--text-primary);
}

.hero-title {
  font-size: 40px;
  font-weight: 800;
  letter-spacing: 0.08em;
  line-height: 0.95;
}

.hero-subtitle,
.panel-copy,
.focus-copy,
.row-subtitle,
.stat-copy,
.empty-state,
.snapshot-copy small,
.roster-card p {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.hero-meta {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.theme-pill,
.status-pill,
.reason-pill,
.row-badge,
.focus-score {
  padding: 6px 10px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 84%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
}

.theme-pill.stale,
.status-pill.danger,
.row-badge.danger {
  color: var(--accent-error);
  border-color: rgba(239, 68, 68, 0.28);
}

.row-badge.warn {
  color: var(--neon-amber);
  border-color: rgba(245, 158, 11, 0.28);
}

.row-badge.info,
.reason-pill,
.focus-score {
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 26%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.layout-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.45fr) minmax(0, 1fr);
  gap: 14px;
}

.panel {
  min-width: 0;
}

.slice-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.panel-title {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: 0.04em;
}

.panel-title.minor {
  font-size: 20px;
}

.focus-card {
  padding: 16px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--accent) 18%, transparent);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--accent) 7%, transparent), transparent 76%),
    var(--slice-surface);
}

.focus-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}

.focus-title {
  font-size: 24px;
  font-weight: 800;
  line-height: 1.04;
}

.focus-copy strong {
  color: var(--text-primary);
}

.reason-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.slice-columns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.slice-block {
  padding: 15px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--slice-border);
  background: var(--slice-surface);
}

.list-rows,
.snapshot-list {
  display: flex;
  flex-direction: column;
}

.list-row,
.snapshot-row {
  align-items: flex-start;
  padding: 12px 0;
  border-top: 1px solid var(--line-divider);
}

.list-row:first-of-type,
.snapshot-row:first-of-type {
  padding-top: 0;
  border-top: 0;
}

.row-title {
  margin: 0;
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.45;
}

.snapshot-copy {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 3px;
  text-align: right;
}

.snapshot-copy strong {
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 600;
}

.roster-strip {
  padding-top: 4px;
}

.roster-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.roster-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.roster-card {
  padding: 12px;
  border-radius: var(--radius-md);
  border: 1px solid var(--slice-border);
  background: var(--slice-surface);
}

.roster-line {
  justify-content: flex-start;
  align-items: center;
  margin-bottom: 6px;
}

.roster-card strong {
  color: var(--text-primary);
  font-size: 13px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.stat-card {
  min-height: 148px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.stat-value {
  font-size: 36px;
  font-weight: 800;
}

.list-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

@media (max-width: 1220px) {
  .layout-grid {
    grid-template-columns: 1fr;
  }

  .stats-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

@media (max-width: 860px) {
  .hero {
    flex-direction: column;
    align-items: flex-start;
  }

  .slice-columns,
  .roster-list,
  .stats-grid {
    grid-template-columns: 1fr;
  }

  .hero-meta {
    justify-content: flex-start;
  }
}
</style>

<style>
[data-theme="light"] .dashboard-view .hero {
  border-color: rgba(8, 145, 178, 0.14);
  background:
    radial-gradient(circle at top right, rgba(8, 145, 178, 0.12), transparent 34%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.98), rgba(236, 243, 248, 0.94));
  box-shadow: 0 18px 32px rgba(15, 23, 42, 0.06);
}

[data-theme="light"] .dashboard-view .focus-card,
[data-theme="light"] .dashboard-view .slice-block,
[data-theme="light"] .dashboard-view .roster-card,
[data-theme="light"] .dashboard-view .theme-pill,
[data-theme="light"] .dashboard-view .status-pill {
  background: rgba(255, 255, 255, 0.8);
}

[data-theme="light"] .dashboard-view .list-row,
[data-theme="light"] .dashboard-view .snapshot-row {
  border-top-color: rgba(15, 23, 42, 0.08);
}
</style>
