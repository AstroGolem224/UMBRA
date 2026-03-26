<template>
  <div class="agents-view">
    <ViewHero
      kicker="roster"
      title="Agents"
      :subtitle="`${onlineCount} / ${agentStore.agents.length} online with live routing metadata.`"
    >
      <template #meta>
        <span class="view-hero-pill">{{ onlineCount }} live</span>
        <NeonButton size="sm" variant="primary" @click="showAddModal = true">+ add agent</NeonButton>
      </template>
    </ViewHero>

    <div v-if="agentStore.loading" class="state-card">loading agents...</div>
    <div v-else-if="agentStore.error" class="error-card">{{ agentStore.error }}</div>

    <section v-else class="agents-shell">
      <div class="agents-grid">
        <AgentCard
          v-for="agent in agentStore.agents"
          :key="agent.id"
          :agent="agent"
          @select="openPanel"
        />
      </div>

      <aside class="telemetry-card glass-panel">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">uap telemetry</p>
            <h2 class="panel-title">agent intake</h2>
          </div>
          <span class="meta-pill">live hook</span>
        </div>

        <div class="telemetry-block">
          <span class="block-label">heartbeat endpoint</span>
          <code class="code-pill">{{ uapEndpoint }}</code>
        </div>

        <div class="telemetry-block">
          <span class="block-label">auth mode</span>
          <code class="code-pill">per-agent token</code>
        </div>

        <div class="telemetry-block">
          <span class="block-label">selected token</span>
          <code class="code-pill">{{ selectedAgentToken || "select an agent to inspect its token" }}</code>
        </div>

        <p class="telemetry-copy">timeout: 30 min / auth: <code>x-agent-token</code> must match the token provisioned for that exact agent id.</p>
      </aside>
    </section>

    <Transition name="slide-in">
      <div v-if="selectedAgent" class="detail-panel glass-panel">
        <div class="detail-header">
          <div>
            <p class="panel-kicker">agent detail</p>
            <span class="detail-name">{{ selectedAgent.name }}</span>
          </div>
          <button class="close-btn" @click="selectedAgent = null">x</button>
        </div>

        <div class="detail-section">
          <span class="detail-label">status</span>
          <StatusBadge :status="selectedAgent.status" />
        </div>

        <div class="detail-section">
          <span class="detail-label">allowed tools</span>
          <div class="tool-list">
            <span v-for="tool in selectedAgent.allowedTools" :key="tool" class="tool-tag">{{ tool }}</span>
          </div>
        </div>

        <div class="detail-section">
          <span class="detail-label">skills</span>
          <div class="tool-list">
            <span v-for="skill in selectedAgent.skills" :key="skill" class="tool-tag">{{ skill }}</span>
          </div>
        </div>

        <div class="detail-section">
          <span class="detail-label">last seen</span>
          <span class="detail-val">{{ new Date(selectedAgent.lastSeen).toLocaleString() }}</span>
        </div>

        <div class="detail-section">
          <span class="detail-label">uap token</span>
          <code class="code-pill">{{ selectedAgentToken || "no token provisioned" }}</code>
        </div>

        <div class="detail-section">
          <span class="detail-label">notes</span>
          <textarea
            v-model="draftNote.notes"
            class="notes-input glass-input"
            placeholder="notes about this agent..."
            rows="4"
          />
        </div>

        <div class="detail-section">
          <span class="detail-label">dashboard link</span>
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
              title="open in browser"
              @click="openLink"
            >open</button>
          </div>
        </div>

        <div class="detail-section">
          <span class="detail-label">push task</span>
          <div class="push-task-form">
            <input
              v-model="taskDraft.title"
              class="glass-input compact-input"
              placeholder="task title..."
              @keyup.enter="pushTask"
            />
            <div class="push-row">
              <select v-model="taskDraft.priority" class="glass-input priority-select">
                <option value="critical">critical</option>
                <option value="high">high</option>
                <option value="medium">medium</option>
                <option value="low">low</option>
              </select>
              <NeonButton size="sm" variant="primary" :loading="pushing" @click="pushTask">
                push
              </NeonButton>
            </div>
            <span v-if="pushed" class="saved-label">pushed.</span>
          </div>
        </div>

        <div class="detail-divider" />

        <div class="panel-footer">
          <NeonButton size="sm" variant="primary" :loading="saving" @click="saveNote">
            save notes
          </NeonButton>
          <span v-if="saved" class="saved-label">saved.</span>
          <NeonButton
            size="sm"
            variant="danger"
            ghost
            :loading="deleting"
            style="margin-left:auto"
            @click="deleteAgent"
          >delete</NeonButton>
        </div>
      </div>
    </Transition>

    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showAddModal" class="modal-backdrop" @click.self="closeAddModal">
          <div class="modal glass-panel">
            <div class="modal-header">
              <span class="modal-title">add agent</span>
              <button class="close-btn" @click="closeAddModal">x</button>
            </div>

            <div class="form-row">
              <div class="form-field">
                <label class="form-label">name</label>
                <input v-model="newAgent.name" class="glass-input" placeholder="e.g. Scout" @input="syncId" />
              </div>
              <div class="form-field">
                <label class="form-label">id</label>
                <input v-model="newAgent.id" class="glass-input" placeholder="e.g. scout" />
              </div>
            </div>

            <div class="form-field">
              <label class="form-label">role</label>
              <input v-model="newAgent.role" class="glass-input" placeholder="e.g. Research Agent" />
            </div>

            <div class="form-field">
              <label class="form-label">skills <span class="hint">comma separated</span></label>
              <input v-model="newAgent.skillsRaw" class="glass-input" placeholder="e.g. python, research, summarization" />
            </div>

            <div class="form-field">
              <label class="form-label">tools <span class="hint">comma separated</span></label>
              <input v-model="newAgent.toolsRaw" class="glass-input" placeholder="e.g. browser, filesystem" />
            </div>

            <div v-if="addError" class="add-error">{{ addError }}</div>

            <div class="modal-footer">
              <NeonButton variant="secondary" ghost size="sm" @click="closeAddModal">cancel</NeonButton>
              <NeonButton
                variant="primary"
                size="sm"
                :loading="adding"
                :disabled="!newAgent.name.trim() || !newAgent.id.trim()"
                @click="submitAddAgent"
              >add</NeonButton>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import AgentCard from "@/components/agents/AgentCard.vue";
import ViewHero from "@/components/layout/ViewHero.vue";
import type { Agent, CustomAgentConfig } from "@/interfaces";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import NeonButton from "@/components/ui/NeonButton.vue";
import StatusBadge from "@/components/ui/StatusBadge.vue";

const agentStore = useAgentStore();
const configStore = useConfigStore();

const selectedAgent = ref<Agent | null>(null);
const saving = ref(false);
const saved = ref(false);
const pushing = ref(false);
const pushed = ref(false);
const deleting = ref(false);
const onlineCount = computed(() =>
  agentStore.agents.filter((a) => ["online", "working", "idle"].includes(a.status)).length
);
const selectedAgentToken = computed(() =>
  selectedAgent.value ? configStore.config.agentAuthTokens?.[selectedAgent.value.id] ?? "" : ""
);
const uapEndpoint = computed(
  () =>
    `http://${configStore.config.uapAdvertiseHost}:${configStore.config.uapPort}/api/agents/<id>/heartbeat`
);

const showAddModal = ref(false);
const adding = ref(false);
const addError = ref<string | null>(null);
const newAgent = reactive({ name: "", id: "", role: "", skillsRaw: "", toolsRaw: "" });

function syncId() {
  newAgent.id = newAgent.name.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
}

function closeAddModal() {
  showAddModal.value = false;
  addError.value = null;
  Object.assign(newAgent, { name: "", id: "", role: "", skillsRaw: "", toolsRaw: "" });
}

async function submitAddAgent() {
  if (!newAgent.name.trim() || !newAgent.id.trim()) return;
  adding.value = true;
  addError.value = null;
  try {
    const cfg: CustomAgentConfig = {
      id: newAgent.id.trim(),
      name: newAgent.name.trim(),
      role: newAgent.role.trim(),
      skills: newAgent.skillsRaw.split(",").map((s) => s.trim()).filter(Boolean),
      allowedTools: newAgent.toolsRaw.split(",").map((s) => s.trim()).filter(Boolean),
    };
    await agentStore.addAgent(cfg);
    await configStore.load();
    closeAddModal();
  } catch (e) {
    addError.value = String(e);
  } finally {
    adding.value = false;
  }
}

async function deleteAgent() {
  if (!selectedAgent.value) return;
  deleting.value = true;
  try {
    await agentStore.removeAgent(selectedAgent.value.id);
    await configStore.load();
    selectedAgent.value = null;
  } finally {
    deleting.value = false;
  }
}

const draftNote = reactive({ notes: "", link: "" });
const taskDraft = reactive({ title: "", priority: "medium" });

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

async function pushTask() {
  if (!selectedAgent.value || !taskDraft.title.trim()) return;
  pushing.value = true;
  try {
    await agentStore.pushTask(selectedAgent.value.id, {
      title: taskDraft.title.trim(),
      priority: taskDraft.priority as "critical" | "high" | "medium" | "low",
    });
    taskDraft.title = "";
    pushed.value = true;
    setTimeout(() => (pushed.value = false), 2000);
  } finally {
    pushing.value = false;
  }
}

async function openLink() {
  if (!draftNote.link) return;

  try {
    await shellOpen(draftNote.link);
  } catch {
    window.open(draftNote.link, "_blank");
  }
}

onMounted(async () => {
  if (!configStore.loaded) {
    await configStore.load();
  }
  await agentStore.loadAgents();
  await agentStore.setupLiveUpdates();
});
</script>

<style scoped>
.agents-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.page-header,
.panel-head,
.detail-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.page-header {
  align-items: center;
}

.header-copy {
  max-width: 620px;
}

.header-actions,
.panel-footer,
.link-row,
.push-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-kicker,
.panel-kicker,
.detail-label,
.block-label,
.form-label,
.hint {
  margin: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.page-title,
.panel-title,
.detail-name {
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-display);
}

.page-title {
  font-size: 30px;
  font-weight: 800;
  letter-spacing: 0.08em;
}

.page-subtitle,
.telemetry-copy,
.detail-val,
.saved-label {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.meta-pill,
.code-pill,
.tool-tag {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  border: none;
  background: color-mix(in srgb, var(--accent) 8%, var(--bg-surface));
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
}

.code-pill {
  word-break: break-all;
}

.agents-shell {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(280px, 0.7fr);
  gap: 14px;
  align-items: start;
}

.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
}

.telemetry-card,
.state-card,
.error-card {
  padding: 18px;
  border-radius: var(--radius-xl);
}

.telemetry-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.telemetry-block,
.detail-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-card {
  color: var(--text-muted);
  border: 1px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 88%, transparent);
}

.error-card,
.add-error {
  color: var(--accent-error);
  border: 1px solid rgba(239, 68, 68, 0.24);
  background: rgba(239, 68, 68, 0.08);
  padding: 12px 14px;
  border-radius: var(--radius-lg);
}

.detail-panel {
  position: fixed;
  right: 24px;
  top: 84px;
  width: 340px;
  padding: 20px;
  z-index: 100;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
  border-radius: var(--radius-xl);
}

.detail-header {
  align-items: flex-start;
}

.detail-name {
  font-size: 24px;
  font-weight: 800;
  line-height: 1;
  letter-spacing: 0.05em;
}

.tool-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tool-tag {
  padding: 5px 9px;
}

.notes-input,
.link-input,
.compact-input,
.priority-select {
  font-size: 12px;
  line-height: 1.6;
}

.notes-input {
  width: 100%;
  resize: vertical;
  min-height: 110px;
}

.link-input {
  flex: 1;
}

.open-btn,
.close-btn {
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 92%, transparent);
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  padding: 7px 10px;
  transition: border-color 0.16s ease, color 0.16s ease, background 0.16s ease;
}

.open-btn:hover,
.close-btn:hover {
  border-color: color-mix(in srgb, var(--accent) 26%, transparent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.push-task-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.priority-select {
  flex: 1;
}

.detail-divider {
  border: none;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  margin: 0;
}

.saved-label {
  color: var(--accent-success);
}

.modal-enter-active,
.modal-leave-active,
.slide-in-enter-active,
.slide-in-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.modal-enter-from,
.modal-leave-to,
.slide-in-enter-from,
.slide-in-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

@media (max-width: 1240px) {
  .agents-shell {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 820px) {
  .page-header,
  .header-actions {
    flex-direction: column;
    align-items: flex-start;
  }

  .detail-panel {
    position: static;
    width: 100%;
    max-height: none;
  }
}
</style>

<style>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
  backdrop-filter: blur(2px);
}

.modal {
  width: min(500px, 92vw);
  max-height: 85vh;
  overflow-y: auto;
  padding: 24px;
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.modal-title {
  color: var(--text-primary);
  font-size: 20px;
  font-weight: 700;
  text-transform: lowercase;
}

.modal .form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.modal .form-row {
  display: flex;
  gap: 12px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 10px;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
}
</style>
