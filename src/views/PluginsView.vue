<template>
  <div class="plugins-view">
    <header class="page-header">
      <h1 class="page-title">INTEGRATIONS</h1>
      <NeonButton size="sm" variant="secondary" ghost :loading="loading" @click="refresh">
        ↺ REFRESH
      </NeonButton>
    </header>

    <!-- Obsidian -->
    <GlassCard class="plugin-card">
      <div class="plugin-header">
        <span class="plugin-icon">◈</span>
        <div class="plugin-meta">
          <h3 class="plugin-name">OBSIDIAN</h3>
          <p class="plugin-desc">Local vault — note access via filesystem</p>
        </div>
        <span class="status-dot" :class="obsidian?.connected ? 'ok' : 'err'" />
        <span class="status-label" :class="obsidian?.connected ? 'ok' : 'err'">
          {{ obsidian?.connected ? 'CONNECTED' : obsidianError ? 'ERROR' : 'SCANNING…' }}
        </span>
      </div>

      <div v-if="obsidian?.connected" class="plugin-body">
        <div class="stat-row">
          <span class="stat-label">Vault</span>
          <span class="stat-value mono">{{ shortPath(obsidian.vaultPath) }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Notes</span>
          <span class="stat-value">{{ obsidian.totalNotes.toLocaleString() }}</span>
        </div>
        <div v-if="obsidian.recentNotes.length" class="recent-list">
          <span class="stat-label">Recent</span>
          <ul>
            <li v-for="n in obsidian.recentNotes" :key="n.name" class="recent-item">
              <span class="recent-name">{{ n.name }}</span>
              <span class="recent-time">{{ relTime(n.modified) }}</span>
            </li>
          </ul>
        </div>
      </div>
      <p v-else-if="obsidianError" class="plugin-error">{{ obsidianError }}</p>
    </GlassCard>

    <!-- TM-lite -->
    <GlassCard class="plugin-card">
      <div class="plugin-header">
        <span class="plugin-icon">▣</span>
        <div class="plugin-meta">
          <h3 class="plugin-name">TM-LITE</h3>
          <p class="plugin-desc">Task management — reads from Obsidian vault/Tasks/</p>
        </div>
        <span class="status-dot" :class="tmTasks !== null ? 'ok' : 'err'" />
        <span class="status-label" :class="tmTasks !== null ? 'ok' : 'err'">
          {{ tmTasks !== null ? 'CONNECTED' : tmError ? 'ERROR' : 'LOADING…' }}
        </span>
      </div>

      <div v-if="tmTasks !== null" class="plugin-body">
        <div class="stat-row">
          <span class="stat-label">Open tasks</span>
          <span class="stat-value">{{ tmTasks.length }}</span>
        </div>
        <ul v-if="tmTasks.length" class="task-list">
          <li v-for="t in tmTasks.slice(0, 6)" :key="t.id" class="task-item">
            <span class="task-priority" :class="t.priority">●</span>
            <span class="task-title">{{ t.title }}</span>
            <span class="task-project">{{ t.project }}</span>
          </li>
        </ul>
        <p v-else class="plugin-empty">No open tasks.</p>
      </div>
      <p v-else-if="tmError" class="plugin-error">{{ tmError }}</p>
    </GlassCard>

    <!-- GitHub -->
    <GlassCard class="plugin-card">
      <div class="plugin-header">
        <span class="plugin-icon">◉</span>
        <div class="plugin-meta">
          <h3 class="plugin-name">GITHUB</h3>
          <p class="plugin-desc">Repo stats — api.github.com via PAT</p>
        </div>
        <span class="status-dot" :class="githubStore.repos.length ? 'ok' : 'err'" />
        <span class="status-label" :class="githubStore.repos.length ? 'ok' : 'err'">
          {{ githubStore.repos.length ? 'CONNECTED' : githubStore.loading ? 'LOADING…' : 'NO DATA' }}
        </span>
      </div>

      <div v-if="githubStore.repos.length" class="plugin-body">
        <div class="stat-row">
          <span class="stat-label">Tracked repos</span>
          <span class="stat-value">{{ githubStore.repos.length }}</span>
        </div>
        <ul class="task-list">
          <li v-for="r in githubStore.repos" :key="r.id" class="task-item">
            <span class="task-priority medium">◈</span>
            <span class="task-title">{{ r.name }}</span>
            <span class="task-project">{{ r.openIssues }} issues</span>
          </li>
        </ul>
      </div>
      <p v-else-if="!githubStore.loading" class="plugin-empty">
        No repos loaded. Set a GitHub PAT in Settings → GITHUB.
      </p>
    </GlassCard>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import { useGithubStore } from "@/stores/useGithubStore";

interface RecentNote { name: string; modified: string }
interface ObsidianStats { connected: boolean; totalNotes: number; recentNotes: RecentNote[]; vaultPath: string }
interface TmTask { id: string; title: string; status: string; project: string; priority: string }

const githubStore = useGithubStore();

const loading = ref(false);
const obsidian = ref<ObsidianStats | null>(null);
const obsidianError = ref<string | null>(null);
const tmTasks = ref<TmTask[] | null>(null);
const tmError = ref<string | null>(null);

async function loadObsidian() {
  try {
    obsidian.value = await invoke<ObsidianStats>("get_obsidian_stats");
    obsidianError.value = null;
  } catch (e) {
    obsidianError.value = String(e);
  }
}

async function loadTm() {
  try {
    tmTasks.value = await invoke<TmTask[]>("get_tmlite_tasks");
    tmError.value = null;
  } catch (e) {
    tmTasks.value = [];
    tmError.value = String(e);
  }
}

async function refresh() {
  loading.value = true;
  await Promise.all([loadObsidian(), loadTm(), githubStore.loadRepos()]);
  loading.value = false;
}

function shortPath(p: string) {
  const parts = p.replace(/\\/g, "/").split("/");
  return parts.length > 3 ? "…/" + parts.slice(-2).join("/") : p;
}

function relTime(iso: string) {
  const diff = Date.now() - new Date(iso).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

onMounted(refresh);
</script>

<style scoped>
.plugins-view {
  max-width: 720px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.page-title {
  font-family: var(--font-display), "Iceland", monospace;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.2em;
  color: var(--text-primary);
  margin: 0;
}

.plugin-card {
  padding: 18px 20px;
}

.plugin-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}

.plugin-icon {
  font-size: 20px;
  color: var(--accent);
  width: 24px;
  text-align: center;
  flex-shrink: 0;
}

.plugin-meta {
  flex: 1;
}

.plugin-name {
  font-family: var(--font-display), "Iceland", monospace;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.15em;
  color: var(--text-primary);
  margin: 0 0 2px;
}

.plugin-desc {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.ok { background: var(--accent-success); box-shadow: 0 0 6px var(--accent-success); }
.status-dot.err { background: var(--accent-error); }

.status-label {
  font-size: 10px;
  letter-spacing: 0.12em;
  font-family: var(--font-display), "Iceland", monospace;
}

.status-label.ok { color: var(--accent-success); }
.status-label.err { color: var(--accent-error); }

.plugin-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stat-row {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.stat-label {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 0.1em;
  min-width: 80px;
  flex-shrink: 0;
}

.stat-value {
  font-size: 13px;
  color: var(--text-primary);
}

.stat-value.mono {
  font-family: var(--font-mono), monospace;
  font-size: 11px;
  color: var(--text-secondary);
}

.recent-list {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.recent-list ul {
  list-style: none;
  margin: 0;
  padding: 0;
  flex: 1;
}

.recent-item {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
  border-bottom: 1px solid var(--glass-border);
  font-size: 11px;
}

.recent-name {
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 280px;
}

.recent-time {
  color: var(--text-muted);
  flex-shrink: 0;
  margin-left: 12px;
}

.task-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 0;
  border-bottom: 1px solid var(--glass-border);
  font-size: 11px;
}

.task-item:last-child { border-bottom: none; }

.task-priority {
  font-size: 8px;
  flex-shrink: 0;
}

.task-priority.critical { color: #ff3333; }
.task-priority.high     { color: var(--accent); }
.task-priority.medium   { color: var(--accent-secondary); }
.task-priority.low      { color: var(--text-muted); }

.task-title {
  flex: 1;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-project {
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.plugin-error {
  font-size: 11px;
  color: var(--accent-error);
  font-family: var(--font-mono), monospace;
  margin: 0;
}

.plugin-empty {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
  font-style: italic;
}
</style>
