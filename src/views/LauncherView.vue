<template>
  <div class="launcher-view">
    <header class="page-header">
      <h1 class="page-title">LAUNCHER</h1>
      <span class="page-subtitle">IDE & GitHub</span>
    </header>

    <section class="section">
      <h2 class="section-title">IDEs</h2>
      <div class="launcher-grid">
        <GlassCard
          v-for="ide in ides"
          :key="ide.id"
          clickable
          :variant="launching === ide.id ? 'accent' : 'default'"
          @click="launchIde(ide)"
        >
          <div class="launcher-item">
            <span class="launcher-icon">{{ ide.icon }}</span>
            <div class="launcher-info">
              <span class="launcher-name">{{ ide.name }}</span>
              <span class="launcher-path">{{ ide.path }}</span>
            </div>
            <span v-if="launching === ide.id" class="launching-label">Launching...</span>
          </div>
        </GlassCard>
      </div>
    </section>

    <section class="section">
      <h2 class="section-title">
        GITHUB
        <button
          class="refresh-btn"
          :class="{ spinning: githubStore.loading }"
          @click="githubStore.loadRepos()"
          title="Refresh repo status"
        >⟳</button>
      </h2>
      <div class="launcher-grid">
        <GlassCard
          v-for="repo in githubTargets"
          :key="repo.id"
          clickable
          @click="openGithub(repo)"
        >
          <div class="launcher-item">
            <span class="launcher-icon">⌥</span>
            <div class="launcher-info">
              <span class="launcher-name">{{ repo.name }}</span>
              <span class="launcher-path">{{ repo.owner }}/{{ repo.repo }}</span>
            </div>
          </div>
          <div v-if="githubStore.repoById(repo.id)" class="repo-status">
            <span class="repo-stat">
              <span class="repo-stat-icon">◈</span>
              {{ githubStore.repoById(repo.id)!.openIssues }} open
            </span>
            <span v-if="githubStore.repoById(repo.id)!.pushedAt" class="repo-stat">
              <span class="repo-stat-icon">↑</span>
              {{ relativeTime(githubStore.repoById(repo.id)!.pushedAt!) }}
            </span>
          </div>
          <div v-else-if="githubStore.loading" class="repo-status-loading">fetching...</div>
        </GlassCard>
      </div>
      <div v-if="githubStore.error" class="error-bar">{{ githubStore.error }}</div>
    </section>

    <div v-if="lastError" class="error-bar">{{ lastError }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useConfigStore } from "@/stores/useConfigStore";
import { useGithubStore } from "@/stores/useGithubStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import type { LaunchTarget, GithubOpenTarget } from "@/interfaces";

const configStore = useConfigStore();
const githubStore = useGithubStore();
const launching = ref<string | null>(null);
const lastError = ref<string | null>(null);

const ides = computed<LaunchTarget[]>(() => configStore.config.launchTargets ?? []);
const githubTargets = computed<GithubOpenTarget[]>(() => configStore.config.githubTargets ?? []);

onMounted(() => {
  githubStore.loadRepos();
});

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60_000);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  return `${days}d ago`;
}

async function launchIde(target: LaunchTarget) {
  launching.value = target.id;
  lastError.value = null;
  try {
    await invoke("launch_target", { targetId: target.id });
  } catch (e) {
    lastError.value = String(e);
  } finally {
    launching.value = null;
  }
}

async function openGithub(target: GithubOpenTarget) {
  lastError.value = null;
  try {
    await invoke("open_github", { owner: target.owner, repo: target.repo });
  } catch (e) {
    lastError.value = String(e);
  }
}
</script>

<style scoped>
.launcher-view {
  max-width: 900px;
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

.section { margin-bottom: 28px; }

.section-title {
  font-family: "Iceland", monospace;
  font-size: 11px;
  letter-spacing: 0.2em;
  color: var(--text-muted);
  margin: 0 0 12px;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 8px;
}

.refresh-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  padding: 0;
  line-height: 1;
  transition: color 0.2s;
}
.refresh-btn:hover { color: var(--accent); }
.refresh-btn.spinning { animation: spin 0.8s linear infinite; }

@keyframes spin { to { transform: rotate(360deg); } }

.launcher-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
}

.launcher-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.launcher-icon {
  font-size: 24px;
  width: 32px;
  text-align: center;
  color: var(--accent);
  flex-shrink: 0;
}

.launcher-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.launcher-name {
  font-family: "Iceland", monospace;
  font-size: 14px;
  letter-spacing: 0.08em;
  color: var(--text-primary);
}

.launcher-path {
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.launching-label {
  font-size: 10px;
  color: var(--accent);
  font-family: "Iceland", monospace;
  letter-spacing: 0.08em;
  animation: blink 0.8s step-end infinite;
}

@keyframes blink { 50% { opacity: 0; } }

.repo-status {
  display: flex;
  gap: 12px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--glass-border);
}

.repo-stat {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.repo-stat-icon {
  color: var(--accent);
  font-size: 9px;
}

.repo-status-loading {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--glass-border);
  font-size: 10px;
  color: var(--text-muted);
  font-family: "Iceland", monospace;
  letter-spacing: 0.08em;
  animation: blink 1.2s step-end infinite;
}

.error-bar {
  margin-top: 16px;
  padding: 10px 14px;
  border-radius: 6px;
  background: rgba(255, 50, 50, 0.08);
  border: 1px solid var(--accent-error);
  color: var(--accent-error);
  font-size: 12px;
}
</style>
