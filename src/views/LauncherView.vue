<template>
  <div class="launcher-view">
    <ViewHero
      kicker="launch targets"
      title="Launcher"
      subtitle="ides, repo shortcuts and pinned delivery entry points."
    >
      <template #meta>
        <span class="view-hero-pill">{{ ides.length }} ides</span>
        <span class="view-hero-pill">{{ githubTargets.length }} pinned repos</span>
      </template>
    </ViewHero>

    <section class="summary-grid">
      <GlassCard class="summary-card">
        <p class="summary-label">ides</p>
        <p class="summary-value">{{ ides.length }}</p>
        <p class="summary-copy">local launch targets configured in settings</p>
      </GlassCard>

      <GlassCard class="summary-card">
        <p class="summary-label">all repos</p>
        <p class="summary-value">{{ allRepos.length }}</p>
        <p class="summary-copy">{{ reposLoading ? "refreshing live repo directory" : "available for direct browser launch" }}</p>
      </GlassCard>

      <GlassCard class="summary-card">
        <p class="summary-label">pinned repos</p>
        <p class="summary-value">{{ githubTargets.length }}</p>
        <p class="summary-copy">tracked delivery repos with live issue state</p>
      </GlassCard>

      <GlassCard class="summary-card">
        <p class="summary-label">repo signal</p>
        <p class="summary-value">{{ repoSignal }}</p>
        <p class="summary-copy">{{ githubStore.loading ? "loading github health" : "fast read on current repo readiness" }}</p>
      </GlassCard>
    </section>

    <section class="launcher-stage">
      <GlassCard class="stage-panel">
        <div class="section-head">
          <h2 class="section-title">ides</h2>
          <span class="section-meta">{{ ides.length }} routes</span>
        </div>
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
              <span v-if="launching === ide.id" class="launching-label">launching...</span>
            </div>
            <div class="quick-actions">
              <button class="quick-btn primary" @click.stop="launchIde(ide)">launch</button>
              <button class="quick-btn" @click.stop="copyValue(`ide:${ide.id}`, ide.path)">
                {{ copyLabel(`ide:${ide.id}`, "copy path") }}
              </button>
            </div>
          </GlassCard>
        </div>
      </GlassCard>

      <div class="repo-stack">
        <GlassCard class="stage-panel">
          <div class="section-head">
            <h2 class="section-title">all repos</h2>
            <button
              class="refresh-btn"
              :class="{ spinning: reposLoading }"
              @click="loadAllRepos"
              title="Refresh repo list"
            >
              refresh
            </button>
          </div>
          <div class="repo-dropdown-row">
            <div class="repo-select-wrap">
              <select
                v-model="selectedRepo"
                class="repo-select glass-input"
                :disabled="reposLoading || allRepos.length === 0"
              >
                <option value="" disabled>
                  {{ reposLoading ? 'Loading...' : allRepos.length === 0 ? 'No repos (check PAT in Settings)' : 'Select a repo...' }}
                </option>
                <option v-for="repo in allRepos" :key="repo.fullName" :value="repo.htmlUrl">
                  {{ repo.private ? '[private] ' : '' }}{{ repo.name }}
                </option>
              </select>
            </div>
            <button class="open-btn" :disabled="!selectedRepo" @click="openSelectedRepo">open repo</button>
            <button class="open-btn subtle" :disabled="!selectedRepo" @click="copyValue(`repo:${selectedRepo}`, selectedRepo)">
              {{ copyLabel(`repo:${selectedRepo}`, "copy link") }}
            </button>
          </div>
          <div class="quick-actions dense">
            <button class="quick-btn" :disabled="!selectedRepoEntry" @click="openSelectedIssues">issues</button>
            <button class="quick-btn" :disabled="!selectedRepoEntry" @click="openSelectedPulls">prs</button>
            <button class="quick-btn" :disabled="!selectedRepoEntry || !repoRootAvailable" @click="openSelectedFolder">folder</button>
            <button class="quick-btn" :disabled="!selectedRepoEntry || !repoRootAvailable" @click="openSelectedTerminal">terminal</button>
            <button class="quick-btn" :disabled="!selectedRepoEntry" @click="copySelectedClone('https')">
              {{ copyLabel(selectedRepoCloneKey("https"), "copy https") }}
            </button>
            <button class="quick-btn" :disabled="!selectedRepoEntry" @click="copySelectedClone('ssh')">
              {{ copyLabel(selectedRepoCloneKey("ssh"), "copy ssh") }}
            </button>
          </div>
          <p class="section-copy">full repo directory for quick browser handoff without leaving UMBRA.</p>
          <div v-if="reposError" class="error-bar">{{ reposError }}</div>
        </GlassCard>

        <GlassCard class="stage-panel">
          <div class="section-head">
            <h2 class="section-title">pinned repos</h2>
            <button
              class="refresh-btn"
              :class="{ spinning: githubStore.loading }"
              @click="githubStore.loadRepos()"
              title="Refresh repo status"
            >
              refresh
            </button>
          </div>
          <div class="launcher-grid compact">
            <GlassCard
              v-for="repo in githubTargets"
              :key="repo.id"
              clickable
              @click="openGithub(repo)"
            >
              <div class="launcher-item">
                <span class="launcher-icon">GH</span>
                <div class="launcher-info">
                  <span class="launcher-name">{{ repo.name }}</span>
                  <span class="launcher-path">{{ repo.owner }}/{{ repo.repo }}</span>
                </div>
              </div>
              <div class="quick-actions">
                <button class="quick-btn primary" @click.stop="openGithub(repo)">open</button>
                <button class="quick-btn" @click.stop="openRepoIssues(repo)">issues</button>
                <button class="quick-btn" @click.stop="openRepoPulls(repo)">prs</button>
                <button class="quick-btn" :disabled="!repoRootAvailable" @click.stop="openLocalRepoFolder(repo.repo)">folder</button>
                <button class="quick-btn" :disabled="!repoRootAvailable" @click.stop="openLocalRepoTerminal(repo.repo)">terminal</button>
                <button class="quick-btn" @click.stop="copyValue(`pinned:${repo.id}`, repoUrl(repo))">
                  {{ copyLabel(`pinned:${repo.id}`, "copy link") }}
                </button>
                <button class="quick-btn" @click.stop="copyValue(`pinned:https:${repo.id}`, cloneHttpsUrl(repoFullName(repo)))">
                  {{ copyLabel(`pinned:https:${repo.id}`, "copy https") }}
                </button>
                <button class="quick-btn" @click.stop="copyValue(`pinned:ssh:${repo.id}`, cloneSshUrl(repoFullName(repo)))">
                  {{ copyLabel(`pinned:ssh:${repo.id}`, "copy ssh") }}
                </button>
              </div>
              <div v-if="githubStore.repoById(repo.id)" class="repo-status">
                <span class="repo-stat">
                  <span class="repo-stat-icon">o</span>
                  {{ githubStore.repoById(repo.id)!.openIssues }} open
                </span>
                <span v-if="githubStore.repoById(repo.id)!.pushedAt" class="repo-stat">
                  <span class="repo-stat-icon">^</span>
                  {{ relativeTime(githubStore.repoById(repo.id)!.pushedAt!) }}
                </span>
              </div>
              <div v-else-if="githubStore.loading" class="repo-status-loading">fetching...</div>
            </GlassCard>
          </div>
          <div v-if="githubStore.error" class="error-bar">{{ githubStore.error }}</div>
        </GlassCard>
      </div>
    </section>

    <div v-if="lastError" class="error-bar">{{ lastError }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ViewHero from "@/components/layout/ViewHero.vue";
import { useConfigStore } from "@/stores/useConfigStore";
import { useGithubStore } from "@/stores/useGithubStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import type { LaunchTarget, GithubOpenTarget } from "@/interfaces";

const configStore = useConfigStore();
const githubStore = useGithubStore();
const launching = ref<string | null>(null);
const copiedKey = ref<string | null>(null);
const lastError = ref<string | null>(null);

interface SlimRepo {
  name: string;
  fullName: string;
  htmlUrl: string;
  private: boolean;
  description: string;
  pushedAt: string;
}

const allRepos = ref<SlimRepo[]>([]);
const reposLoading = ref(false);
const reposError = ref<string | null>(null);
const selectedRepo = ref("");
const selectedRepoEntry = computed(() => allRepos.value.find((repo) => repo.htmlUrl === selectedRepo.value) ?? null);
const repoRootAvailable = computed(() => Boolean(configStore.config.repoRootPath?.trim()));

const ides = computed<LaunchTarget[]>(() => configStore.config.launchTargets ?? []);
const githubTargets = computed<GithubOpenTarget[]>(() => configStore.config.githubTargets ?? []);
const repoSignal = computed(() => {
  if (githubStore.loading) return "...";
  if (githubStore.error) return "warn";
  const issues = githubStore.repos.reduce((sum, repo) => sum + repo.openIssues, 0);
  if (issues === 0 && githubTargets.value.length > 0) return "clear";
  if (issues <= 9) return "steady";
  return "busy";
});

async function loadAllRepos() {
  reposLoading.value = true;
  reposError.value = null;
  try {
    allRepos.value = (await invoke<SlimRepo[]>("list_user_repos")) ?? [];
  } catch (e) {
    reposError.value = String(e);
  } finally {
    reposLoading.value = false;
  }
}

async function openSelectedRepo() {
  if (!selectedRepo.value) return;
  try {
    await openGithubUrl(selectedRepo.value);
  } catch (e) {
    lastError.value = String(e);
  }
}

function repoUrl(target: GithubOpenTarget) {
  return `https://github.com/${target.owner}/${target.repo}`;
}

function repoFullName(target: GithubOpenTarget) {
  return `${target.owner}/${target.repo}`;
}

function issuesUrl(fullName: string) {
  return `https://github.com/${fullName}/issues`;
}

function pullsUrl(fullName: string) {
  return `https://github.com/${fullName}/pulls`;
}

function cloneHttpsUrl(fullName: string) {
  return `https://github.com/${fullName}.git`;
}

function cloneSshUrl(fullName: string) {
  return `git@github.com:${fullName}.git`;
}

function selectedRepoCloneKey(mode: "https" | "ssh") {
  return selectedRepoEntry.value ? `selected:${mode}:${selectedRepoEntry.value.fullName}` : `selected:${mode}:empty`;
}

function copyLabel(key: string, fallback: string) {
  return copiedKey.value === key ? "copied" : fallback;
}

async function copyValue(key: string, value: string) {
  lastError.value = null;
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(value);
    } else {
      const input = document.createElement("textarea");
      input.value = value;
      input.setAttribute("readonly", "true");
      input.style.position = "absolute";
      input.style.left = "-9999px";
      document.body.appendChild(input);
      input.select();
      document.execCommand("copy");
      document.body.removeChild(input);
    }
    copiedKey.value = key;
    window.setTimeout(() => {
      if (copiedKey.value === key) copiedKey.value = null;
    }, 1400);
  } catch (e) {
    lastError.value = `Copy failed: ${String(e)}`;
  }
}

async function openGithubUrl(url: string) {
  lastError.value = null;
  await invoke("open_github_url", { url });
}

async function openSelectedIssues() {
  if (!selectedRepoEntry.value) return;
  await openGithubUrl(issuesUrl(selectedRepoEntry.value.fullName));
}

async function openSelectedPulls() {
  if (!selectedRepoEntry.value) return;
  await openGithubUrl(pullsUrl(selectedRepoEntry.value.fullName));
}

async function copySelectedClone(mode: "https" | "ssh") {
  if (!selectedRepoEntry.value) return;
  const fullName = selectedRepoEntry.value.fullName;
  const url = mode === "https" ? cloneHttpsUrl(fullName) : cloneSshUrl(fullName);
  await copyValue(selectedRepoCloneKey(mode), url);
}

async function openLocalRepoFolder(repoName: string) {
  lastError.value = null;
  try {
    await invoke("open_local_repo_folder", { repoName });
  } catch (e) {
    lastError.value = String(e);
  }
}

async function openLocalRepoTerminal(repoName: string) {
  lastError.value = null;
  try {
    await invoke("open_local_repo_terminal", { repoName });
  } catch (e) {
    lastError.value = String(e);
  }
}

async function openSelectedFolder() {
  if (!selectedRepoEntry.value) return;
  await openLocalRepoFolder(selectedRepoEntry.value.name);
}

async function openSelectedTerminal() {
  if (!selectedRepoEntry.value) return;
  await openLocalRepoTerminal(selectedRepoEntry.value.name);
}

onMounted(() => {
  loadAllRepos();
  if (githubStore.repos.length === 0) githubStore.loadRepos();
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

async function openRepoIssues(target: GithubOpenTarget) {
  await openGithubUrl(issuesUrl(repoFullName(target)));
}

async function openRepoPulls(target: GithubOpenTarget) {
  await openGithubUrl(pullsUrl(repoFullName(target)));
}
</script>

<style scoped>
.launcher-view {
  max-width: 980px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.page-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.page-kicker {
  margin: 0 0 4px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.page-title {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
  color: var(--text-primary);
  margin: 0;
}

.page-subtitle {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  letter-spacing: 0.08em;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.summary-card,
.stage-panel,
.repo-stack,
.section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.summary-label,
.section-title {
  font-family: var(--font-mono);
  font-size: 12px;
  letter-spacing: 0.12em;
  color: var(--text-secondary);
  margin: 0;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 8px;
}

.summary-label {
  margin: 0;
  font-size: 11px;
}

.summary-value {
  margin: 0;
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: 800;
  color: var(--text-primary);
  letter-spacing: 0.04em;
}

.summary-copy,
.section-copy {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.launcher-stage {
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(320px, 0.85fr);
  gap: 12px;
  align-items: start;
}

.repo-stack {
  gap: 12px;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-meta {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.refresh-btn {
  padding: 4px 8px;
  background: color-mix(in srgb, var(--accent) 6%, var(--bg-surface));
  border: none;
  border-radius: var(--radius-pill);
  color: var(--text-secondary);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  line-height: 1;
  transition: color 0.2s, background 0.2s;
}

.refresh-btn:hover {
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-surface));
}

.refresh-btn.spinning {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.launcher-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
}

.launcher-grid.compact {
  grid-template-columns: 1fr;
}

.launcher-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.quick-actions.dense {
  margin-top: 10px;
}

.launcher-icon {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
  flex-shrink: 0;
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.08em;
}

.launcher-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.launcher-name {
  font-family: var(--font-display);
  font-size: 14px;
  letter-spacing: 0.08em;
  color: var(--text-primary);
}

.launcher-path {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.launching-label {
  font-size: 10px;
  color: var(--accent);
  font-family: var(--font-mono);
  letter-spacing: 0.08em;
  animation: blink 0.8s step-end infinite;
}

@keyframes blink {
  50% {
    opacity: 0;
  }
}

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
  font-size: 11px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.repo-stat-icon {
  color: var(--accent);
  font-size: 10px;
}

.repo-status-loading {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--glass-border);
  font-size: 10px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  letter-spacing: 0.08em;
  animation: blink 1.2s step-end infinite;
}

.repo-dropdown-row {
  display: flex;
  gap: 8px;
  align-items: center;
  max-width: 640px;
}

.repo-select-wrap {
  flex: 1;
  min-width: 0;
}

.repo-select {
  width: 100%;
  font-family: var(--font-sans);
  font-size: 13px;
  padding: 9px 12px;
  background: var(--glass-bg);
  color: var(--text-primary);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s;
}

.repo-select:focus {
  border-color: var(--accent);
}

.repo-select option {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.repo-select:disabled {
  opacity: 0.5;
  cursor: default;
}

.open-btn {
  flex-shrink: 0;
  padding: 9px 14px;
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
  color: var(--accent);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
}

.open-btn:hover:not(:disabled) {
  background: var(--accent-dim);
}

.open-btn.subtle {
  background: color-mix(in srgb, var(--glass-bg) 86%, transparent);
  border-color: color-mix(in srgb, var(--glass-border) 88%, transparent);
  color: var(--text-secondary);
}

.open-btn.subtle:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  border-color: color-mix(in srgb, var(--accent) 35%, transparent);
  color: var(--accent);
}

.open-btn:disabled {
  opacity: 0.58;
  cursor: default;
  border-color: color-mix(in srgb, var(--glass-border) 90%, transparent);
  color: var(--text-muted);
  background: color-mix(in srgb, var(--glass-bg) 88%, transparent);
}

.quick-btn {
  border: 1px solid color-mix(in srgb, var(--glass-border) 92%, transparent);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--glass-bg) 88%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 6px 10px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}

.quick-btn:hover {
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.quick-btn:disabled {
  opacity: 0.58;
  cursor: default;
  border-color: color-mix(in srgb, var(--glass-border) 92%, transparent);
  color: var(--text-muted);
  background: color-mix(in srgb, var(--glass-bg) 88%, transparent);
}

.quick-btn:disabled:hover {
  border-color: color-mix(in srgb, var(--glass-border) 92%, transparent);
  color: var(--text-muted);
  background: color-mix(in srgb, var(--glass-bg) 88%, transparent);
}

.quick-btn.primary {
  border-color: color-mix(in srgb, var(--accent) 38%, transparent);
  color: var(--accent);
}

.error-bar {
  padding: 10px 14px;
  border-radius: var(--radius-lg);
  background: rgba(255, 50, 50, 0.08);
  border: 1px solid var(--accent-error);
  color: var(--accent-error);
  font-size: 12px;
}

:global([data-theme="light"]) .refresh-btn,
:global([data-theme="light"]) .open-btn {
  background: rgba(0, 101, 141, 0.06);
}

:global([data-theme="light"]) .open-btn.subtle,
:global([data-theme="light"]) .quick-btn {
  background: rgba(255, 255, 255, 0.94);
  border-color: rgba(15, 23, 42, 0.14);
}

:global([data-theme="light"]) .launcher-icon {
  background: rgba(8, 145, 178, 0.1);
}

@media (max-width: 760px) {
  .summary-grid,
  .launcher-stage {
    grid-template-columns: 1fr;
  }

  .repo-dropdown-row {
    flex-direction: column;
    align-items: stretch;
  }

  .open-btn {
    width: 100%;
  }
}
</style>
