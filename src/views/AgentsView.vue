<template>
  <div class="agents-view">
    <header class="page-header">
      <h1 class="page-title">AGENTS</h1>
      <span class="page-subtitle">{{ onlineCount }} / {{ agentStore.agents.length }} online</span>
    </header>

    <div v-if="agentStore.loading" class="loading">Loading agents...</div>
    <div v-else-if="agentStore.error" class="error-msg">{{ agentStore.error }}</div>

    <div v-else class="agents-grid">
      <AgentCard
        v-for="agent in agentStore.agents"
        :key="agent.id"
        :agent="agent"
        @select="openPanel"
      />
    </div>

    <!-- Detail Panel -->
    <Transition name="slide-in">
      <div v-if="selectedAgent" class="detail-panel glass-panel">
        <div class="detail-header">
          <span class="detail-name">{{ selectedAgent.name }}</span>
          <button class="close-btn" @click="selectedAgent = null">&#x2715;</button>
        </div>

        <div class="detail-section">
          <span class="detail-label">STATUS</span>
          <StatusBadge :status="selectedAgent.status" />
        </div>

        <div class="detail-section">
          <span class="detail-label">ALLOWED TOOLS</span>
          <div class="tool-list">
            <span v-for="tool in selectedAgent.allowedTools" :key="tool" class="tool-tag">{{ tool }}</span>
          </div>
        </div>

        <div class="detail-section">
          <span class="detail-label">SKILLS</span>
          <div class="tool-list">
            <span v-for="skill in selectedAgent.skills" :key="skill" class="tool-tag">{{ skill }}</span>
          </div>
        </div>

        <div class="detail-section">
          <span class="detail-label">LAST SEEN</span>
          <span class="detail-val">{{ new Date(selectedAgent.lastSeen).toLocaleString() }}</span>
        </div>

        <!-- Notes -->
        <div class="detail-section">
          <span class="detail-label">NOTES</span>
          <textarea
            v-model="draftNote.notes"
            class="notes-input glass-input"
            placeholder="Notizen zum Agenten..."
            rows="3"
          />
        </div>

        <!-- Link -->
        <div class="detail-section">
          <span class="detail-label">DASHBOARD LINK</span>
          <div class="link-row">
            <input
              v-model="draftNote.link"
              class="glass-input link-input"
              placeholder="http://..."
              type="text"
            />
            <button
              v-if="draftNote.link"
              class="open-btn"
              title="Open in browser"
              @click="openLink"
            >&#x2197;</button>
          </div>
        </div>

        <NeonButton size="sm" variant="primary" :loading="saving" @click="saveNote">
          SAVE
        </NeonButton>
        <span v-if="saved" class="saved-label">Saved.</span>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from "vue";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import AgentCard from "@/components/agents/AgentCard.vue";
import StatusBadge from "@/components/ui/StatusBadge.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import type { Agent } from "@/interfaces";

const agentStore = useAgentStore();
const configStore = useConfigStore();

const selectedAgent = ref<Agent | null>(null);
const saving = ref(false);
const saved = ref(false);
const onlineCount = computed(() => agentStore.agents.filter((a) => a.status === "online").length);

const draftNote = reactive({ notes: "", link: "" });

function openPanel(agent: Agent) {
  selectedAgent.value = agent;
  const existing = configStore.config.agentNotes?.[agent.id];
  draftNote.notes = existing?.notes ?? "";
  draftNote.link = existing?.link ?? "";
  saved.value = false;
}

async function saveNote() {
  if (!selectedAgent.value) return;
  saving.value = true;
  try {
    const agentNotes = { ...(configStore.config.agentNotes ?? {}) };
    agentNotes[selectedAgent.value.id] = { notes: draftNote.notes, link: draftNote.link };
    await configStore.saveConfig({ ...configStore.config, agentNotes });
    saved.value = true;
    setTimeout(() => (saved.value = false), 2000);
  } finally {
    saving.value = false;
  }
}

async function openLink() {
  if (draftNote.link) {
    try {
      await shellOpen(draftNote.link);
    } catch {
      window.open(draftNote.link, "_blank");
    }
  }
}

onMounted(() => {
  agentStore.loadAgents();
  agentStore.setupLiveUpdates();
});
</script>

<style scoped>
.agents-view {
  max-width: 1200px;
  position: relative;
}

.page-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 24px;
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
}

.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.loading, .error-msg {
  font-size: 13px;
  color: var(--text-muted);
  padding: 20px 0;
}
.error-msg { color: var(--accent-error); }

.detail-panel {
  position: fixed;
  right: 24px;
  top: 80px;
  width: 320px;
  padding: 20px;
  z-index: 100;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.detail-name {
  font-family: var(--font-display), "Iceland", monospace;
  font-size: 18px;
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

.detail-section {
  margin-bottom: 14px;
}

.detail-label {
  display: block;
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
  margin-bottom: 6px;
  text-transform: uppercase;
}

.detail-val {
  font-size: 12px;
  color: var(--text-secondary);
}

.tool-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tool-tag {
  font-size: 10px;
  font-family: "Iceland", monospace;
  padding: 2px 7px;
  border-radius: 3px;
  background: var(--bg-surface);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
}

.notes-input {
  width: 100%;
  resize: vertical;
  font-size: 12px;
  padding: 7px 10px;
  font-family: var(--font-sans), sans-serif;
  line-height: 1.5;
  box-sizing: border-box;
}

.link-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.link-input {
  flex: 1;
  font-size: 12px;
  padding: 7px 10px;
  font-family: var(--font-mono), monospace;
}

.open-btn {
  background: var(--accent-dim);
  border: 1px solid var(--accent);
  color: var(--accent);
  cursor: pointer;
  font-size: 14px;
  padding: 5px 10px;
  border-radius: 6px;
  transition: all 0.15s;
  flex-shrink: 0;
}
.open-btn:hover { background: var(--accent); color: var(--bg-primary); }

.saved-label {
  font-size: 11px;
  color: var(--accent-success);
  font-family: "Iceland", monospace;
  letter-spacing: 0.1em;
  margin-left: 8px;
}

.slide-in-enter-active, .slide-in-leave-active {
  transition: transform 0.2s, opacity 0.2s;
}
.slide-in-enter-from, .slide-in-leave-to {
  transform: translateX(20px);
  opacity: 0;
}
</style>
