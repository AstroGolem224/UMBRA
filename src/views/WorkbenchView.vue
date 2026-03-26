<template>
  <div class="workbench-view">
    <ViewHero
      kicker="dispatch"
      title="Workbench"
      :subtitle="`${readyAgentCount} ready agents · ${workspaceOptions.length} workspaces · ${workbenchStore.runs.length} runs.`"
    >
      <template #meta>
        <span class="view-hero-pill">{{ readyAgentCount }} ready</span>
        <span class="view-hero-pill">{{ workspaceOptions.length }} workspaces</span>
        <span class="view-hero-pill">{{ activeWorkspace?.writable ? "write access" : "read only" }}</span>
      </template>
    </ViewHero>

    <div v-if="workbenchStore.error" class="error-card">{{ workbenchStore.error }}</div>

    <section class="workbench-shell">
      <div class="composer-card glass-panel">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">compose</p>
            <h2 class="panel-title">dispatch work</h2>
          </div>
          <div class="summary-actions">
            <button class="artifact-action" type="button" @click="workspaceManagerOpen = !workspaceManagerOpen">
              {{ workspaceManagerOpen ? "hide workspaces" : "manage workspaces" }}
            </button>
            <span class="meta-pill">phase 9</span>
          </div>
        </div>

        <div class="composer-frame">
          <div class="composer-intro">
            <div class="composer-copy">
              <strong>{{ activeAgent?.name ?? "select an agent" }}</strong>
              <span>{{ activeWorkspace?.name ?? "select a workspace" }}</span>
            </div>
            <div class="composer-pills">
              <span class="status-chip" :class="`status-chip--${activeRunStatusTone}`">
                {{ workbenchStore.activeRun?.status ?? "idle" }}
              </span>
              <span class="ghost-pill">{{ workbenchStore.draft.mode }}</span>
              <span v-if="workbenchStore.activeRun?.outcomeStatus" class="ghost-pill">
                {{ workbenchStore.activeRun.outcomeStatus }}
              </span>
            </div>
          </div>

          <textarea
            v-model="workbenchStore.draft.prompt"
            class="glass-input prompt-input"
            rows="8"
            placeholder="describe the task or question for the selected agent..."
          />

          <div class="composer-grid">
            <label class="form-field">
              <span class="field-label">agent</span>
              <select v-model="workbenchStore.draft.agentId" class="glass-input">
                <option value="">select agent</option>
                <option v-for="agent in dispatchableAgents" :key="agent.id" :value="agent.id">
                  {{ agent.name }} · {{ agent.status }}
                </option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">workspace</span>
              <select v-model="workbenchStore.draft.workspaceId" class="glass-input">
                <option value="">select workspace</option>
                <option v-for="workspace in workspaceOptions" :key="workspace.id" :value="workspace.id">
                  {{ workspace.name }} · {{ workspace.writable ? "rw" : "ro" }}
                </option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">mode</span>
              <select v-model="workbenchStore.draft.mode" class="glass-input">
                <option value="task">task</option>
                <option value="chat">chat</option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">persona</span>
              <select v-model="workbenchStore.draft.personaId" class="glass-input">
                <option :value="null">none</option>
                <option v-for="persona in personaOptions" :key="persona.id" :value="persona.id">
                  {{ persona.name }}
                </option>
              </select>
            </label>

            <label class="form-field form-field--wide">
              <span class="field-label">linked pm task</span>
              <select v-model="workbenchStore.draft.pmTaskId" class="glass-input">
                <option :value="null">none</option>
                <option v-for="task in pmTaskOptions" :key="task.id" :value="task.id">
                  {{ task.title }}
                </option>
              </select>
            </label>
          </div>

          <div v-if="workspaceManagerOpen" class="workspace-manager">
            <div class="workspace-manager__head">
              <div>
                <p class="panel-kicker">workspace manager</p>
                <h3 class="manager-title">inline presets</h3>
              </div>
              <div class="summary-actions">
                <button class="artifact-action" type="button" @click="addWorkspacePreset">add workspace</button>
                <button class="artifact-action" type="button" :disabled="workspaceSaving" @click="saveWorkspaceManager">
                  {{ workspaceSaving ? "saving..." : "save presets" }}
                </button>
              </div>
            </div>

            <div v-if="workspaceDrafts.length" class="workspace-list">
              <div v-for="(workspace, index) in workspaceDrafts" :key="workspace.id" class="workspace-row">
                <div class="workspace-row__head">
                  <input v-model="workspace.name" class="glass-input" placeholder="workspace name" />
                  <label class="toggle-pill"><input v-model="workspace.writable" type="checkbox" /><span>writable</span></label>
                  <label class="toggle-pill">
                    <input :checked="workspaceDefaultId === workspace.id" type="radio" name="workbench-default-workspace" @change="workspaceDefaultId = workspace.id" />
                    <span>default</span>
                  </label>
                  <button class="artifact-action" type="button" @click="removeWorkspacePreset(index)">delete</button>
                </div>
                <div class="workspace-row__body">
                  <input v-model="workspace.rootPath" class="glass-input" placeholder="absolute path" />
                  <button class="artifact-action" type="button" @click="pickWorkspaceRoot(index)">pick folder</button>
                </div>
              </div>
            </div>
            <p v-else class="help-copy">no workspace presets yet.</p>
          </div>

          <div class="composer-footer">
            <div class="composer-meta">
              <p v-if="!workspaceOptions.length" class="help-copy">no workspace presets configured yet.</p>
              <p v-else-if="!readyAgentCount" class="help-copy">no ready agents available yet.</p>
              <p v-else-if="workbenchStore.draft.continueFromRunId" class="help-copy">continuing run {{ workbenchStore.draft.continueFromRunId }}</p>
              <p v-else class="help-copy">{{ activeWorkspace?.rootPath ?? "workspace root pending" }}</p>
            </div>
            <div class="composer-actions">
              <NeonButton v-if="workbenchStore.draft.continueFromRunId" variant="secondary" ghost size="sm" @click="workbenchStore.clearContinuation()">clear thread</NeonButton>
              <NeonButton variant="secondary" ghost size="sm" :disabled="!inspectorWorkspaceId" @click="openWorkspaceFolder">open workspace</NeonButton>
              <NeonButton variant="primary" size="sm" :loading="workbenchStore.sending" :disabled="!workbenchStore.canSend || !selectedAgentReady" @click="submitRun">send</NeonButton>
            </div>
          </div>
        </div>
      </div>

      <aside class="summary-card glass-panel">
        <div class="panel-head">
          <div>
            <p class="panel-kicker">focus</p>
            <h2 class="panel-title">agent answer</h2>
          </div>
        </div>

        <div class="summary-block">
          <span class="block-label">latest reply</span>
          <strong>{{ latestAgentResponse?.body ? truncate(latestAgentResponse.body, 140) : "waiting for agent output" }}</strong>
          <span class="block-copy">{{ latestAgentResponse ? formatDate(latestAgentResponse.createdAt) : "agent messages surface here once the selected run responds." }}</span>
        </div>

        <div class="summary-block">
          <span class="block-label">workspace</span>
          <strong>{{ activeWorkspace?.name ?? "none" }}</strong>
          <span class="block-copy">{{ activeWorkspace?.rootPath ?? "select a workspace preset" }}</span>
          <div class="summary-actions">
            <button class="artifact-action" type="button" :disabled="!inspectorWorkspaceId" @click="openWorkspaceFolder">folder</button>
            <button class="artifact-action" type="button" :disabled="!inspectorWorkspaceId" @click="openWorkspaceTerminal">terminal</button>
          </div>
        </div>

        <div class="summary-block">
          <div class="artifact-drilldown__head">
            <div>
              <span class="block-label">onboarding</span>
              <strong>{{ activeProviderId }} readiness</strong>
            </div>
            <span class="ghost-pill">{{ onboardingChecklistSummary }}</span>
          </div>
          <span class="block-copy">{{ onboardingMessage }}</span>
          <div v-if="providerChecklist?.items?.length" class="checklist-list">
            <div v-for="item in providerChecklist.items" :key="item.key" class="checklist-row">
              <span class="status-chip" :class="item.ready ? 'status-chip--success' : 'status-chip--idle'">{{ item.ready ? "ready" : "todo" }}</span>
              <div>
                <strong>{{ item.label }}</strong>
                <span class="block-copy">{{ item.detail }}</span>
              </div>
            </div>
          </div>
          <div class="summary-actions">
            <button class="artifact-action" type="button" :disabled="!canRunOnboardingAction" @click="refreshProviderChecklist">checklist</button>
            <button class="artifact-action" type="button" :disabled="!canRunOnboardingAction" @click="checkProviderAuth">auth</button>
            <button class="artifact-action" type="button" :disabled="!canRunOnboardingAction" @click="smokeProvider">smoke</button>
            <button class="artifact-action" type="button" :disabled="!canRunOnboardingAction" @click="bootstrapProvider">bootstrap</button>
          </div>
        </div>

        <div class="summary-metrics">
          <div class="metric-card"><span class="block-label">artifacts</span><strong>{{ workbenchStore.activeArtifacts.length }}</strong></div>
          <div class="metric-card"><span class="block-label">events</span><strong>{{ workbenchStore.activeEvents.length }}</strong></div>
          <div class="metric-card"><span class="block-label">thread replies</span><strong>{{ threadReplies.length }}</strong></div>
        </div>
      </aside>

      <div class="runs-card glass-panel">
        <div class="panel-head">
          <div><p class="panel-kicker">runs</p><h2 class="panel-title">recent dispatches</h2></div>
        </div>

        <div v-if="workbenchStore.loading" class="empty-state">loading runs...</div>
        <div v-else-if="!workbenchStore.runs.length" class="empty-state">no runs yet. pick an agent + workspace, verify onboarding, then dispatch the first task.</div>
        <button v-for="run in workbenchStore.runs" :key="run.id" class="run-row" :class="{ active: run.id === workbenchStore.activeRunId }" @click="workbenchStore.selectRun(run.id)">
          <div class="run-row__head">
            <span class="run-row__title">{{ run.agentId }}</span>
            <span class="status-chip" :class="`status-chip--${statusTone(run.status)}`">{{ run.status }}</span>
          </div>
          <span class="run-row__status">{{ run.mode }}<template v-if="run.outcomeStatus"> · {{ run.outcomeStatus }}</template></span>
          <span v-if="run.parentRunId" class="run-row__thread">reply to {{ run.parentRunId }}</span>
          <span class="run-row__body">{{ truncate(run.prompt, 88) }}</span>
        </button>
      </div>

      <div class="timeline-card glass-panel">
        <div class="panel-head">
          <div><p class="panel-kicker">timeline</p><h2 class="panel-title">{{ workbenchStore.activeRun ? "run events" : "select a run" }}</h2></div>
          <div class="summary-actions">
            <NeonButton v-if="workbenchStore.activeRun && ['queued', 'working'].includes(workbenchStore.activeRun.status)" variant="danger" ghost size="sm" @click="workbenchStore.cancelRun(workbenchStore.activeRun.id)">cancel</NeonButton>
            <NeonButton v-if="workbenchStore.activeRun" variant="secondary" ghost size="sm" @click="workbenchStore.retryRun(workbenchStore.activeRun.id)">retry</NeonButton>
            <NeonButton v-if="workbenchStore.activeRun" variant="secondary" ghost size="sm" @click="continueRun(workbenchStore.activeRun)">continue</NeonButton>
          </div>
        </div>

        <div v-if="!workbenchStore.activeRun" class="empty-state">choose a run to inspect its timeline, artifacts and delivery state.</div>
        <div v-else class="timeline-list">
          <button v-if="workbenchStore.eventsPageStateByRun[workbenchStore.activeRun.id]?.hasMore" class="artifact-action timeline-more" type="button" @click="workbenchStore.loadOlderEvents(workbenchStore.activeRun.id)">load older events</button>
          <div class="thread-meta">
            <span v-if="workbenchStore.activeRun.parentRunId" class="ghost-pill">reply chain: {{ workbenchStore.activeRun.parentRunId }} → {{ workbenchStore.activeRun.id }}</span>
            <span v-if="threadReplies.length" class="ghost-pill">{{ threadReplies.length }} threaded repl{{ threadReplies.length === 1 ? "y" : "ies" }}</span>
          </div>
          <article v-for="event in workbenchStore.activeEvents" :key="event.id" class="timeline-item" :class="`timeline-item--${event.type}`">
            <div class="timeline-item__head">
              <span class="timeline-type">{{ formatEventType(event.type) }}</span>
              <span class="timeline-time">{{ formatDate(event.createdAt) }}</span>
            </div>
            <p class="timeline-body">{{ event.body }}</p>
          </article>
        </div>
      </div>

      <aside class="inspector-card glass-panel">
        <div class="panel-head">
          <div><p class="panel-kicker">inspector</p><h2 class="panel-title">selection</h2></div>
        </div>

        <div class="inspector-block"><span class="block-label">agent</span><strong>{{ activeAgent?.name ?? "none" }}</strong><span class="block-copy">{{ activeAgent?.role ?? "select an agent" }}</span></div>
        <div class="inspector-block"><span class="block-label">active run</span><strong>{{ workbenchStore.activeRun?.status ?? "idle" }}</strong><span class="block-copy">{{ workbenchStore.activeRun?.providerId ?? "provider pending" }}<template v-if="workbenchStore.activeRun?.outcomeStatus"> · {{ workbenchStore.activeRun.outcomeStatus }}</template></span></div>
        <div class="inspector-block">
          <span class="block-label">linked pm task</span>
          <strong>{{ activePmTask?.title ?? "none" }}</strong>
          <span class="block-copy">{{ activePmTask?.project ?? "optional delivery anchor" }}</span>
          <div class="summary-actions">
            <button v-if="activePmTask && workbenchStore.activeRun" class="artifact-action" type="button" @click="postPmUpdate">post run update</button>
          </div>
        </div>
        <div class="inspector-block"><span class="block-label">persona</span><strong>{{ activePersona?.name ?? "none" }}</strong><span class="block-copy">{{ activePersona?.description ?? "optional instruction overlay" }}</span></div>

        <div class="inspector-block">
          <div class="artifact-drilldown__head">
            <div><span class="block-label">result drilldown</span><strong>{{ selectedArtifact?.label ?? "select an artifact" }}</strong></div>
            <span class="ghost-pill">{{ selectedArtifact?.kind ?? "none" }}</span>
          </div>
          <div class="artifact-group-grid">
            <button v-for="artifact in workbenchStore.activeArtifacts" :key="artifact.id" class="artifact-chip" :class="{ active: artifact.id === selectedArtifactId }" type="button" @click="selectedArtifactId = artifact.id">
              {{ artifact.label }}
            </button>
          </div>
          <div v-if="selectedArtifact" class="artifact-drilldown">
            <div class="artifact-drilldown__meta">
              <span class="ghost-pill">{{ selectedArtifact.kind }}</span>
              <span class="block-copy">{{ formatDate(selectedArtifact.createdAt) }}</span>
            </div>
            <div class="summary-actions">
              <button class="artifact-action" type="button" @click="copyArtifact(selectedArtifact.value)">copy</button>
              <button v-if="selectedArtifact.kind === 'file' && workbenchStore.activeRun" class="artifact-action" type="button" @click="revealRunPath(selectedArtifact.label)">open file</button>
            </div>
            <pre class="artifact-value">{{ selectedArtifact.value }}</pre>
          </div>
          <span v-else class="block-copy">no structured artifacts yet</span>
        </div>
      </aside>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import ViewHero from "@/components/layout/ViewHero.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import type {
  DispatchRun,
  ProviderAuthState,
  ProviderProbeResult,
  ProviderSetupChecklistResult,
  RunArtifact,
  RunEvent,
  WorkspaceBootstrapResult,
  WorkspacePreset,
} from "@/interfaces";
import { deriveProviderIdFromAgentId } from "@/lib/providers";
import { buildWorkspaceConfigUpdate, createEmptyWorkspacePreset } from "@/lib/workspaces";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useWorkbenchStore } from "@/stores/useWorkbenchStore";

const agentStore = useAgentStore();
const configStore = useConfigStore();
const taskStore = useTaskStore();
const workbenchStore = useWorkbenchStore();
const workspaceManagerOpen = ref(false);
const workspaceSaving = ref(false);
const workspaceDrafts = ref<WorkspacePreset[]>([]);
const workspaceDefaultId = ref<string | null>(null);
const selectedArtifactId = ref<string | null>(null);
const providerChecklist = ref<ProviderSetupChecklistResult | null>(null);
const onboardingMessage = ref("select an agent and workspace to check provider readiness.");
const onboardingBusy = ref(false);

const dispatchableAgents = computed(() => agentStore.agents.filter((agent) => ["online", "idle", "working"].includes(agent.status)));
const readyAgentCount = computed(() => dispatchableAgents.value.length);
const workspaceOptions = computed(() => configStore.config.workspacePresets ?? []);
const personaOptions = computed(() => configStore.config.personaPresets ?? []);
const inspectorWorkspaceId = computed(() => workbenchStore.activeRun?.workspaceId ?? workbenchStore.draft.workspaceId);
const inspectorAgentId = computed(() => workbenchStore.activeRun?.agentId ?? workbenchStore.draft.agentId);
const inspectorPersonaId = computed(() => workbenchStore.activeRun?.personaId ?? workbenchStore.draft.personaId);
const activeWorkspace = computed(() => workspaceOptions.value.find((workspace) => workspace.id === inspectorWorkspaceId.value) ?? null);
const activeAgent = computed(() => agentStore.agents.find((agent) => agent.id === inspectorAgentId.value) ?? null);
const selectedAgentReady = computed(() => (activeAgent.value ? ["online", "idle", "working"].includes(activeAgent.value.status) : false));
const activePersona = computed(() => personaOptions.value.find((persona) => persona.id === inspectorPersonaId.value) ?? null);
const pmTaskOptions = computed(() => taskStore.tasks.slice(0, 50));
const activePmTask = computed(() => taskStore.tasks.find((task) => task.id === (workbenchStore.activeRun?.pmTaskId ?? workbenchStore.draft.pmTaskId)) ?? null);
const threadReplies = computed(() => (workbenchStore.activeRun ? workbenchStore.runs.filter((run) => run.parentRunId === workbenchStore.activeRun?.id) : []));
const latestAgentResponse = computed<RunEvent | null>(() => [...workbenchStore.activeEvents].reverse().find((event) => event.type === "agent_message") ?? null);
const activeRunStatusTone = computed(() => statusTone(workbenchStore.activeRun?.status ?? "draft"));
const selectedArtifact = computed(() => workbenchStore.activeArtifacts.find((artifact) => artifact.id === selectedArtifactId.value) ?? workbenchStore.activeArtifacts[0] ?? null);
const activeProviderId = computed(() => deriveProviderIdFromAgentId(inspectorAgentId.value));
const canRunOnboardingAction = computed(() => Boolean(inspectorWorkspaceId.value && inspectorAgentId.value && !onboardingBusy.value));
const onboardingChecklistSummary = computed(() => providerChecklist.value?.summary ?? "not checked");

watch(workspaceOptions, (workspaces) => {
  workspaceDrafts.value = workspaces.map((workspace) => ({ ...workspace, allowedProviders: [...workspace.allowedProviders], allowedAgents: [...workspace.allowedAgents] }));
  workspaceDefaultId.value = configStore.config.defaultWorkspaceId ?? workspaces[0]?.id ?? null;
}, { deep: true, immediate: true });

watch(() => workbenchStore.activeArtifacts, (artifacts) => {
  if (!artifacts.length) {
    selectedArtifactId.value = null;
    return;
  }
  if (!selectedArtifactId.value || !artifacts.some((artifact) => artifact.id === selectedArtifactId.value)) {
    selectedArtifactId.value = artifacts[0].id;
  }
}, { deep: true, immediate: true });

watch([inspectorWorkspaceId, inspectorAgentId], ([workspaceId, agentId]) => {
  providerChecklist.value = null;
  if (!workspaceId || !agentId) {
    onboardingMessage.value = "select an agent and workspace to check provider readiness.";
    return;
  }
  onboardingMessage.value = `ready to check ${deriveProviderIdFromAgentId(agentId)} in ${workspaceOptions.value.find((workspace) => workspace.id === workspaceId)?.name ?? workspaceId}.`;
  void refreshProviderChecklist();
}, { immediate: true });

onMounted(async () => {
  await Promise.all([agentStore.loadAgents(), taskStore.fetchTasks(), workbenchStore.loadRuns(), workbenchStore.setupLiveUpdates()]);
  if (!workbenchStore.draft.agentId && dispatchableAgents.value.length > 0) workbenchStore.draft.agentId = dispatchableAgents.value[0].id;
  if (!workbenchStore.draft.workspaceId && workspaceOptions.value.length > 0) {
    workbenchStore.draft.workspaceId = configStore.config.defaultWorkspaceId || workspaceOptions.value[0].id;
  }
});

function truncate(value: string, max = 72) {
  return value.length > max ? `${value.slice(0, max - 3)}...` : value;
}

function formatDate(value: string) {
  return new Date(value).toLocaleString();
}

function formatEventType(value: string) {
  return value.replaceAll("_", " ");
}

function statusTone(status: string) {
  if (status === "done") return "success";
  if (status === "working" || status === "queued") return "live";
  if (status === "error" || status === "cancelled") return "danger";
  return "idle";
}

function submitRun() {
  void workbenchStore.createRun();
}

function continueRun(run: DispatchRun) {
  workbenchStore.prepareContinuation(run);
}

function addWorkspacePreset() {
  workspaceDrafts.value = [...workspaceDrafts.value, createEmptyWorkspacePreset()];
}

function removeWorkspacePreset(index: number) {
  const removed = workspaceDrafts.value[index];
  workspaceDrafts.value = workspaceDrafts.value.filter((_, entryIndex) => entryIndex !== index);
  if (workspaceDefaultId.value === removed.id) workspaceDefaultId.value = workspaceDrafts.value[0]?.id ?? null;
}

async function pickWorkspaceRoot(index: number) {
  const selection = await open({ directory: true, multiple: false, title: "select workspace root" });
  if (typeof selection === "string") workspaceDrafts.value[index].rootPath = selection;
}

async function saveWorkspaceManager() {
  workspaceSaving.value = true;
  try {
    const nextConfig = buildWorkspaceConfigUpdate(configStore.config, workspaceDrafts.value, workspaceDefaultId.value);
    await configStore.saveConfig(nextConfig);
    if (!workbenchStore.draft.workspaceId || !nextConfig.workspacePresets.some((workspace) => workspace.id === workbenchStore.draft.workspaceId)) {
      workbenchStore.draft.workspaceId = nextConfig.defaultWorkspaceId ?? nextConfig.workspacePresets[0]?.id ?? "";
    }
  } finally {
    workspaceSaving.value = false;
  }
}

async function revealRunPath(relativePath?: string) {
  if (!workbenchStore.activeRun) return;
  await invoke("reveal_run_path", { runId: workbenchStore.activeRun.id, relativePath: relativePath ?? null });
}

async function openWorkspaceFolder() {
  if (!inspectorWorkspaceId.value) return;
  await invoke("open_workspace_folder", { workspaceId: inspectorWorkspaceId.value });
}

async function openWorkspaceTerminal() {
  if (!inspectorWorkspaceId.value) return;
  await invoke("open_workspace_terminal", { workspaceId: inspectorWorkspaceId.value });
}

async function refreshProviderChecklist() {
  if (!inspectorWorkspaceId.value || !inspectorAgentId.value) return;
  onboardingBusy.value = true;
  try {
    providerChecklist.value = await invoke<ProviderSetupChecklistResult>("check_provider_setup", {
      providerId: activeProviderId.value,
      workspaceId: inspectorWorkspaceId.value,
    });
    onboardingMessage.value = providerChecklist.value.summary;
  } catch (error) {
    onboardingMessage.value = String(error);
  } finally {
    onboardingBusy.value = false;
  }
}

async function checkProviderAuth() {
  if (!canRunOnboardingAction.value) return;
  onboardingBusy.value = true;
  try {
    const result = await invoke<ProviderAuthState>("get_provider_auth_state", { providerId: activeProviderId.value });
    onboardingMessage.value = result.summary;
  } catch (error) {
    onboardingMessage.value = String(error);
  } finally {
    onboardingBusy.value = false;
  }
}

async function smokeProvider() {
  if (!inspectorWorkspaceId.value || !canRunOnboardingAction.value) return;
  onboardingBusy.value = true;
  try {
    const result = await invoke<ProviderProbeResult>("smoke_test_provider_command", {
      providerId: activeProviderId.value,
      workspaceId: inspectorWorkspaceId.value,
    });
    onboardingMessage.value = result.summary;
  } catch (error) {
    onboardingMessage.value = String(error);
  } finally {
    onboardingBusy.value = false;
  }
}

async function bootstrapProvider() {
  if (!inspectorWorkspaceId.value || !canRunOnboardingAction.value) return;
  onboardingBusy.value = true;
  try {
    const result = await invoke<WorkspaceBootstrapResult>("bootstrap_provider_workspace", {
      providerId: activeProviderId.value,
      workspaceId: inspectorWorkspaceId.value,
      agentId: inspectorAgentId.value,
      overwrite: false,
    });
    onboardingMessage.value = result.summary;
    await refreshProviderChecklist();
  } catch (error) {
    onboardingMessage.value = String(error);
  } finally {
    onboardingBusy.value = false;
  }
}

async function copyArtifact(value: string) {
  await navigator.clipboard.writeText(value);
}

async function postPmUpdate() {
  if (!activePmTask.value || !workbenchStore.activeRun) return;
  const summaryArtifact = workbenchStore.activeArtifacts.find((artifact) => artifact.label === "pm comment")
    ?? workbenchStore.activeArtifacts.find((artifact) => artifact.label === "agent summary")
    ?? workbenchStore.activeArtifacts.find((artifact) => artifact.label === "result");
  await invoke("add_pm_comment", { taskId: activePmTask.value.id, content: buildPmComment(workbenchStore.activeRun, summaryArtifact) });
  await taskStore.fetchTasks();
}

function buildPmComment(run: DispatchRun, artifact?: RunArtifact) {
  const summary = artifact?.value?.trim() || "run completed";
  return [`umbra run ${run.id}`, `status: ${run.status}${run.outcomeStatus ? ` (${run.outcomeStatus})` : ""}`, `agent: ${run.agentId}`, `provider: ${run.providerId}`, `workspace: ${run.workspaceId}`, "", summary].join("\n");
}
</script>

<style scoped>
.workbench-view { display: flex; flex-direction: column; gap: 18px; }
.workbench-shell { display: grid; grid-template-columns: minmax(300px, 1.2fr) minmax(240px, 0.78fr) minmax(320px, 1.1fr); gap: 16px; align-items: start; }
.glass-panel { border-radius: var(--radius-2xl); border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent); background: linear-gradient(180deg, color-mix(in srgb, var(--glass-bg) 92%, transparent), color-mix(in srgb, var(--bg-secondary) 90%, transparent)); box-shadow: 0 18px 42px rgba(2, 6, 23, 0.16); padding: 18px; }
.composer-card { grid-column: 1 / span 2; }
.timeline-card { grid-column: 2 / span 2; }
.panel-head, .composer-intro, .timeline-item__head, .run-row__head, .artifact-drilldown__head, .workspace-manager__head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.panel-head { margin-bottom: 14px; }
.panel-kicker, .block-label, .timeline-type, .field-label { font-family: var(--font-mono); font-size: 10px; letter-spacing: 0.14em; text-transform: uppercase; color: var(--text-muted); }
.panel-title, .manager-title { margin: 2px 0 0; font-family: var(--font-display); font-size: 24px; line-height: 1; letter-spacing: 0.05em; }
.manager-title { font-size: 16px; }
.meta-pill, .ghost-pill, .status-chip { display: inline-flex; align-items: center; justify-content: center; padding: 6px 10px; border-radius: var(--radius-pill); font-family: var(--font-mono); font-size: 10px; letter-spacing: 0.08em; text-transform: uppercase; }
.meta-pill, .ghost-pill { border: none; background: color-mix(in srgb, var(--accent) 6%, var(--bg-surface)); color: var(--text-secondary); }
.status-chip { border: none; }
.status-chip--success { background: rgba(34, 197, 94, 0.12); color: #22c55e; }
.status-chip--live { background: var(--accent-dim); color: var(--accent); }
.status-chip--danger { background: rgba(239, 68, 68, 0.10); color: var(--accent-error); }
.status-chip--idle { background: color-mix(in srgb, var(--bg-surface) 80%, transparent); color: var(--text-secondary); }
.composer-frame, .workspace-manager, .workspace-list, .summary-card, .inspector-card, .timeline-list, .artifact-drilldown { display: flex; flex-direction: column; gap: 14px; }
.composer-copy, .composer-pills, .summary-actions, .composer-actions, .thread-meta, .artifact-group-grid, .artifact-drilldown__meta, .workspace-row__head, .workspace-row__body { display: flex; gap: 8px; flex-wrap: wrap; }
.composer-copy { flex-direction: column; gap: 4px; }
.glass-input { width: 100%; border-radius: var(--radius-lg); border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent); background: color-mix(in srgb, var(--bg-secondary) 94%, transparent); color: var(--text-primary); padding: 12px 14px; }
.prompt-input { resize: vertical; min-height: 180px; }
.composer-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
.form-field { display: flex; flex-direction: column; gap: 6px; }
.form-field--wide { grid-column: span 2; }
.workspace-manager, .summary-block, .inspector-block, .metric-card, .artifact-drilldown, .workspace-row, .timeline-item, .run-row { border-radius: var(--radius-xl); border: 1px solid color-mix(in srgb, var(--glass-border) 82%, transparent); background: color-mix(in srgb, var(--bg-secondary) 90%, transparent); padding: 12px; }
.workspace-row { gap: 10px; }
.workspace-row__head input:first-child, .workspace-row__body input { flex: 1; }
.toggle-pill { display: inline-flex; align-items: center; gap: 6px; color: var(--text-secondary); font-size: 12px; }
.composer-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.composer-meta { display: flex; flex-direction: column; gap: 4px; }
.help-copy, .block-copy, .timeline-time, .run-row__body, .run-row__status, .empty-state { color: var(--text-muted); font-size: 12px; line-height: 1.55; }
.summary-block strong, .inspector-block strong, .metric-card strong { display: block; margin-top: 4px; font-size: 15px; line-height: 1.45; }
.summary-metrics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
.checklist-list { display: flex; flex-direction: column; gap: 8px; }
.checklist-row { display: grid; grid-template-columns: auto 1fr; gap: 10px; align-items: start; }
.runs-card, .timeline-card { min-height: 420px; }
.run-row { width: 100%; display: flex; flex-direction: column; align-items: flex-start; gap: 6px; color: inherit; text-align: left; cursor: pointer; }
.run-row + .run-row { margin-top: 10px; }
.run-row.active { border-color: color-mix(in srgb, var(--accent) 24%, transparent); background: color-mix(in srgb, var(--accent) 10%, transparent); }
.run-row__title { font-family: var(--font-display); font-size: 16px; letter-spacing: 0.04em; text-transform: uppercase; }
.run-row__thread { color: var(--text-muted); font-family: var(--font-mono); font-size: 10px; letter-spacing: 0.08em; text-transform: uppercase; }
.timeline-more { align-self: flex-start; }
.timeline-item--system { border-color: color-mix(in srgb, var(--accent) 18%, transparent); }
.timeline-item--agent_message { border-color: color-mix(in srgb, var(--accent-success) 22%, transparent); }
.timeline-item--stdout { border-color: color-mix(in srgb, var(--accent-secondary) 18%, transparent); }
.timeline-item--stderr { border-color: rgba(239, 68, 68, 0.26); background: rgba(239, 68, 68, 0.06); }
.timeline-body, .artifact-value { margin: 8px 0 0; white-space: pre-wrap; line-height: 1.65; }
.artifact-chip, .artifact-action { border: none; background: color-mix(in srgb, var(--accent) 6%, var(--bg-surface)); color: var(--text-primary); border-radius: var(--radius-pill); padding: 6px 10px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; cursor: pointer; }
.artifact-chip.active { background: color-mix(in srgb, var(--accent) 14%, var(--bg-surface)); color: var(--accent); }
.artifact-value { color: var(--text-secondary); font-size: 12px; word-break: break-word; font-family: inherit; }
.error-card { border-radius: var(--radius-xl); border: 1px solid rgba(239, 68, 68, 0.24); background: rgba(239, 68, 68, 0.08); color: var(--accent-error); padding: 12px 14px; }
@media (max-width: 1380px) { .workbench-shell { grid-template-columns: 1fr 1fr; } .composer-card, .timeline-card { grid-column: auto; } }
@media (max-width: 960px) { .workbench-shell, .composer-grid, .summary-metrics { grid-template-columns: 1fr; } .composer-footer, .workspace-row__head, .workspace-row__body, .workspace-manager__head { flex-direction: column; align-items: stretch; } }
</style>
