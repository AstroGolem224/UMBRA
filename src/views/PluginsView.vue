<template>
  <div class="plugins-view">
    <ViewHero
      kicker="modules"
      title="Integrations"
      subtitle="connected systems, broker signals and the next plugin layer for UMBRA."
    >
      <template #meta>
        <span class="view-hero-pill">{{ brokerSuggestions.length ? `${brokerSuggestions.length} broker routes` : "broker waiting" }}</span>
        <span class="view-hero-pill" :class="{ 'is-stale': loading }">
          {{ loading ? "refreshing modules" : "module scan ready" }}
        </span>
        <NeonButton size="sm" variant="secondary" ghost :loading="loading" @click="refresh">
          refresh
        </NeonButton>
      </template>
    </ViewHero>

    <section class="plugin-grid">
      <GlassCard class="plugin-card">
        <div class="plugin-header">
          <span class="plugin-icon">OB</span>
          <div class="plugin-meta">
            <h3 class="plugin-name">OBSIDIAN</h3>
            <p class="plugin-desc">local vault access via filesystem</p>
          </div>
          <span class="status-pill" :class="obsidian?.connected ? 'ok' : 'err'">
            {{ obsidian?.connected ? "CONNECTED" : obsidianError ? "ERROR" : "SCANNING" }}
          </span>
        </div>

        <div v-if="obsidian?.connected" class="plugin-body">
          <div class="stat-row">
            <span class="stat-label">vault</span>
            <span class="stat-value mono">{{ shortPath(obsidian.vaultPath) }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">notes</span>
            <span class="stat-value">{{ obsidian.totalNotes.toLocaleString() }}</span>
          </div>
          <div v-if="obsidian.recentNotes.length" class="recent-list">
            <span class="stat-label">recent</span>
            <ul>
              <li v-for="note in obsidian.recentNotes" :key="note.name" class="recent-item">
                <span class="recent-name">{{ note.name }}</span>
                <span class="recent-time">{{ relTime(note.modified) }}</span>
              </li>
            </ul>
          </div>
        </div>
        <p v-else-if="obsidianError" class="plugin-error">{{ obsidianError }}</p>
      </GlassCard>

      <GlassCard class="plugin-card broker-card">
        <div class="plugin-header">
          <span class="plugin-icon">AB</span>
          <div class="plugin-meta">
            <h3 class="plugin-name">ASSIGNMENT BROKER</h3>
            <p class="plugin-desc">recommends who should pick up which PM task next</p>
          </div>
          <span class="status-pill" :class="brokerSuggestions.length ? 'ok' : 'err'">
            {{ brokerSuggestions.length ? "ACTIVE" : "WAITING FOR SIGNALS" }}
          </span>
        </div>

        <div v-if="brokerSuggestions.length" class="plugin-body">
          <article class="broker-feature">
            <p class="broker-kicker">top route</p>
            <h4 class="broker-title">{{ brokerSuggestions[0].task.title }}</h4>
            <p class="broker-copy">
              send to <strong>{{ brokerSuggestions[0].agent.name }}</strong> / {{ brokerSuggestions[0].agent.role || "generalist" }}
            </p>
            <div class="broker-reasons">
              <span v-for="reason in brokerSuggestions[0].reasons" :key="reason" class="reason-pill">{{ reason }}</span>
            </div>
          </article>

          <div class="broker-list">
            <article v-for="item in brokerSuggestions.slice(1)" :key="`${item.task.id}-${item.agent.id}`" class="broker-item">
              <div>
                <p class="task-title">{{ item.task.title }}</p>
                <p class="plugin-desc">{{ item.agent.name }} / {{ item.task.priority }}</p>
              </div>
              <span class="broker-score">{{ item.score }}</span>
            </article>
          </div>
        </div>
        <p v-else class="plugin-empty">
          needs live agents plus non-completed PM tasks before it can suggest assignments.
        </p>
      </GlassCard>

      <GlassCard class="plugin-card">
        <div class="plugin-header">
          <span class="plugin-icon">TM</span>
          <div class="plugin-meta">
            <h3 class="plugin-name">TM-LITE</h3>
            <p class="plugin-desc">task ingestion from the obsidian vault tasks folder</p>
          </div>
          <span class="status-pill" :class="tmTasks !== null ? 'ok' : 'err'">
            {{ tmTasks !== null ? "CONNECTED" : tmError ? "ERROR" : "LOADING" }}
          </span>
        </div>

        <div v-if="tmTasks !== null" class="plugin-body">
          <div class="stat-row">
            <span class="stat-label">open tasks</span>
            <span class="stat-value">{{ tmTasks.length }}</span>
          </div>
          <ul v-if="tmTasks.length" class="task-list">
            <li v-for="task in tmTasks.slice(0, 6)" :key="task.id" class="task-item">
              <span class="task-priority" :class="task.priority">o</span>
              <span class="task-title">{{ task.title }}</span>
              <span class="task-project">{{ task.project }}</span>
            </li>
          </ul>
          <p v-else class="plugin-empty">no open tasks.</p>
        </div>
        <p v-else-if="tmError" class="plugin-error">{{ tmError }}</p>
      </GlassCard>

      <GlassCard class="plugin-card">
        <div class="plugin-header">
          <span class="plugin-icon">GH</span>
          <div class="plugin-meta">
            <h3 class="plugin-name">GITHUB</h3>
            <p class="plugin-desc">repo health via the github api</p>
          </div>
          <span class="status-pill" :class="githubStore.repos.length ? 'ok' : 'err'">
            {{ githubStore.repos.length ? "CONNECTED" : githubStore.loading ? "LOADING" : "NO DATA" }}
          </span>
        </div>

        <div v-if="githubStore.repos.length" class="plugin-body">
          <div class="stat-row">
            <span class="stat-label">tracked repos</span>
            <span class="stat-value">{{ githubStore.repos.length }}</span>
          </div>
          <ul class="task-list">
            <li v-for="repo in githubStore.repos" :key="repo.id" class="task-item">
              <span class="task-priority medium">+</span>
              <span class="task-title">{{ repo.name }}</span>
              <span class="task-project">{{ repo.openIssues }} issues</span>
            </li>
          </ul>
        </div>
        <p v-else-if="!githubStore.loading" class="plugin-empty">
          no repos loaded. set a github pat in settings and refresh.
        </p>
      </GlassCard>

      <GlassCard class="plugin-card roadmap-card">
        <div class="plugin-header">
          <span class="plugin-icon">NX</span>
          <div class="plugin-meta">
            <h3 class="plugin-name">NEXT PLUGINS</h3>
            <p class="plugin-desc">the additions that would make UMBRA feel like real mission control</p>
          </div>
        </div>

        <div class="roadmap-grid">
          <article v-for="plugin in roadmapPlugins" :key="plugin.name" class="roadmap-item">
            <div class="roadmap-head">
              <strong>{{ plugin.name }}</strong>
              <span>{{ plugin.tag }}</span>
            </div>
            <p>{{ plugin.description }}</p>
          </article>
        </div>
      </GlassCard>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ViewHero from "@/components/layout/ViewHero.vue";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import { buildAssignmentSuggestions } from "@/lib/assignment-broker";
import { useAgentStore } from "@/stores/useAgentStore";
import { useGithubStore } from "@/stores/useGithubStore";
import { useTaskStore } from "@/stores/useTaskStore";

interface RecentNote {
  name: string;
  modified: string;
}

interface ObsidianStats {
  connected: boolean;
  totalNotes: number;
  recentNotes: RecentNote[];
  vaultPath: string;
}

interface TmTask {
  id: string;
  title: string;
  status: string;
  project: string;
  priority: string;
}

const agentStore = useAgentStore();
const githubStore = useGithubStore();
const taskStore = useTaskStore();

const loading = ref(false);
const obsidian = ref<ObsidianStats | null>(null);
const obsidianError = ref<string | null>(null);
const tmTasks = ref<TmTask[] | null>(null);
const tmError = ref<string | null>(null);
const roadmapPlugins = [
  {
    name: "release radar",
    tag: "delivery",
    description: "pulls commits, open prs and pm blockers into one release-ready lane with changelog hints.",
  },
  {
    name: "build sentinel",
    tag: "ops",
    description: "ingests ci, test and crash signals so agents see breakage before they ship on top of it.",
  },
  {
    name: "vault graph lens",
    tag: "knowledge",
    description: "maps notes, prompts and skills into reusable clusters instead of flat lists.",
  },
  {
    name: "artifact watcher",
    tag: "delivery",
    description: "tracks exported builds, installers and release assets across local folders and ci.",
  },
];

const brokerSuggestions = computed(() => buildAssignmentSuggestions(taskStore.tasks, agentStore.agents, 4));

async function loadObsidian() {
  try {
    obsidian.value = await invoke<ObsidianStats>("get_obsidian_stats");
    obsidianError.value = null;
  } catch (error) {
    obsidianError.value = String(error);
  }
}

async function loadTm() {
  try {
    tmTasks.value = await invoke<TmTask[]>("get_tmlite_tasks");
    tmError.value = null;
  } catch (error) {
    tmTasks.value = [];
    tmError.value = String(error);
  }
}

async function refresh() {
  loading.value = true;
  await Promise.all([
    loadObsidian(),
    loadTm(),
    githubStore.loadRepos(),
    agentStore.loadAgents(),
    agentStore.setupLiveUpdates(),
    taskStore.fetchTasks(),
    taskStore.setupLiveUpdates(),
  ]);
  loading.value = false;
}

function shortPath(path: string) {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts.length > 3 ? ".../" + parts.slice(-2).join("/") : path;
}

function relTime(iso: string) {
  const diff = Date.now() - new Date(iso).getTime();
  const minutes = Math.floor(diff / 60_000);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

onMounted(refresh);
</script>

<style scoped>
.plugins-view {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.page-header,
.plugin-header,
.stat-row,
.roadmap-head,
.broker-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.page-kicker,
.plugin-desc,
.broker-kicker,
.status-pill,
.stat-label,
.task-project,
.recent-time {
  font-family: var(--font-mono);
  font-size: 11px;
}

.page-kicker,
.plugin-desc,
.stat-label,
.task-project,
.recent-time {
  color: var(--text-muted);
}

.page-title,
.plugin-name,
.broker-title {
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-display);
}

.page-title {
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

.page-subtitle,
.plugin-empty,
.roadmap-item p,
.broker-copy,
.plugin-error {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.plugin-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.plugin-card,
.plugin-body,
.recent-list,
.broker-feature,
.broker-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.plugin-icon {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
}

.plugin-meta {
  flex: 1;
}

.plugin-name {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.status-pill,
.reason-pill,
.broker-score {
  padding: 5px 9px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 82%, transparent);
}

.status-pill.ok {
  color: var(--accent-success);
}

.status-pill.err,
.plugin-error {
  color: var(--accent-error);
}

.stat-label {
  min-width: 80px;
  text-transform: uppercase;
  letter-spacing: 0.12em;
}

.stat-value,
.broker-title {
  font-size: 14px;
  color: var(--text-primary);
}

.stat-value.mono {
  font-family: var(--font-mono);
  font-size: 11px;
}

.recent-list ul,
.task-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.recent-item,
.task-item,
.broker-item {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  padding: 7px 0;
  border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
}

.recent-item:last-child,
.task-item:last-child,
.broker-item:last-child {
  border-bottom: none;
}

.recent-name,
.task-title {
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-item {
  align-items: center;
  font-size: 12px;
}

.task-priority {
  width: 14px;
  text-align: center;
  flex-shrink: 0;
}

.task-priority.critical {
  color: #ff3333;
}

.task-priority.high {
  color: var(--accent);
}

.task-priority.medium {
  color: var(--accent-secondary);
}

.task-priority.low {
  color: var(--text-muted);
}

.broker-card {
  gap: 12px;
}

.broker-feature {
  padding: 12px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
  background: color-mix(in srgb, var(--bg-elevated) 34%, transparent);
}

.broker-copy strong {
  color: var(--text-primary);
}

.broker-reasons {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.reason-pill,
.broker-score {
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 28%, transparent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.roadmap-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.roadmap-item {
  padding: 11px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  background: color-mix(in srgb, var(--bg-elevated) 38%, transparent);
}

.roadmap-head {
  align-items: baseline;
  margin-bottom: 6px;
  color: var(--text-primary);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.roadmap-head span {
  color: var(--accent);
  font-family: var(--font-mono);
}

:global([data-theme="light"]) .broker-feature,
:global([data-theme="light"]) .roadmap-item {
  background: rgba(248, 250, 252, 0.88);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .status-pill,
:global([data-theme="light"]) .reason-pill,
:global([data-theme="light"]) .broker-score {
  background: rgba(255, 255, 255, 0.88);
}

@media (max-width: 960px) {
  .plugin-grid,
  .roadmap-grid {
    grid-template-columns: 1fr;
  }
}
</style>
