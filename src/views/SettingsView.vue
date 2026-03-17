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
          <select v-model="draft.theme" class="field-input glass-input">
            <option value="ember">Ember (default)</option>
            <option value="neon">Neon</option>
            <option value="light">Light</option>
          </select>
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
          <input v-model="draft.pmToolUrl" class="field-input glass-input" type="text" placeholder="http://localhost:4173" />
        </div>
        <div class="field">
          <label class="field-label">Poll Interval (seconds)</label>
          <input v-model.number="draft.pmToolPollSeconds" class="field-input glass-input" type="number" min="5" max="300" />
        </div>
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
</style>
