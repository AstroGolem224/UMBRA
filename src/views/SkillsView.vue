<template>
  <div class="skills-view">
    <header class="page-header">
      <h1 class="page-title">SKILLS</h1>
      <span class="page-subtitle">{{ filtered.length }} skills</span>
      <NeonButton size="sm" variant="secondary" ghost :loading="loading" @click="load">↺</NeonButton>
    </header>

    <div class="toolbar">
      <input v-model="search" class="search-input glass-input" placeholder="Search skills..." />
    </div>

    <div v-if="loading" class="loading">Scanning ~/.claude/skills/...</div>
    <div v-else-if="error" class="error-msg">{{ error }}</div>
    <div v-else class="skills-grid">
      <GlassCard
        v-for="skill in filtered"
        :key="skill.id"
        clickable
        @click="selected = skill"
      >
        <div class="skill-header">
          <span class="skill-name">{{ skill.name }}</span>
          <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
        </div>
        <p class="skill-desc">{{ skill.description || 'No description.' }}</p>
      </GlassCard>
    </div>

    <Transition name="fade">
      <div v-if="selected" class="skill-modal-backdrop" @click.self="selected = null">
        <div class="skill-modal glass-panel">
          <div class="modal-header">
            <span class="modal-title">{{ selected.name }}</span>
            <button class="close-btn" @click="selected = null">✕</button>
          </div>
          <div v-if="selected.version" class="modal-section">
            <span class="modal-label">VERSION</span>
            <span class="modal-val">v{{ selected.version }}</span>
          </div>
          <div class="modal-section">
            <span class="modal-label">FOLDER</span>
            <span class="modal-val mono">~/.claude/skills/{{ selected.id }}/</span>
          </div>
          <p class="modal-desc">{{ selected.description || 'No description available.' }}</p>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";

interface SkillInfo {
  id: string;
  name: string;
  version: string;
  description: string;
}

const skills = ref<SkillInfo[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const search = ref("");
const selected = ref<SkillInfo | null>(null);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    skills.value = await invoke<SkillInfo[]>("list_skills");
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

const filtered = computed(() =>
  skills.value.filter(
    (s) => !search.value || s.name.toLowerCase().includes(search.value.toLowerCase()) || s.description.toLowerCase().includes(search.value.toLowerCase())
  )
);

onMounted(load);
</script>

<style scoped>
.skills-view {
  max-width: 1200px;
  position: relative;
}

.page-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 20px;
}

.page-title {
  font-family: var(--font-display), "Iceland", monospace;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.2em;
  color: var(--text-primary);
  margin: 0;
}

.page-subtitle {
  font-size: 12px;
  color: var(--text-muted);
  letter-spacing: 0.1em;
  flex: 1;
}

.toolbar {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.search-input {
  flex: 1;
  font-size: 12px;
  padding: 7px 12px;
}

.loading, .error-msg {
  font-size: 13px;
  color: var(--text-muted);
  padding: 20px 0;
}
.error-msg { color: var(--accent-error); }

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.skill-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.skill-name {
  font-family: var(--font-display), "Iceland", monospace;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--text-primary);
}

.skill-version {
  font-size: 10px;
  font-family: var(--font-mono), monospace;
  color: var(--accent);
  letter-spacing: 0.06em;
}

.skill-desc {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.skill-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.skill-modal {
  width: min(560px, 90vw);
  max-height: 80vh;
  overflow-y: auto;
  padding: 24px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.modal-title {
  font-family: var(--font-display), "Iceland", monospace;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--text-primary);
  text-transform: uppercase;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 6px;
  border-radius: 4px;
}
.close-btn:hover { color: var(--text-primary); background: var(--bg-surface-hover); }

.modal-section {
  margin-bottom: 10px;
}

.modal-label {
  display: block;
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.modal-val {
  font-size: 12px;
  font-family: var(--font-display), "Iceland", monospace;
  color: var(--accent);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.modal-val.mono {
  font-family: var(--font-mono), monospace;
  text-transform: none;
  font-size: 11px;
}

.modal-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 12px 0 0;
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.15s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
