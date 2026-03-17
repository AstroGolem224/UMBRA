<template>
  <div class="skills-view">
    <header class="page-header">
      <h1 class="page-title">SKILLS</h1>
      <span class="page-subtitle">{{ filtered.length }} skills</span>
    </header>

    <div class="toolbar">
      <input v-model="search" class="search-input glass-input" placeholder="Search skills..." />
      <select v-model="filterAgent" class="agent-filter glass-input">
        <option value="">All Agents</option>
        <option value="prism">Prism</option>
        <option value="forge">Forge</option>
        <option value="jim">Jim</option>
      </select>
    </div>

    <div v-if="agentStore.loading" class="loading">Loading skills...</div>
    <div v-else class="skills-grid">
      <GlassCard
        v-for="skill in filtered"
        :key="skill.id"
        clickable
        @click="selected = skill"
      >
        <div class="skill-header">
          <span class="skill-name">{{ skill.name }}</span>
          <span class="skill-agent">{{ skill.agentId }}</span>
        </div>
        <p class="skill-desc">{{ skill.description }}</p>
      </GlassCard>
    </div>

    <Transition name="fade">
      <div v-if="selected" class="skill-modal-backdrop" @click.self="selected = null">
        <div class="skill-modal glass-panel">
          <div class="modal-header">
            <span class="modal-title">{{ selected.name }}</span>
            <button class="close-btn" @click="selected = null">✕</button>
          </div>
          <p class="modal-desc">{{ selected.description }}</p>
          <div class="modal-section">
            <span class="modal-label">AGENT</span>
            <span class="modal-val">{{ selected.agentId }}</span>
          </div>
          <div v-if="selected.promptTemplate" class="modal-section">
            <span class="modal-label">PROMPT TEMPLATE</span>
            <pre class="prompt-pre">{{ selected.promptTemplate }}</pre>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useAgentStore } from "@/stores/useAgentStore";
import GlassCard from "@/components/ui/GlassCard.vue";

interface Skill {
  id: string;
  name: string;
  description: string;
  agentId: string;
  promptTemplate?: string;
}

const agentStore = useAgentStore();
const search = ref("");
const filterAgent = ref("");
const selected = ref<Skill | null>(null);

const allSkills = computed<Skill[]>(() => {
  return agentStore.agents.flatMap((agent) =>
    agent.skills.map((s) => ({
      id: `${agent.id}:${s}`,
      name: s,
      description: `${agent.name} skill: ${s}`,
      agentId: agent.id,
    }))
  );
});

const filtered = computed(() =>
  allSkills.value.filter((s) => {
    const matchSearch =
      !search.value || s.name.toLowerCase().includes(search.value.toLowerCase());
    const matchAgent = !filterAgent.value || s.agentId === filterAgent.value;
    return matchSearch && matchAgent;
  })
);
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

.agent-filter {
  font-family: "Iceland", monospace;
  font-size: 12px;
  padding: 7px 12px;
  background: var(--glass-bg);
  color: var(--text-primary);
  border: 1px solid var(--glass-border);
  border-radius: 6px;
  cursor: pointer;
}
.agent-filter option { background: var(--bg-surface); }

.loading {
  font-size: 13px;
  color: var(--text-muted);
}

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
  font-family: "Iceland", monospace;
  font-size: 13px;
  letter-spacing: 0.08em;
  color: var(--text-primary);
}

.skill-agent {
  font-size: 10px;
  font-family: "Iceland", monospace;
  color: var(--accent);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.skill-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}

/* Modal */
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
  width: min(600px, 90vw);
  max-height: 80vh;
  overflow-y: auto;
  padding: 24px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.modal-title {
  font-family: "Iceland", monospace;
  font-size: 20px;
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

.modal-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0 0 16px;
}

.modal-section {
  margin-bottom: 12px;
}

.modal-label {
  display: block;
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.modal-val {
  font-size: 12px;
  font-family: "Iceland", monospace;
  color: var(--accent);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.prompt-pre {
  background: var(--bg-surface);
  padding: 12px;
  border-radius: 6px;
  border-left: 2px solid var(--accent);
  font-size: 12px;
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
