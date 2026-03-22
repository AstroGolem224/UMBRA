<template>
  <div class="skills-view">
    <ViewHero
      kicker="capabilities"
      title="Skills"
      :subtitle="`${filtered.length} indexed skills across local codex skill folders.`"
    >
      <template #meta>
        <span class="view-hero-pill">{{ categories.length }} categories</span>
        <span class="view-hero-pill">{{ agents.length }} agents</span>
        <NeonButton size="sm" variant="secondary" ghost :loading="loading" @click="load">REFRESH</NeonButton>
      </template>
    </ViewHero>

    <div class="toolbar">
      <input
        v-model="search"
        class="search-input glass-input"
        placeholder="Search skills, folders or content..."
      />
    </div>

    <div class="filter-strip">
      <div class="filter-group">
        <span class="filter-label">CATEGORY</span>
        <button
          class="filter-pill"
          :class="{ active: activeCategory === null }"
          @click="activeCategory = null"
        >
          ALL
        </button>
        <button
          v-for="category in categories"
          :key="category"
          class="filter-pill"
          :class="{ active: activeCategory === category }"
          @click="activeCategory = category"
        >
          {{ category }}
        </button>
      </div>

      <div class="filter-group">
        <span class="filter-label">AGENT</span>
        <button
          class="filter-pill"
          :class="{ active: activeAgent === null }"
          @click="activeAgent = null"
        >
          ALL
        </button>
        <button
          v-for="agent in agents"
          :key="agent"
          class="filter-pill"
          :class="{ active: activeAgent === agent }"
          @click="activeAgent = agent"
        >
          {{ agent }}
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">Scanning ~/.codex/skills/...</div>
    <div v-else-if="error" class="error-msg">{{ error }}</div>
    <div v-else class="skills-grid">
      <GlassCard
        v-for="skill in filtered"
        :key="skill.id"
        clickable
        class="skill-card"
        @click="openSkill(skill)"
      >
        <div class="skill-header">
          <span class="skill-name">{{ skill.name }}</span>
          <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
        </div>

        <div class="skill-meta">
          <span class="skill-chip">{{ skill.category }}</span>
          <span v-for="agent in skill.agents" :key="agent" class="skill-chip agent">{{ agent }}</span>
        </div>

        <p class="skill-desc">{{ skill.description || "No description." }}</p>
      </GlassCard>
    </div>

    <Transition name="fade">
      <div v-if="selected" class="skill-modal-backdrop" @click.self="selected = null">
        <div class="skill-modal glass-panel">
          <div class="modal-header">
            <span class="modal-title">{{ selected.name }}</span>
            <button class="close-btn" @click="selected = null">X</button>
          </div>

          <div class="modal-grid">
            <div class="modal-section">
              <span class="modal-label">VERSION</span>
              <span class="modal-val">{{ selected.version || "n/a" }}</span>
            </div>
            <div class="modal-section">
              <span class="modal-label">CATEGORY</span>
              <span class="modal-val">{{ selected.category }}</span>
            </div>
            <div class="modal-section">
              <span class="modal-label">FOLDER</span>
              <span class="modal-val mono">~/.codex/skills/{{ selected.folder }}</span>
            </div>
            <div class="modal-section">
              <span class="modal-label">AGENTS</span>
              <div class="modal-chip-row">
                <span v-for="agent in selected.agents" :key="agent" class="skill-chip agent">{{ agent }}</span>
              </div>
            </div>
          </div>

          <p class="modal-desc">{{ selected.description || "No description available." }}</p>

          <div class="modal-section">
            <span class="modal-label">SKILL.MD</span>
            <pre class="modal-content">{{ selected.content }}</pre>
          </div>

          <div class="modal-nav">
            <NeonButton size="sm" variant="secondary" ghost :disabled="selectedIndex <= 0" @click="moveSelection(-1)">
              PREV
            </NeonButton>
            <NeonButton
              size="sm"
              variant="secondary"
              ghost
              :disabled="selectedIndex === -1 || selectedIndex >= filtered.length - 1"
              @click="moveSelection(1)"
            >
              NEXT
            </NeonButton>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ViewHero from "@/components/layout/ViewHero.vue";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";

interface SkillInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  category: string;
  agents: string[];
  content: string;
  folder: string;
}

const skills = ref<SkillInfo[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const search = ref("");
const selected = ref<SkillInfo | null>(null);
const activeCategory = ref<string | null>(null);
const activeAgent = ref<string | null>(null);

const categories = computed(() =>
  [...new Set(skills.value.map((skill) => skill.category).filter(Boolean))].sort()
);
const agents = computed(() =>
  [...new Set(skills.value.flatMap((skill) => skill.agents).filter(Boolean))].sort()
);

const filtered = computed(() => {
  const query = search.value.trim().toLowerCase();
  return skills.value.filter((skill) => {
    if (activeCategory.value && skill.category !== activeCategory.value) return false;
    if (activeAgent.value && !skill.agents.includes(activeAgent.value)) return false;
    if (!query) return true;
    return [
      skill.name,
      skill.description,
      skill.folder,
      skill.content,
      skill.category,
      ...skill.agents,
    ]
      .join("\n")
      .toLowerCase()
      .includes(query);
  });
});

const selectedIndex = computed(() =>
  selected.value ? filtered.value.findIndex((skill) => skill.id === selected.value?.id) : -1
);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    skills.value = await invoke<SkillInfo[]>("list_skills");
    if (selected.value) {
      selected.value = skills.value.find((skill) => skill.id === selected.value?.id) ?? null;
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function openSkill(skill: SkillInfo) {
  selected.value = skill;
}

function moveSelection(delta: number) {
  if (selectedIndex.value === -1) return;
  const next = filtered.value[selectedIndex.value + delta];
  if (next) {
    selected.value = next;
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (!selected.value) return;
  if (event.key === "Escape") {
    selected.value = null;
  } else if (event.key === "ArrowRight" || event.key.toLowerCase() === "j") {
    moveSelection(1);
  } else if (event.key === "ArrowLeft" || event.key.toLowerCase() === "k") {
    moveSelection(-1);
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
  load();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
});
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
  font-size: 25px;
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
  margin-bottom: 16px;
}

.search-input {
  flex: 1;
  font-size: 12px;
  padding: 7px 12px;
}

.filter-strip {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.filter-group {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.filter-label {
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
  min-width: 72px;
}

.filter-pill,
.skill-chip {
  border: 1px solid var(--glass-border);
  background: transparent;
  color: var(--text-muted);
  border-radius: 999px;
  padding: 3px 9px;
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.filter-pill {
  cursor: pointer;
  transition: all 0.12s;
}

.filter-pill:hover,
.filter-pill.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-dim);
}

.skill-chip.agent {
  color: var(--accent-secondary);
}

.loading,
.error-msg {
  font-size: 13px;
  color: var(--text-muted);
  padding: 20px 0;
}

.error-msg {
  color: var(--accent-error);
}

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}

.skill-card {
  min-height: 160px;
}

.skill-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
  gap: 8px;
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

.skill-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 10px;
}

.skill-desc {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.skill-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.skill-modal {
  width: min(900px, 94vw);
  max-height: 88vh;
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
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
}

.close-btn:hover {
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}

.modal-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 14px;
}

.modal-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.modal-label {
  display: block;
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
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

.modal-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.modal-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0 0 16px;
}

.modal-content {
  margin: 0;
  padding: 16px;
  border-radius: 10px;
  background: rgba(4, 6, 12, 0.72);
  border: 1px solid var(--glass-border);
  color: var(--text-secondary);
  font-family: var(--font-mono), monospace;
  font-size: 11px;
  line-height: 1.6;
  white-space: pre-wrap;
  overflow-x: auto;
}

.modal-nav {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 900px) {
  .modal-grid {
    grid-template-columns: 1fr;
  }
}
</style>
