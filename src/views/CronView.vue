<template>
  <div class="cron-view">
    <ViewHero
      kicker="automation"
      title="Cron"
      subtitle="agent schedule telemetry and the push contract behind it."
    >
      <template #meta>
        <span class="view-hero-pill" :class="{ 'is-stale': !cronStore.lastSync }">
          {{ cronStore.lastSync ? `synced ${relativeTime(cronStore.lastSync)}` : "awaiting agent push" }}
        </span>
        <NeonButton size="sm" variant="secondary" ghost :loading="cronStore.loading" @click="cronStore.loadJobs">
          refresh
        </NeonButton>
      </template>
    </ViewHero>

    <div v-if="cronStore.error" class="error-bar">{{ cronStore.error }}</div>

    <section class="summary-grid">
      <GlassCard class="summary-card">
        <p class="summary-label">reporting agents</p>
        <p class="summary-value">{{ agentCount }}</p>
        <p class="summary-copy">agents currently publishing cron snapshots into UMBRA</p>
      </GlassCard>

      <GlassCard class="summary-card">
        <p class="summary-label">scheduled jobs</p>
        <p class="summary-value">{{ cronStore.jobs.length }}</p>
        <p class="summary-copy">{{ enabledCount }} enabled / {{ pausedCount }} paused</p>
      </GlassCard>

      <GlassCard class="summary-card">
        <p class="summary-label">next due</p>
        <p class="summary-value">{{ nextDueLabel }}</p>
        <p class="summary-copy">{{ nextDueJob ? `${nextDueJob.agentName} / ${nextDueJob.job}` : "waiting for nextRun telemetry" }}</p>
      </GlassCard>

      <GlassCard class="summary-card">
        <p class="summary-label">attention</p>
        <p class="summary-value">{{ errorCount }}</p>
        <p class="summary-copy">jobs with lastStatus = error or missed schedule hints</p>
      </GlassCard>
    </section>

    <section class="api-grid">
      <GlassCard class="api-card">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">agent api</p>
            <h2 class="panel-title">POST /api/agents/:id/cron-jobs</h2>
          </div>
          <span class="status-pill">x-agent-token required</span>
        </div>

        <div class="api-meta">
          <div class="meta-row">
            <span class="meta-label">purpose</span>
            <span class="meta-value">agent sends the full cron snapshot it currently owns</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">minimum fields</span>
            <span class="meta-value">agentName, jobs[].job, jobs[].timing, jobs[].recurrence</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">recommended</span>
            <span class="meta-value">nextRun, lastRun, lastStatus, timezone, notes, source</span>
          </div>
        </div>

        <pre class="api-snippet"><code>{{ apiExample }}</code></pre>
      </GlassCard>

      <GlassCard class="api-card">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">design note</p>
            <h2 class="panel-title">why snapshot, not deltas</h2>
          </div>
        </div>
        <ul class="reason-list">
          <li>agents can restart without replaying a mutation log</li>
          <li>umbra always renders the latest truth per agent, not stale partial updates</li>
          <li>frontend merging stays trivial because each push replaces that agent slice</li>
        </ul>
      </GlassCard>
    </section>

    <section v-if="groupedJobs.length" class="agent-groups">
      <GlassCard v-for="group in groupedJobs" :key="group.agentId" class="agent-group">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">agent</p>
            <h3 class="panel-title minor">{{ group.agentName }}</h3>
          </div>
          <div class="group-meta">
            <span class="status-pill">{{ group.jobs.length }} jobs</span>
            <span class="status-pill" :class="{ danger: group.errorCount > 0 }">
              {{ group.errorCount > 0 ? `${group.errorCount} failing` : "healthy" }}
            </span>
          </div>
        </div>

        <div class="job-list">
          <article v-for="job in group.jobs" :key="job.id" class="job-card" :class="{ muted: !job.enabled }">
            <div class="job-head">
              <div>
                <p class="job-name">{{ job.job }}</p>
                <p class="job-subtitle">{{ job.timing }} / {{ job.recurrence }}</p>
              </div>
              <span class="job-status" :class="statusClass(job.lastStatus)">
                {{ job.lastStatus }}
              </span>
            </div>

            <div class="meta-grid">
              <div class="meta-row">
                <span class="meta-label">next</span>
                <span class="meta-value">{{ formatTimestamp(job.nextRun) }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">last</span>
                <span class="meta-value">{{ formatTimestamp(job.lastRun) }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">timezone</span>
                <span class="meta-value">{{ job.timezone || "agent local" }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">source</span>
                <span class="meta-value">{{ job.source || "cron" }}</span>
              </div>
            </div>

            <p v-if="job.notes" class="job-notes">{{ job.notes }}</p>
            <p class="job-foot">
              {{ job.enabled ? "enabled" : "paused" }} / updated {{ relativeTime(job.updatedAt) }}
            </p>
          </article>
        </div>
      </GlassCard>
    </section>

    <GlassCard v-else class="empty-card">
      <p class="empty-title">no agent cron telemetry yet</p>
      <p class="empty-copy">
        point an agent at the UAP endpoint and post a cron snapshot. the tab will populate as soon as the first payload lands.
      </p>
    </GlassCard>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import type { AgentCronJob } from "@/interfaces";
import { useCronStore } from "@/stores/useCronStore";
import ViewHero from "@/components/layout/ViewHero.vue";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";

const cronStore = useCronStore();

const groupedJobs = computed(() => {
  const groups = new Map<string, { agentId: string; agentName: string; jobs: AgentCronJob[]; errorCount: number }>();

  for (const job of cronStore.jobs) {
    const existing = groups.get(job.agentId) ?? {
      agentId: job.agentId,
      agentName: job.agentName,
      jobs: [],
      errorCount: 0,
    };
    existing.jobs.push(job);
    if (job.lastStatus === "error") existing.errorCount += 1;
    groups.set(job.agentId, existing);
  }

  return [...groups.values()]
    .map((group) => ({
      ...group,
      jobs: [...group.jobs].sort((a, b) => sortByNextRun(a, b)),
    }))
    .sort((a, b) => a.agentName.localeCompare(b.agentName));
});

const agentCount = computed(() => groupedJobs.value.length);
const enabledCount = computed(() => cronStore.jobs.filter((job) => job.enabled).length);
const pausedCount = computed(() => cronStore.jobs.filter((job) => !job.enabled).length);
const errorCount = computed(() => cronStore.jobs.filter((job) => job.lastStatus === "error").length);

const nextDueJob = computed(() =>
  [...cronStore.jobs]
    .filter((job) => Boolean(job.nextRun))
    .sort((a, b) => sortByNextRun(a, b))[0]
);

const nextDueLabel = computed(() => formatTimestamp(nextDueJob.value?.nextRun, true));

const apiExample = `curl -X POST "$UMBRA_UAP/api/agents/forge/cron-jobs" \\
  -H "Content-Type: application/json" \\
  -H "X-Agent-Token: $UMBRA_TOKEN" \\
  -d '{
    "agentName": "forge",
    "jobs": [
      {
        "id": "daily-build",
        "job": "daily build",
        "timing": "09:00",
        "recurrence": "weekdays",
        "timezone": "Europe/Berlin",
        "nextRun": "2026-03-21T09:00:00+01:00",
        "lastRun": "2026-03-20T09:00:04+01:00",
        "lastStatus": "ok",
        "notes": "ships internal build + changelog digest",
        "source": "systemd timer"
      }
    ]
  }'`;

function sortByNextRun(a: AgentCronJob, b: AgentCronJob) {
  const aTime = a.nextRun ? new Date(a.nextRun).getTime() : Number.POSITIVE_INFINITY;
  const bTime = b.nextRun ? new Date(b.nextRun).getTime() : Number.POSITIVE_INFINITY;
  return aTime - bTime || a.job.localeCompare(b.job);
}

function formatTimestamp(value?: string | null, compact = false) {
  if (!value) return compact ? "no next run" : "not reported";

  const date = new Date(value);
  const formatter = new Intl.DateTimeFormat("de-DE", {
    day: "2-digit",
    month: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

  return formatter.format(date);
}

function relativeTime(value?: string | null) {
  if (!value) return "never";
  const deltaMs = new Date(value).getTime() - Date.now();
  const minutes = Math.round(deltaMs / 60_000);
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

  if (Math.abs(minutes) < 60) return formatter.format(minutes, "minute");
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) return formatter.format(hours, "hour");
  const days = Math.round(hours / 24);
  return formatter.format(days, "day");
}

function statusClass(status: string) {
  return {
    ok: status === "ok",
    error: status === "error",
    muted: status !== "ok" && status !== "error",
  };
}

onMounted(async () => {
  await cronStore.loadJobs();
  await cronStore.setupLiveUpdates();
});
</script>

<style scoped>
.cron-view {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.page-header,
.panel-head,
.job-head,
.meta-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.header-actions,
.group-meta {
  display: flex;
  align-items: center;
  gap: 10px;
}

.page-kicker,
.page-subtitle,
.panel-kicker,
.summary-label,
.meta-label,
.job-foot,
.job-subtitle {
  margin: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
}

.page-title,
.panel-title,
.summary-value {
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-display);
}

.page-title {
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

.sync-pill,
.status-pill,
.job-status {
  padding: 5px 9px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 82%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
}

.status-pill.danger,
.job-status.error {
  color: var(--accent-error);
  border-color: rgba(239, 68, 68, 0.32);
}

.job-status.ok {
  color: var(--accent-success);
  border-color: rgba(34, 197, 94, 0.24);
}

.job-status.muted {
  color: var(--text-muted);
}

.error-bar {
  padding: 12px 14px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.28);
  border-radius: var(--radius-lg);
  color: var(--accent-error);
  font-family: var(--font-mono);
  font-size: 12px;
}

.summary-grid,
.api-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.api-grid {
  grid-template-columns: minmax(0, 1.3fr) minmax(0, 0.9fr);
}

.summary-card,
.agent-group,
.api-card,
.empty-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.summary-label,
.panel-kicker {
  text-transform: uppercase;
}

.summary-value {
  font-size: 34px;
  font-weight: 800;
}

.summary-copy,
.empty-copy,
.reason-list,
.meta-value,
.job-notes {
  margin: 0;
  color: var(--text-secondary);
  line-height: 1.6;
  font-size: 12px;
}

.panel-title {
  font-size: 18px;
  font-weight: 800;
  letter-spacing: 0.05em;
}

.panel-title.minor {
  font-size: 20px;
}

.api-meta,
.meta-grid,
.job-list,
.agent-groups {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.meta-row {
  align-items: baseline;
}

.meta-label {
  min-width: 84px;
  text-transform: uppercase;
}

.meta-value {
  flex: 1;
}

.api-snippet {
  margin: 0;
  padding: 12px;
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--bg-elevated) 70%, transparent);
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.6;
  overflow-x: auto;
}

.reason-list {
  padding-left: 18px;
}

.job-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  background: color-mix(in srgb, var(--bg-elevated) 34%, transparent);
}

.job-card.muted {
  opacity: 0.72;
}

.job-name,
.empty-title {
  margin: 0;
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 700;
}

.job-notes {
  padding: 10px 12px;
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

:global([data-theme="light"]) .sync-pill,
:global([data-theme="light"]) .status-pill,
:global([data-theme="light"]) .job-status {
  background: rgba(255, 255, 255, 0.88);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .job-card,
:global([data-theme="light"]) .api-snippet,
:global([data-theme="light"]) .job-notes {
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .job-card,
:global([data-theme="light"]) .job-notes {
  background: rgba(248, 250, 252, 0.9);
}

:global([data-theme="light"]) .api-snippet {
  background: rgba(241, 245, 249, 0.92);
}

@media (max-width: 1180px) {
  .summary-grid,
  .api-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .page-header,
  .header-actions,
  .panel-head,
  .job-head,
  .meta-row {
    flex-direction: column;
  }

  .summary-grid,
  .api-grid {
    grid-template-columns: 1fr;
  }
}
</style>
