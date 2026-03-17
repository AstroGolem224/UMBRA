<template>
  <div class="settings-view">
    <header class="page-header">
      <h1 class="page-title">SETTINGS</h1>
    </header>

    <form class="settings-form" @submit.prevent="save">
      <GlassCard>
        <h3 class="card-title">APPEARANCE</h3>
        <div class="field">
          <label class="field-label">Theme</label>
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
        <h3 class="card-title">OBSIDIAN VAULT</h3>
        <div class="field">
          <label class="field-label">Vault Path</label>
          <input v-model="draft.vaultPath" class="field-input glass-input" type="text" />
        </div>
        <div class="field">
          <label class="field-label">Notes Subdirectory</label>
          <input v-model="draft.notesSubdir" class="field-input glass-input" type="text" placeholder="UMBRA_Notes" />
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">LAUNCH TARGETS</h3>
        <div v-for="(target, i) in draft.launchTargets" :key="i" class="launch-row">
          <input v-model="target.name" class="glass-input" placeholder="Name" style="width: 120px" />
          <input v-model="target.path" class="glass-input" placeholder="Executable path" style="flex:1" />
          <NeonButton variant="danger" size="sm" ghost @click="draft.launchTargets!.splice(i, 1)">✕</NeonButton>
        </div>
        <NeonButton size="sm" variant="secondary" @click="addLaunchTarget">+ Add Target</NeonButton>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">PM TOOL</h3>
        <div class="field">
          <label class="field-label">PM Tool URL</label>
          <input v-model="draft.pmToolUrl" class="field-input glass-input" type="text" placeholder="http://100.115.61.30:8000" />
        </div>
        <div class="field">
          <label class="field-label">Poll Interval (seconds)</label>
          <input v-model.number="draft.pmToolPollSeconds" class="field-input glass-input" type="number" min="5" max="300" />
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">GITHUB</h3>
        <div class="field">
          <label class="field-label">Personal Access Token (PAT)</label>
          <input v-model="draft.githubPat" class="field-input glass-input" type="password" placeholder="ghp_..." autocomplete="off" />
        </div>
        <p class="field-hint">Token needs <code>public_repo</code> scope (or <code>repo</code> for private repos). Leave blank for public repos without rate-limit auth.</p>
      </GlassCard>

      <div class="form-actions">
        <NeonButton type="submit" variant="primary" :loading="saving">SAVE SETTINGS</NeonButton>
        <span v-if="saved" class="saved-label">Saved.</span>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import { useConfigStore } from "@/stores/useConfigStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import type { AppConfig } from "@/interfaces";

const configStore = useConfigStore();
const saving = ref(false);
const saved = ref(false);

const themes = [
  { value: "ember", label: "Ember", color: "#d4520a" },
  { value: "neon",  label: "Neon",  color: "#00f5ff" },
  { value: "light", label: "Light", color: "#3b82f6" },
];

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
  draft.launchTargets.push({ id: crypto.randomUUID(), name: "", path: "", icon: "▶" });
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
  max-width: 700px;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-family: "Iceland", monospace;
  font-size: 24px;
  letter-spacing: 0.2em;
  color: var(--text-primary);
  margin: 0;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-title {
  font-family: "Iceland", monospace;
  font-size: 11px;
  letter-spacing: 0.2em;
  color: var(--text-muted);
  margin: 0 0 14px;
  text-transform: uppercase;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.field:last-child { margin-bottom: 0; }

.field-label {
  font-size: 11px;
  letter-spacing: 0.1em;
  color: var(--text-muted);
}

.field-input {
  font-size: 13px;
  padding: 8px 12px;
}

.launch-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.form-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.saved-label {
  font-size: 12px;
  color: var(--accent-success);
  font-family: "Iceland", monospace;
  letter-spacing: 0.1em;
}

.field-hint {
  font-size: 10px;
  color: var(--text-muted);
  line-height: 1.5;
  margin-top: 6px;
}

.field-hint code {
  font-family: var(--font-mono);
  color: var(--accent-secondary);
  font-size: 10px;
}

.theme-swatches {
  display: flex;
  gap: 8px;
}

.theme-swatch {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 7px 14px;
  border-radius: 8px;
  border: 1px solid var(--glass-border);
  background: var(--bg-surface);
  color: var(--text-secondary);
  font-family: "Iceland", monospace;
  font-size: 12px;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.15s;
}

.theme-swatch:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.theme-swatch.active {
  border-color: var(--accent);
  background: var(--accent-dim);
  color: var(--accent);
}

.swatch-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
