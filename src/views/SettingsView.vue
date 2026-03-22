<template>
  <div class="settings-view">
    <ViewHero
      kicker="configuration"
      title="Settings"
      subtitle="theme, vault, launch targets and external service settings."
    >
      <template #meta>
        <span class="view-hero-pill">{{ draft.theme }} active</span>
        <span class="view-hero-pill">{{ draft.pmToolPollSeconds }}s pm poll</span>
      </template>
    </ViewHero>

    <form class="settings-form" @submit.prevent="save">
      <GlassCard>
        <h3 class="card-title">appearance</h3>
        <div class="field">
          <label class="field-label">theme</label>
          <div class="theme-swatches">
            <button
              v-for="t in themes"
              :key="t.value"
              class="theme-swatch"
              :class="{ active: draft.theme === t.value }"
              type="button"
              @click="applyTheme(t.value)"
            >
              <span class="swatch-dot" :style="{ background: t.color }" />
              {{ t.label }}
            </button>
          </div>
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">obsidian vault</h3>
        <div class="field">
          <label class="field-label">vault path</label>
          <input v-model="draft.vaultPath" class="field-input glass-input" type="text" />
        </div>
        <div class="field">
          <label class="field-label">notes subdirectory</label>
          <input v-model="draft.notesSubdir" class="field-input glass-input" type="text" placeholder="UMBRA_Notes" />
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">launch targets</h3>
        <div v-for="(target, i) in draft.launchTargets" :key="i" class="launch-row">
          <input v-model="target.name" class="glass-input launch-name" placeholder="name" />
          <input v-model="target.path" class="glass-input launch-path" placeholder="executable path" />
          <NeonButton variant="danger" size="sm" ghost @click="draft.launchTargets!.splice(i, 1)">delete</NeonButton>
        </div>
        <NeonButton size="sm" variant="secondary" @click="addLaunchTarget">+ add target</NeonButton>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">pm tool</h3>
        <div class="field">
          <label class="field-label">api url</label>
          <input v-model="draft.pmToolUrl" class="field-input glass-input" type="text" placeholder="https://pm-tool.local/api" />
        </div>
        <div class="field">
          <label class="field-label">dashboard url</label>
          <input
            v-model="draft.pmToolDashboardUrl"
            class="field-input glass-input"
            type="text"
            placeholder="https://pm-tool.local"
          />
        </div>
        <div class="field">
          <label class="field-label">poll interval (seconds)</label>
          <input v-model.number="draft.pmToolPollSeconds" class="field-input glass-input" type="number" min="5" max="300" />
        </div>
        <div class="doc-links">
          <NeonButton size="sm" variant="secondary" ghost :disabled="!pmDocsUrl" @click="openExternal(pmDocsUrl)">open api docs</NeonButton>
          <NeonButton size="sm" variant="secondary" ghost :disabled="!pmDashboardUrl" @click="openExternal(pmDashboardUrl)">open tool dashboard</NeonButton>
        </div>
        <p class="field-hint">leave blank if you do not want a live PM integration. docs use the api url, dashboard prefers the explicit dashboard url.</p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">release & updates</h3>
        <div class="field">
          <label class="field-label">updater endpoint</label>
          <textarea
            v-model="draft.updaterEndpoint"
            class="field-input glass-input updater-textarea"
            rows="3"
            placeholder="https://releases.example.com/latest.json"
          />
        </div>
        <div class="field">
          <label class="field-label">updater public key</label>
          <textarea
            v-model="draft.updaterPublicKey"
            class="field-input glass-input updater-textarea"
            rows="4"
            placeholder="CONTENT FROM PUBLICKEY.PEM"
          />
        </div>
        <label class="checkbox-row">
          <input v-model="draft.autoCheckForUpdates" type="checkbox" />
          <span>check for updates on app start</span>
        </label>
        <div class="doc-links">
          <NeonButton size="sm" variant="secondary" ghost :loading="checkingUpdates" @click="checkForUpdates">check now</NeonButton>
          <NeonButton
            size="sm"
            variant="secondary"
            ghost
            :disabled="!canInstallUpdate"
            :loading="installingUpdate"
            @click="installUpdate"
          >
            install pending update
          </NeonButton>
        </div>
        <p class="field-hint">
          {{ updateMessage }}
        </p>
        <p class="field-hint">
          runtime update checks work from these settings. signed updater bundles still require signing env vars during the release build.
        </p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">taskboard</h3>
        <div class="field">
          <label class="field-label">lane defaults</label>
          <div class="lane-pref-list">
            <div v-for="lane in laneOptions" :key="lane.kind" class="lane-pref-row">
              <div class="lane-copy">
                <span class="lane-name">{{ lane.label }}</span>
                <span class="lane-hint">{{ lane.hint }}</span>
              </div>
              <div class="lane-pref-actions">
                <button
                  v-for="mode in laneModes"
                  :key="mode.value"
                  type="button"
                  class="lane-mode"
                  :class="{ active: lanePrefValue(lane.kind) === mode.value }"
                  @click="setLanePref(lane.kind, mode.value)"
                >
                  {{ mode.label }}
                </button>
              </div>
            </div>
          </div>
        </div>
        <p class="field-hint">`smart` keeps UMBRA defaults. `expanded` and `collapsed` override the board startup state explicitly.</p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">uap</h3>
        <div class="field">
          <label class="field-label">advertise host</label>
          <input v-model="draft.uapAdvertiseHost" class="field-input glass-input" type="text" placeholder="127.0.0.1" />
        </div>
        <div class="field-row">
          <div class="field">
            <label class="field-label">port</label>
            <input v-model.number="draft.uapPort" class="field-input glass-input" type="number" min="1" max="65535" />
          </div>
          <div class="field">
            <label class="field-label">agent token</label>
            <input v-model="draft.uapToken" class="field-input glass-input" type="text" autocomplete="off" />
          </div>
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">github</h3>
        <div class="field">
          <label class="field-label">local repo root</label>
          <input
            v-model="draft.repoRootPath"
            class="field-input glass-input"
            type="text"
            placeholder="C:/Repos"
          />
        </div>
        <div class="field">
          <label class="field-label">personal access token</label>
          <input v-model="draft.githubPat" class="field-input glass-input" type="password" placeholder="ghp_..." autocomplete="off" />
        </div>
        <p class="field-hint">token needs <code>public_repo</code> scope, or <code>repo</code> for private repos. launcher all-repos needs a token; local repo actions use the root path above.</p>
      </GlassCard>

      <div class="form-actions">
        <NeonButton type="submit" variant="primary" :loading="saving">save settings</NeonButton>
        <span v-if="saved" class="saved-label">saved.</span>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import ViewHero from "@/components/layout/ViewHero.vue";
import type { AppConfig, UpdateCheckResult } from "@/interfaces";
import { useConfigStore } from "@/stores/useConfigStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";

const configStore = useConfigStore();
const saving = ref(false);
const saved = ref(false);
const checkingUpdates = ref(false);
const installingUpdate = ref(false);
const lastUpdateCheck = ref<UpdateCheckResult | null>(null);
const updateError = ref("");

const themes = [
  { value: "ember", label: "ember", color: "#d4520a" },
  { value: "neon", label: "neon", color: "#00f5ff" },
  { value: "light", label: "light", color: "#3b82f6" },
];

const laneOptions = [
  { kind: "backlog", label: "backlog", hint: "smart = collapse only on dense boards" },
  { kind: "in_progress", label: "in progress", hint: "usually best left open" },
  { kind: "review", label: "review", hint: "smart = starts collapsed when populated" },
  { kind: "done", label: "done", hint: "smart = starts collapsed when populated" },
] as const;

const laneModes = [
  { value: "smart", label: "smart" },
  { value: "expanded", label: "expanded" },
  { value: "collapsed", label: "collapsed" },
] as const;

const pmApiUrl = computed(() => draft.pmToolUrl.trim().replace(/\/+$/, ""));
const pmDocsUrl = computed(() => {
  if (!pmApiUrl.value) return "";
  return `${pmApiUrl.value}/docs`;
});
const pmDashboardUrl = computed(() => {
  const explicit = draft.pmToolDashboardUrl.trim().replace(/\/+$/, "");
  if (explicit) return explicit;
  if (!pmApiUrl.value) return "";
  try {
    const url = new URL(pmApiUrl.value);
    if (url.port === "8000") {
      url.port = "4173";
    }
    url.pathname = "";
    url.search = "";
    url.hash = "";
    return url.toString().replace(/\/+$/, "");
  } catch {
    return "";
  }
});
const canInstallUpdate = computed(() => Boolean(lastUpdateCheck.value?.updateAvailable) && !installingUpdate.value);
const updateMessage = computed(() => {
  if (updateError.value) return updateError.value;
  if (!draft.updaterEndpoint.trim() || !draft.updaterPublicKey.trim()) {
    return "set endpoint + public key to enable runtime update checks.";
  }
  if (checkingUpdates.value) return "checking release feed...";
  if (installingUpdate.value) return "installing update. windows will close the app if an update is ready.";
  if (!lastUpdateCheck.value) return "no check yet.";
  if (!lastUpdateCheck.value.configured) return "updater config incomplete.";
  if (lastUpdateCheck.value.updateAvailable) {
    return `update ${lastUpdateCheck.value.version} is ready for install.`;
  }
  return `no update available. current version: ${lastUpdateCheck.value.currentVersion}.`;
});

function applyTheme(t: string) {
  draft.theme = t;
  configStore.setTheme(t);
}

const draft = reactive<AppConfig>({ ...configStore.config });

watch(
  () => configStore.config,
  (c) => Object.assign(draft, c),
  { deep: true }
);

function addLaunchTarget() {
  if (!draft.launchTargets) draft.launchTargets = [];
  draft.launchTargets.push({ id: crypto.randomUUID(), name: "", path: "", icon: "LN" });
}

function lanePrefValue(kind: (typeof laneOptions)[number]["kind"]) {
  const pref = draft.taskLanePrefs?.[kind];
  if (pref === true) return "collapsed";
  if (pref === false) return "expanded";
  return "smart";
}

function setLanePref(kind: (typeof laneOptions)[number]["kind"], value: (typeof laneModes)[number]["value"]) {
  const next = { ...(draft.taskLanePrefs ?? {}) };
  if (value === "smart") {
    delete next[kind];
  } else {
    next[kind] = value === "collapsed";
  }
  draft.taskLanePrefs = next;
}

async function openExternal(url: string) {
  if (!url) return;
  try {
    await shellOpen(url);
  } catch {
    window.open(url, "_blank");
  }
}

async function checkForUpdates() {
  checkingUpdates.value = true;
  updateError.value = "";
  try {
    lastUpdateCheck.value = await invoke<UpdateCheckResult>("check_for_updates");
  } catch (error) {
    updateError.value = `update check failed: ${String(error)}`;
  } finally {
    checkingUpdates.value = false;
  }
}

async function installUpdate() {
  if (!canInstallUpdate.value) return;
  installingUpdate.value = true;
  updateError.value = "";
  try {
    const installed = await invoke<boolean>("install_pending_update");
    if (!installed) {
      updateError.value = "no pending update available.";
      return;
    }
    lastUpdateCheck.value = null;
  } catch (error) {
    updateError.value = `update install failed: ${String(error)}`;
  } finally {
    installingUpdate.value = false;
  }
}

async function save() {
  saving.value = true;
  saved.value = false;
  try {
    await configStore.saveConfig(draft);
    saved.value = true;
    setTimeout(() => (saved.value = false), 2000);
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.settings-view {
  max-width: 860px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.page-kicker,
.card-title,
.field-label {
  margin: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.page-title {
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

.page-subtitle,
.field-hint,
.saved-label {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card-title {
  margin-bottom: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.field:last-child {
  margin-bottom: 0;
}

.field-row {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 12px;
}

.field-input {
  font-size: 13px;
  padding: 10px 12px;
}

.updater-textarea {
  min-height: 88px;
  resize: vertical;
}

.launch-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.launch-name {
  width: 140px;
}

.launch-path {
  flex: 1;
}

.form-actions,
.doc-links,
.theme-swatches,
.lane-pref-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.lane-pref-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.lane-pref-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 10px 0;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
}

.lane-pref-row:first-child {
  padding-top: 0;
  border-top: 0;
}

.lane-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.lane-name {
  color: var(--text-primary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-size: 12px;
}

.lane-hint {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.lane-mode {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 7px 12px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 80%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color 0.16s ease, color 0.16s ease, background 0.16s ease;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.lane-mode:hover,
.lane-mode.active {
  border-color: color-mix(in srgb, var(--accent) 28%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--accent);
}

.saved-label {
  color: var(--accent-success);
}

.checkbox-row {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
  font-size: 13px;
}

.checkbox-row input {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
}

.field-hint code {
  font-family: var(--font-mono);
  color: var(--accent-secondary);
  font-size: 11px;
}

.theme-swatch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 80%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color 0.16s ease, color 0.16s ease, background 0.16s ease;
}

.theme-swatch:hover,
.theme-swatch.active {
  border-color: color-mix(in srgb, var(--accent) 28%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--accent);
}

.swatch-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

:global([data-theme="light"]) .theme-swatch {
  background: rgba(255, 255, 255, 0.88);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .lane-mode {
  background: rgba(255, 255, 255, 0.9);
  border-color: rgba(15, 23, 42, 0.12);
}

:global([data-theme="light"]) .theme-swatch:hover,
:global([data-theme="light"]) .theme-swatch.active,
:global([data-theme="light"]) .lane-mode:hover,
:global([data-theme="light"]) .lane-mode.active {
  border-color: rgba(8, 145, 178, 0.2);
  background: rgba(240, 249, 255, 0.96);
}

@media (max-width: 760px) {
  .field-row {
    grid-template-columns: 1fr;
  }

  .launch-row {
    flex-direction: column;
    align-items: stretch;
  }

  .lane-pref-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .launch-name {
    width: 100%;
  }
}
</style>
