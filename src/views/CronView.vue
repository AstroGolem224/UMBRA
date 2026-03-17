<template>
  <div class="cron-view">
    <header class="page-header">
      <div class="header-left">
        <h1 class="page-title">CRON</h1>
        <span class="page-subtitle">// scheduled jobs</span>
      </div>
      <NeonButton size="sm" @click="showForm = true">+ ADD JOB</NeonButton>
    </header>

    <div v-if="cronStore.error" class="error-bar">{{ cronStore.error }}</div>

    <!-- Job Table -->
    <div class="jobs-section">
      <div v-if="cronStore.jobs.length === 0 && !cronStore.loading" class="empty-state">
        <span class="empty-icon">⏱</span>
        <span>no scheduled jobs</span>
        <NeonButton size="sm" ghost @click="showForm = true">create first job</NeonButton>
      </div>

      <div v-else class="job-list">
        <div
          v-for="job in cronStore.jobs"
          :key="job.id"
          class="job-row glass-panel"
          :class="{ disabled: !job.enabled }"
        >
          <div class="job-main">
            <div class="job-toggle">
              <button
                class="toggle-btn"
                :class="{ on: job.enabled }"
                :title="job.enabled ? 'Disable' : 'Enable'"
                @click="cronStore.toggleJob(job.id)"
              >
                <span class="toggle-dot" />
              </button>
            </div>

            <div class="job-info">
              <div class="job-name">{{ job.name }}</div>
              <div class="job-schedule">
                <span class="mono-label">{{ job.schedule }}</span>
              </div>
              <div class="job-command">
                <span class="mono-muted">{{ truncate(job.command, 60) }}</span>
              </div>
            </div>

            <div class="job-status-col">
              <span class="status-badge" :class="job.lastStatus">
                {{ job.lastStatus }}
              </span>
              <span v-if="job.lastRun" class="last-run">{{ relativeTime(job.lastRun) }}</span>
            </div>
          </div>

          <div class="job-actions">
            <NeonButton
              size="sm"
              ghost
              :loading="cronStore.runningId === job.id"
              @click="handleRunNow(job.id)"
            >
              ▶ RUN
            </NeonButton>
            <button class="icon-btn output-btn" title="View output" @click="viewOutput(job)">
              ≡
            </button>
            <button class="icon-btn delete-btn" title="Delete" @click="confirmDelete(job.id)">
              ✕
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Output Panel -->
    <div v-if="selectedOutput" class="output-panel glass-panel">
      <div class="output-header">
        <span class="output-title">OUTPUT — {{ selectedJobName }}</span>
        <button class="icon-btn" @click="selectedOutput = null; selectedJobName = ''">✕</button>
      </div>
      <pre class="output-body">{{ selectedOutput }}</pre>
    </div>

    <!-- Add Job Form -->
    <div v-if="showForm" class="form-overlay" @click.self="showForm = false">
      <GlassCard class="form-panel">
        <h3 class="form-title">NEW JOB</h3>

        <div class="form-field">
          <label class="field-label">NAME</label>
          <input v-model="form.name" class="field-input" placeholder="e.g. Daily Backup" />
        </div>

        <div class="form-field">
          <label class="field-label">SCHEDULE</label>
          <input v-model="form.schedule" class="field-input" placeholder='e.g. every 1 hours' />
          <span class="field-hint">tokio-cron syntax: "every 30 seconds", "0 */6 * * * *"</span>
        </div>

        <div class="form-field">
          <label class="field-label">COMMAND</label>
          <input v-model="form.command" class="field-input" placeholder="e.g. git -C C:\repo pull" />
        </div>

        <div class="form-actions">
          <NeonButton ghost @click="showForm = false">CANCEL</NeonButton>
          <NeonButton :loading="saving" @click="handleCreate">CREATE</NeonButton>
        </div>
      </GlassCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useCronStore } from "@/stores/useCronStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import type { CronJob } from "@/interfaces";

const cronStore = useCronStore();

const showForm = ref(false);
const saving = ref(false);
const selectedOutput = ref<string | null>(null);
const selectedJobName = ref("");

const form = ref({ name: "", schedule: "every 30 minutes", command: "" });

async function handleCreate() {
  if (!form.value.name.trim() || !form.value.command.trim()) return;
  saving.value = true;
  try {
    await cronStore.createJob(form.value.name, form.value.schedule, form.value.command);
    form.value = { name: "", schedule: "every 30 minutes", command: "" };
    showForm.value = false;
  } finally {
    saving.value = false;
  }
}

async function handleRunNow(id: string) {
  await cronStore.runNow(id);
  const job = cronStore.jobs.find((j) => j.id === id);
  if (job?.lastOutput) {
    selectedOutput.value = job.lastOutput;
    selectedJobName.value = job.name;
  }
}

function viewOutput(job: CronJob) {
  selectedOutput.value = job.lastOutput ?? "— no output —";
  selectedJobName.value = job.name;
}

function confirmDelete(id: string) {
  if (confirm("Delete this job?")) cronStore.deleteJob(id);
}

function truncate(s: string, max: number) {
  return s.length > max ? s.slice(0, max) + "…" : s;
}

function relativeTime(iso: string) {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

onMounted(() => {
  cronStore.loadJobs();
  cronStore.setupLiveUpdates();
});
</script>

<style scoped>
.cron-view {
  max-width: 900px;
  position: relative;
}

/* ─── Header ──────────────────────────────── */
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
}

/* ─── Error ───────────────────────────────── */
.error-bar {
  margin-bottom: 16px;
  padding: 10px 14px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid var(--accent-error);
  border-radius: 6px;
  color: var(--accent-error);
  font-family: var(--font-mono);
  font-size: 12px;
}

/* ─── Empty State ─────────────────────────── */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 24px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 12px;
}

.empty-icon {
  font-size: 32px;
  opacity: 0.4;
}

/* ─── Job List ────────────────────────────── */
.job-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.job-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 16px;
  border-radius: 8px;
  transition: opacity 0.2s;
}

.job-row.disabled {
  opacity: 0.45;
}

.job-main {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

/* Toggle */
.toggle-btn {
  width: 32px;
  height: 18px;
  border-radius: 9px;
  border: 1px solid var(--glass-border);
  background: var(--bg-elevated);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.toggle-btn.on {
  background: var(--accent-dim);
  border-color: var(--accent);
}

.toggle-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--text-muted);
  transition: all 0.2s;
}

.toggle-btn.on .toggle-dot {
  background: var(--accent);
  transform: translateX(14px);
  box-shadow: 0 0 6px var(--accent-dim);
}

/* Job info */
.job-info {
  flex: 1;
  min-width: 0;
}

.job-name {
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.job-schedule {
  margin-bottom: 2px;
}

.mono-label {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--accent);
}

.mono-muted {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

/* Status col */
.job-status-col {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.status-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 3px;
  border: 1px solid;
}

.status-badge.ok {
  color: var(--neon-green);
  border-color: rgba(76, 175, 80, 0.3);
  background: rgba(76, 175, 80, 0.08);
}

.status-badge.error {
  color: var(--accent-error);
  border-color: rgba(239, 68, 68, 0.3);
  background: rgba(239, 68, 68, 0.08);
}

.status-badge.pending {
  color: var(--text-muted);
  border-color: var(--glass-border);
  background: transparent;
}

.last-run {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
}

/* Actions */
.job-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  border: 1px solid var(--glass-border);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.delete-btn:hover {
  border-color: var(--accent-error);
  color: var(--accent-error);
}

/* ─── Output Panel ────────────────────────── */
.output-panel {
  margin-top: 16px;
  padding: 0;
  overflow: hidden;
}

.output-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--glass-border);
}

.output-title {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.1em;
  color: var(--accent);
}

.output-body {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  padding: 12px 14px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
  margin: 0;
}

/* ─── Form Overlay ────────────────────────── */
.form-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: rgba(8, 13, 19, 0.75);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.form-panel {
  width: 460px;
  max-width: 90vw;
}

.form-title {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-primary);
  margin-bottom: 20px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 14px;
}

.field-label {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.12em;
  color: var(--text-muted);
  text-transform: uppercase;
}

.field-input {
  background: var(--bg-elevated);
  border: 1px solid var(--glass-border);
  border-radius: 6px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  padding: 8px 10px;
  outline: none;
  transition: border-color 0.15s;
}

.field-input:focus {
  border-color: var(--glass-border-hot);
}

.field-hint {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
