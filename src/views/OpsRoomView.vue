<template>
  <div class="ops-room-view">
    <ViewHero kicker="ops room" title="Ops Room" :subtitle="`${opsStore.channels.length} channels · ${opsStore.activeJobs.length} jobs · ${escalations.length} escalations`">
      <template #meta>
        <span class="view-hero-pill">{{ activeWorkspace?.name ?? "workspace pending" }}</span>
        <span class="view-hero-pill">{{ liveRunCount }} live runs</span>
      </template>
    </ViewHero>
    <div v-if="opsStore.error" class="error-card">{{ opsStore.error }}</div>
    <section class="ops-shell">
      <aside class="glass-panel stack">
        <div class="row">
          <input v-model="newChannelName" class="glass-input" placeholder="channel name" />
          <select v-model="newChannelWorkspaceId" class="glass-input">
            <option value="">workspace</option>
            <option v-for="workspace in workspaceOptions" :key="workspace.id" :value="workspace.id">{{ workspace.name }}</option>
          </select>
          <NeonButton size="sm" variant="secondary" @click="createChannel">create</NeonButton>
        </div>
        <button v-for="channel in opsStore.channels" :key="channel.id" class="card-button channel-button" :class="{ active: channel.id === opsStore.activeChannelId }" type="button" @click="opsStore.selectChannel(channel.id)">
          <div class="row between full-width">
            <strong>{{ channel.name }}</strong>
            <span class="status-chip" :class="channelHealthTone(channel.id)">{{ channelHealthLabel(channel.id) }}</span>
          </div>
          <span>{{ workspaceName(channel.workspaceId) }}</span>
          <div class="row">
            <span class="chip">{{ channelRunCount(channel.id) }} runs</span>
            <span class="chip">{{ channelEscalationCount(channel.id) }} escalations</span>
          </div>
        </button>
      </aside>

      <main class="glass-panel stack">
        <div class="row between">
          <div>
            <p class="eyebrow">channel</p>
            <h2>{{ opsStore.activeChannel?.name ?? "select a channel" }}</h2>
            <p class="muted">{{ activeWorkspace?.rootPath ?? "select a workspace-backed channel to start routing work" }}</p>
          </div>
          <div class="row">
            <button class="chip" type="button" :disabled="!activeWorkspace" @click="openWorkspaceFolder">folder</button>
            <button class="chip" type="button" :disabled="!activeWorkspace" @click="openWorkspaceTerminal">terminal</button>
          </div>
        </div>

        <div class="presence-strip">
          <article v-for="presence in channelPresence" :key="presence.agent.id" class="presence-card">
            <div class="row between full-width">
              <strong>{{ presence.agent.name }}</strong>
              <span class="status-chip" :class="presence.tone">{{ presence.label }}</span>
            </div>
            <p class="muted">{{ presence.providerLabel }} · {{ presence.agent.role || "no role" }}</p>
            <div class="row">
              <span class="chip">{{ presence.runCount }} channel runs</span>
              <span v-if="presence.blockedCount" class="chip chip-alert">{{ presence.blockedCount }} blocked</span>
              <span v-if="presence.needsInputCount" class="chip chip-alert">{{ presence.needsInputCount }} needs input</span>
            </div>
          </article>
        </div>

        <div v-if="workspaceManagerOpen" class="stack card">
          <div v-for="(workspace, index) in workspaceDrafts" :key="workspace.id" class="stack">
            <div class="row">
              <input v-model="workspace.name" class="glass-input" placeholder="workspace name" />
              <input v-model="workspace.rootPath" class="glass-input" placeholder="absolute path" />
            </div>
            <div class="row">
              <label class="chip toggle"><input v-model="workspace.writable" type="checkbox" /> writable</label>
              <label class="chip toggle"><input :checked="workspaceDefaultId === workspace.id" type="radio" name="ops-default" @change="workspaceDefaultId = workspace.id" /> default</label>
              <button class="chip" type="button" @click="pickWorkspaceRoot(index)">pick</button>
              <button class="chip" type="button" @click="removeWorkspacePreset(index)">delete</button>
            </div>
          </div>
          <div class="row">
            <button class="chip" type="button" @click="addWorkspacePreset">add workspace</button>
            <button class="chip" type="button" :disabled="workspaceSaving" @click="saveWorkspaceManager">{{ workspaceSaving ? "saving..." : "save workspaces" }}</button>
            <button class="chip" type="button" @click="workspaceManagerOpen = false">close</button>
          </div>
        </div>

        <div v-if="opsStore.activeChannel" class="card stack">
          <div class="row between">
            <div>
              <p class="eyebrow">workspace</p>
              <strong>{{ activeWorkspace?.name ?? opsStore.activeChannel.workspaceId }}</strong>
              <p class="muted">{{ activeWorkspace?.rootPath ?? "workspace root pending" }}</p>
            </div>
            <button class="chip" type="button" @click="workspaceManagerOpen = !workspaceManagerOpen">{{ workspaceManagerOpen ? "hide manager" : "manage workspaces" }}</button>
          </div>
          <div v-if="replyTargetMessage" class="card stack">
            <p class="eyebrow">replying to</p>
            <strong>{{ replyTargetMessage.authorLabel ?? replyTargetMessage.agentId ?? "room" }}</strong>
            <p class="muted">{{ truncate(replyTargetMessage.body, 140) }}</p>
            <button class="chip" type="button" @click="opsStore.setReplyTarget(null)">clear reply</button>
          </div>
          <textarea v-model="opsStore.draft.body" class="glass-input" rows="5" placeholder="message the room or reply to a thread..." />
          <div class="row">
            <select v-model="opsStore.draft.agentId" class="glass-input">
              <option :value="null">auto route</option>
              <option v-for="agent in dispatchableAgents" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
            </select>
            <NeonButton size="sm" variant="primary" :loading="opsStore.sending" :disabled="!opsStore.draft.body.trim()" @click="sendMessage">send</NeonButton>
          </div>
        </div>

        <div class="stack message-list">
          <button v-if="opsStore.activeChannelId && opsStore.messagePageStateByChannel[opsStore.activeChannelId]?.hasMore" class="chip" type="button" @click="opsStore.loadOlderMessages(opsStore.activeChannelId)">load older messages</button>
          <article v-for="message in opsStore.activeMessages" :key="message.id" class="card message-card" :class="{ reply: Boolean(message.parentMessageId) }">
            <div class="row between">
              <strong>{{ message.authorLabel ?? message.agentId ?? "room" }}</strong>
              <span class="muted">{{ formatDate(message.createdAt) }}</span>
            </div>
            <p v-if="message.parentMessageId" class="muted">reply to {{ parentMessageLabel(message.parentMessageId) }}</p>
            <p class="message-body">{{ message.body }}</p>
            <div class="row">
              <span class="chip">{{ message.kind }}</span>
              <span v-if="message.runId" class="status-chip" :class="runTone(messageRun(message.runId))">{{ runLabel(messageRun(message.runId)) }}</span>
              <span v-if="message.jobId" class="chip">job {{ message.jobId }}</span>
              <span v-if="replyCount(message.id)" class="chip">{{ replyCount(message.id) }} replies</span>
            </div>
            <div class="row">
              <button class="chip" type="button" @click="opsStore.setReplyTarget(message.id)">reply</button>
              <button v-if="message.kind === 'user'" class="chip" type="button" @click="primeJob(message.id)">make job</button>
            </div>
          </article>
        </div>
      </main>

      <aside class="glass-panel stack">
        <div class="card stack rail-section">
          <p class="eyebrow">workspace</p>
          <strong>{{ activeWorkspace?.name ?? "none" }}</strong>
          <span class="muted">{{ activeWorkspace?.rootPath ?? "select a channel workspace" }}</span>
          <div class="row">
            <span class="chip">{{ liveRunCount }} live runs</span>
            <span class="chip">{{ blockedCount }} blocked</span>
            <span class="chip">{{ needsInputCount }} needs input</span>
          </div>
        </div>

        <div class="card stack rail-section">
          <div class="row between">
            <div><p class="eyebrow">escalations</p><strong>blocked + needs input</strong></div>
            <span class="status-chip" :class="escalations.length ? 'status-chip--warn' : 'status-chip--success'">{{ escalations.length ? `${escalations.length} open` : "clear" }}</span>
          </div>
          <div v-if="escalations.length" class="stack">
            <div v-for="item in escalations" :key="item.run.id" class="card escalation-card">
              <div class="row between full-width">
                <strong>{{ item.title }}</strong>
                <span class="status-chip" :class="runTone(item.run)">{{ runLabel(item.run) }}</span>
              </div>
              <p class="muted">{{ item.agentLabel }} · {{ item.providerLabel }}</p>
              <p class="muted">{{ item.summary }}</p>
              <div class="row">
                <span v-if="item.jobLabel" class="chip">{{ item.jobLabel }}</span>
                <span class="chip">{{ formatDate(item.run.updatedAt) }}</span>
              </div>
            </div>
          </div>
          <span v-else class="muted">no blocked or needs-input runs in this channel.</span>
        </div>

        <div class="card stack rail-section">
          <div class="row between">
            <div><p class="eyebrow">onboarding</p><strong>{{ activeProviderId }} readiness</strong></div>
            <span class="ghost-pill">{{ onboardingChecklistSummary }}</span>
          </div>
          <span class="muted">{{ onboardingMessage }}</span>
          <div v-if="providerChecklist?.items?.length" class="stack">
            <div v-for="item in providerChecklist.items" :key="item.key" class="card">
              <div class="row between full-width">
                <strong>{{ item.label }}</strong>
                <span class="status-chip" :class="item.ready ? 'status-chip--success' : 'status-chip--neutral'">{{ item.ready ? "ready" : "todo" }}</span>
              </div>
              <p class="muted">{{ item.detail }}</p>
            </div>
          </div>
          <div class="row">
            <button class="chip" type="button" :disabled="!canRunOnboardingAction" @click="refreshProviderChecklist">checklist</button>
            <button class="chip" type="button" :disabled="!canRunOnboardingAction" @click="checkProviderAuth">auth</button>
            <button class="chip" type="button" :disabled="!canRunOnboardingAction" @click="smokeProvider">smoke</button>
            <button class="chip" type="button" :disabled="!canRunOnboardingAction" @click="bootstrapProvider">bootstrap</button>
          </div>
        </div>

        <div class="card stack rail-section">
          <p class="eyebrow">jobs</p>
          <select v-model="jobSourceMessageId" class="glass-input"><option value="">source message</option><option v-for="message in userMessages" :key="message.id" :value="message.id">{{ truncate(message.body, 48) }}</option></select>
          <input v-model="jobTitle" class="glass-input" placeholder="job title" />
          <textarea v-model="jobSummary" class="glass-input" rows="3" placeholder="job summary" />
          <select v-model="jobAgentId" class="glass-input"><option value="">agent</option><option v-for="agent in dispatchableAgents" :key="agent.id" :value="agent.id">{{ agent.name }}</option></select>
          <NeonButton size="sm" variant="secondary" :disabled="!canCreateJob" @click="createJob">create job</NeonButton>
          <div class="simple-list stack">
            <button v-for="job in opsStore.activeJobs" :key="job.id" class="card-button" :class="{ active: job.id === selectedJobId }" type="button" @click="selectJob(job.id)">
              <div class="row between full-width">
                <strong>{{ job.title }}</strong>
                <span class="status-chip" :class="jobTone(job)">{{ job.status }}</span>
              </div>
              <span>{{ job.agentId }} · {{ runLabel(jobRun(job.runId)) }}</span>
            </button>
          </div>
        </div>

        <div class="card stack rail-section">
          <p class="eyebrow">result drilldown</p>
          <template v-if="selectedJob">
            <strong>{{ selectedJob.title }}</strong>
            <span class="muted">{{ selectedJobRun?.status ?? selectedJob.status }}</span>
            <span v-if="selectedJobRun?.outcomeStatus" class="chip">{{ selectedJobRun.outcomeStatus }}</span>
            <p class="muted">{{ selectedJobSourceMessage ? truncate(selectedJobSourceMessage.body, 160) : selectedJob.summary || "no summary" }}</p>
            <div v-if="selectedJobArtifacts.length" class="stack">
              <div v-for="artifact in selectedJobArtifacts" :key="artifact.id" class="card">
                <div class="row between"><strong>{{ artifact.label }}</strong><span class="chip">{{ artifact.kind }}</span></div>
                <pre class="artifact-value">{{ artifact.value }}</pre>
              </div>
            </div>
            <span v-else class="muted">{{ selectedJobRun ? "no structured artifacts yet" : "run pending" }}</span>
          </template>
          <span v-else class="muted">select a job to inspect linked run output.</span>
        </div>

        <div class="card stack rail-section">
          <p class="eyebrow">approvals</p>
          <div v-if="opsStore.activeApprovals.length" class="stack">
            <div v-for="approval in opsStore.activeApprovals" :key="approval.id" class="card">
              <strong>@{{ approval.agentId }}</strong>
              <p class="muted">{{ approval.reason }}</p>
              <div class="row">
                <button class="chip" type="button" @click="resolveApproval(approval.id, true)">approve</button>
                <button class="chip" type="button" @click="resolveApproval(approval.id, false)">reject</button>
              </div>
            </div>
          </div>
          <span v-else class="muted">no pending approvals.</span>
        </div>

        <div class="card stack rail-section">
          <p class="eyebrow">rules</p>
          <input v-model="ruleName" class="glass-input" placeholder="rule name" />
          <input v-model="rulePattern" class="glass-input" placeholder="pattern or *" />
          <select v-model="ruleTargetAgentId" class="glass-input">
            <option value="">all agents</option>
            <option v-for="agent in dispatchableAgents" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
          </select>
          <label class="chip toggle"><input v-model="ruleRequiresHumanGate" type="checkbox" /> human gate</label>
          <button class="chip" type="button" :disabled="!ruleName.trim() || !rulePattern.trim()" @click="saveRule">save rule</button>
          <div v-for="rule in opsStore.rules" :key="rule.id" class="card">
            <strong>{{ rule.name }}</strong>
            <p class="muted">{{ rule.pattern }}</p>
          </div>
        </div>

        <div class="card stack rail-section">
          <p class="eyebrow">sessions</p>
          <input v-model="templateName" class="glass-input" placeholder="template name" />
          <select v-model="templateWorkspaceId" class="glass-input">
            <option value="">workspace</option>
            <option v-for="workspace in workspaceOptions" :key="workspace.id" :value="workspace.id">{{ workspace.name }}</option>
          </select>
          <select v-model="templateAgentIds" class="glass-input" multiple size="4">
            <option v-for="agent in dispatchableAgents" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
          </select>
          <label class="chip toggle"><input v-model="templateAutoAdvance" type="checkbox" /> auto advance</label>
          <label class="chip toggle"><input v-model="templateRequiresHumanGate" type="checkbox" /> human gate</label>
          <button class="chip" type="button" :disabled="!templateName.trim() || !templateWorkspaceId || !templateAgentIds.length" @click="saveTemplate">save template</button>
          <select v-model="startTemplateId" class="glass-input">
            <option value="">start template</option>
            <option v-for="template in channelTemplates" :key="template.id" :value="template.id">{{ template.name }}</option>
          </select>
          <button class="chip" type="button" :disabled="!startTemplateId || !opsStore.activeChannel" @click="startSession">start session</button>
          <div v-for="session in opsStore.activeSessions" :key="session.id" class="card">
            <strong>{{ session.state }}</strong>
            <p class="muted">{{ templateNameFor(session.templateId) }} · turn {{ session.currentTurnIndex + 1 }}</p>
            <div class="row">
              <button class="chip" type="button" @click="opsStore.advanceSession(session.id)">advance</button>
              <button class="chip" type="button" @click="opsStore.pauseSession(session.id, session.state === 'active')">{{ session.state === "active" ? "pause" : "resume" }}</button>
            </div>
          </div>
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
  Agent,
  DispatchRun,
  OpsJob,
  ProviderAuthState,
  ProviderProbeResult,
  ProviderSetupChecklistResult,
  RunArtifact,
  WorkspaceBootstrapResult,
  WorkspacePreset,
} from "@/interfaces";
import { deriveProviderIdFromAgentId } from "@/lib/providers";
import { buildWorkspaceConfigUpdate, createEmptyWorkspacePreset } from "@/lib/workspaces";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import { useOpsStore } from "@/stores/useOpsStore";
import { useTaskStore } from "@/stores/useTaskStore";

const opsStore = useOpsStore();
const agentStore = useAgentStore();
const configStore = useConfigStore();
const taskStore = useTaskStore();
const newChannelName = ref("");
const newChannelWorkspaceId = ref("");
const jobSourceMessageId = ref("");
const jobTitle = ref("");
const jobSummary = ref("");
const jobAgentId = ref("");
const ruleName = ref("");
const rulePattern = ref("");
const ruleTargetAgentId = ref("");
const ruleRequiresHumanGate = ref(true);
const templateName = ref("");
const templateWorkspaceId = ref("");
const templateAgentIds = ref<string[]>([]);
const templateAutoAdvance = ref(true);
const templateRequiresHumanGate = ref(false);
const startTemplateId = ref("");
const workspaceManagerOpen = ref(false);
const workspaceSaving = ref(false);
const workspaceDrafts = ref<WorkspacePreset[]>([]);
const workspaceDefaultId = ref<string | null>(null);
const selectedJobId = ref<string | null>(null);
const selectedJobRun = ref<DispatchRun | null>(null);
const selectedJobArtifacts = ref<RunArtifact[]>([]);
const providerChecklist = ref<ProviderSetupChecklistResult | null>(null);
const onboardingMessage = ref("select an agent target in this channel to verify provider readiness.");
const onboardingBusy = ref(false);

const workspaceOptions = computed(() => configStore.config.workspacePresets ?? []);
const dispatchableAgents = computed(() => agentStore.agents.filter((agent) => ["online", "idle", "working"].includes(agent.status)));
const activeWorkspace = computed(() => workspaceOptions.value.find((workspace) => workspace.id === opsStore.activeChannel?.workspaceId) ?? null);
const activeMessagesById = computed(() => Object.fromEntries(opsStore.activeMessages.map((message) => [message.id, message])));
const activeRunsById = computed(() => Object.fromEntries(opsStore.activeRuns.map((run) => [run.id, run])));
const replyTargetMessage = computed(() => (opsStore.draft.parentMessageId ? activeMessagesById.value[opsStore.draft.parentMessageId] ?? null : null));
const userMessages = computed(() => opsStore.activeMessages.filter((message) => message.kind === "user"));
const channelTemplates = computed(() => opsStore.sessionTemplates.filter((template) => template.workspaceId === (opsStore.activeChannel?.workspaceId ?? "")));
const canCreateJob = computed(() => !!opsStore.activeChannel && !!jobSourceMessageId.value && !!jobTitle.value.trim() && !!jobAgentId.value);
const selectedJob = computed(() => opsStore.activeJobs.find((job) => job.id === selectedJobId.value) ?? null);
const selectedJobSourceMessage = computed(() => (selectedJob.value ? activeMessagesById.value[selectedJob.value.sourceMessageId] ?? null : null));
const responseAgentId = computed(() => opsStore.draft.agentId ?? opsStore.activeChannel?.defaultAgentId ?? dispatchableAgents.value[0]?.id ?? "");
const activeProviderId = computed(() => deriveProviderIdFromAgentId(responseAgentId.value));
const canRunOnboardingAction = computed(() => Boolean(activeWorkspace.value?.id && responseAgentId.value && !onboardingBusy.value));
const onboardingChecklistSummary = computed(() => providerChecklist.value?.summary ?? "not checked");
const liveRunCount = computed(() => opsStore.activeRuns.filter((run) => ["queued", "working"].includes(run.status)).length);
const blockedCount = computed(() => opsStore.activeRuns.filter((run) => run.outcomeStatus === "blocked").length);
const needsInputCount = computed(() => opsStore.activeRuns.filter((run) => run.outcomeStatus === "needs_input").length);
const escalations = computed(() => opsStore.activeRuns.filter((run) => run.outcomeStatus === "blocked" || run.outcomeStatus === "needs_input").map((run) => {
  const sourceMessage = run.sourceMessageId ? activeMessagesById.value[run.sourceMessageId] : null;
  const relatedJob = run.jobId ? opsStore.activeJobs.find((job) => job.id === run.jobId) ?? null : null;
  return { run, title: relatedJob?.title ?? sourceMessage?.body ?? run.prompt, summary: sourceMessage?.body ?? run.prompt, agentLabel: run.agentId, providerLabel: run.providerId, jobLabel: relatedJob ? `job · ${relatedJob.status}` : null };
}));
const channelPresence = computed(() => dispatchableAgents.value.map((agent) => {
  const runs = opsStore.activeRuns.filter((run) => run.agentId === agent.id);
  const working = runs.some((run) => ["queued", "working"].includes(run.status));
  const blocked = runs.filter((run) => run.outcomeStatus === "blocked").length;
  const needsInput = runs.filter((run) => run.outcomeStatus === "needs_input").length;
  const tone = blocked || needsInput ? "status-chip--warn" : presenceTone(agent.status, working);
  const label = blocked ? "blocked" : needsInput ? "needs input" : working ? "responding" : agent.status;
  return { agent, providerLabel: deriveProviderIdFromAgentId(agent.id), runCount: runs.length, blockedCount: blocked, needsInputCount: needsInput, tone, label };
}));

watch(workspaceOptions, (workspaces) => {
  workspaceDrafts.value = workspaces.map((workspace) => ({ ...workspace, allowedProviders: [...workspace.allowedProviders], allowedAgents: [...workspace.allowedAgents] }));
  workspaceDefaultId.value = configStore.config.defaultWorkspaceId ?? workspaces[0]?.id ?? null;
}, { deep: true, immediate: true });

watch(() => opsStore.activeJobs, (jobs) => {
  if (!jobs.length) { selectedJobId.value = null; return; }
  if (!selectedJobId.value || !jobs.some((job) => job.id === selectedJobId.value)) selectedJobId.value = jobs[0].id;
}, { deep: true, immediate: true });

watch(selectedJobId, () => { void loadSelectedJobDetail(); }, { immediate: true });
watch([() => activeWorkspace.value?.id ?? "", responseAgentId], ([workspaceId, agentId]) => {
  providerChecklist.value = null;
  if (!workspaceId || !agentId) {
    onboardingMessage.value = "select an agent target in this channel to verify provider readiness.";
    return;
  }
  onboardingMessage.value = `ready to check ${deriveProviderIdFromAgentId(agentId)} in ${activeWorkspace.value?.name ?? workspaceId}.`;
  void refreshProviderChecklist();
}, { immediate: true });

onMounted(async () => {
  await Promise.all([agentStore.loadAgents(), agentStore.setupLiveUpdates(), taskStore.fetchTasks(), opsStore.loadChannels(), opsStore.setupLiveUpdates()]);
  if (!newChannelWorkspaceId.value && workspaceOptions.value.length > 0) newChannelWorkspaceId.value = configStore.config.defaultWorkspaceId || workspaceOptions.value[0].id;
  if (!templateWorkspaceId.value && workspaceOptions.value.length > 0) templateWorkspaceId.value = configStore.config.defaultWorkspaceId || workspaceOptions.value[0].id;
});

function workspaceName(workspaceId: string) { return workspaceOptions.value.find((workspace) => workspace.id === workspaceId)?.name ?? workspaceId; }
function formatDate(value: string) { return new Date(value).toLocaleString(); }
function truncate(value: string, max = 64) { return value.length > max ? `${value.slice(0, max - 3)}...` : value; }
function replyCount(messageId: string) { return opsStore.activeMessages.filter((message) => message.parentMessageId === messageId).length; }
function parentMessageLabel(parentMessageId: string) { const parent = activeMessagesById.value[parentMessageId]; return parent ? `${parent.authorLabel ?? parent.agentId ?? "room"} · ${truncate(parent.body, 48)}` : parentMessageId; }
function templateNameFor(templateId: string) { return opsStore.sessionTemplates.find((template) => template.id === templateId)?.name ?? templateId; }
function messageRun(runId?: string | null) { return runId ? activeRunsById.value[runId] ?? null : null; }
function jobRun(runId?: string | null) { return runId ? activeRunsById.value[runId] ?? null : null; }

function runLabel(run: DispatchRun | null) {
  if (!run) return "pending";
  if (run.outcomeStatus === "blocked") return "blocked";
  if (run.outcomeStatus === "needs_input") return "needs input";
  if (run.status === "working") return "working";
  if (run.status === "queued") return "queued";
  if (run.status === "done") return "done";
  return run.status;
}

function runTone(run: DispatchRun | null) {
  if (!run) return "status-chip--neutral";
  if (run.outcomeStatus === "blocked" || run.outcomeStatus === "needs_input") return "status-chip--warn";
  if (run.status === "done") return "status-chip--success";
  if (run.status === "error" || run.status === "cancelled") return "status-chip--danger";
  return "status-chip--info";
}

function jobTone(job: OpsJob) {
  if (job.status === "done") return "status-chip--success";
  if (job.status === "blocked") return "status-chip--warn";
  if (job.status === "running") return "status-chip--info";
  return "status-chip--neutral";
}

function channelRunCount(channelId: string) { return opsStore.activeChannelId === channelId ? opsStore.activeRuns.length : 0; }
function channelEscalationCount(channelId: string) { return opsStore.activeChannelId === channelId ? escalations.value.length : 0; }
function channelHealthTone(channelId: string) { if (opsStore.activeChannelId !== channelId) return "status-chip--neutral"; if (escalations.value.length) return "status-chip--warn"; if (liveRunCount.value) return "status-chip--info"; return "status-chip--success"; }
function channelHealthLabel(channelId: string) { if (opsStore.activeChannelId !== channelId) return "ready"; if (escalations.value.length) return "attention"; if (liveRunCount.value) return "live"; return "nominal"; }
function presenceTone(status: Agent["status"], working: boolean) { if (working || status === "working") return "status-chip--info"; if (status === "online" || status === "idle") return "status-chip--success"; if (status === "error") return "status-chip--danger"; return "status-chip--neutral"; }
async function createChannel() { const channel = await opsStore.createChannel({ name: newChannelName.value, description: "", workspaceId: newChannelWorkspaceId.value, defaultAgentId: null }); if (channel) newChannelName.value = ""; }
function sendMessage() { void opsStore.sendMessage(); }
function primeJob(messageId: string) { jobSourceMessageId.value = messageId; const message = opsStore.activeMessages.find((entry) => entry.id === messageId); if (message && !jobTitle.value.trim()) { jobTitle.value = truncate(message.body, 48); jobSummary.value = message.body; jobAgentId.value = message.agentId ?? opsStore.activeChannel?.defaultAgentId ?? ""; } }
async function createJob() { if (!opsStore.activeChannel) return; const job = await opsStore.createJob({ sourceMessageId: jobSourceMessageId.value, title: jobTitle.value, summary: jobSummary.value, agentId: jobAgentId.value, workspaceId: opsStore.activeChannel.workspaceId, pmTaskId: null }); if (!job) return; selectedJobId.value = job.id; jobSourceMessageId.value = ""; jobTitle.value = ""; jobSummary.value = ""; jobAgentId.value = ""; }
function resolveApproval(approvalId: string, approved: boolean) { void opsStore.resolveApproval(approvalId, approved); }
async function saveRule() { const rule = await opsStore.saveRule({ name: ruleName.value, pattern: rulePattern.value, targetAgentId: ruleTargetAgentId.value || null, workspaceId: opsStore.activeChannel?.workspaceId ?? null, enabled: true, requiresHumanGate: ruleRequiresHumanGate.value }); if (!rule) return; ruleName.value = ""; rulePattern.value = ""; ruleTargetAgentId.value = ""; ruleRequiresHumanGate.value = true; }
async function saveTemplate() { const template = await opsStore.saveSessionTemplate({ name: templateName.value, workspaceId: templateWorkspaceId.value, agentIds: templateAgentIds.value, autoAdvance: templateAutoAdvance.value, requiresHumanGate: templateRequiresHumanGate.value }); if (!template) return; templateName.value = ""; templateAgentIds.value = []; }
function startSession() { void opsStore.startSession(startTemplateId.value); }
function selectJob(jobId: string) { selectedJobId.value = jobId; }
async function loadSelectedJobDetail() { if (!selectedJob.value?.runId) { selectedJobRun.value = null; selectedJobArtifacts.value = []; return; } selectedJobRun.value = await invoke<DispatchRun | null>("get_dispatch_run", { runId: selectedJob.value.runId }); selectedJobArtifacts.value = await invoke<RunArtifact[]>("list_run_artifacts", { runId: selectedJob.value.runId }); }
function addWorkspacePreset() { workspaceDrafts.value = [...workspaceDrafts.value, createEmptyWorkspacePreset()]; }
function removeWorkspacePreset(index: number) { const removed = workspaceDrafts.value[index]; workspaceDrafts.value = workspaceDrafts.value.filter((_, entryIndex) => entryIndex !== index); if (workspaceDefaultId.value === removed.id) workspaceDefaultId.value = workspaceDrafts.value[0]?.id ?? null; }
async function pickWorkspaceRoot(index: number) { const selection = await open({ directory: true, multiple: false, title: "select workspace root" }); if (typeof selection === "string") workspaceDrafts.value[index].rootPath = selection; }
async function saveWorkspaceManager() { workspaceSaving.value = true; try { const nextConfig = buildWorkspaceConfigUpdate(configStore.config, workspaceDrafts.value, workspaceDefaultId.value); await configStore.saveConfig(nextConfig); if (!newChannelWorkspaceId.value || !nextConfig.workspacePresets.some((workspace) => workspace.id === newChannelWorkspaceId.value)) newChannelWorkspaceId.value = nextConfig.defaultWorkspaceId ?? nextConfig.workspacePresets[0]?.id ?? ""; } finally { workspaceSaving.value = false; } }
async function openWorkspaceFolder() { if (activeWorkspace.value) await invoke("open_workspace_folder", { workspaceId: activeWorkspace.value.id }); }
async function openWorkspaceTerminal() { if (activeWorkspace.value) await invoke("open_workspace_terminal", { workspaceId: activeWorkspace.value.id }); }
async function refreshProviderChecklist() { if (!activeWorkspace.value || !responseAgentId.value) return; onboardingBusy.value = true; try { providerChecklist.value = await invoke<ProviderSetupChecklistResult>("check_provider_setup", { providerId: activeProviderId.value, workspaceId: activeWorkspace.value.id }); onboardingMessage.value = providerChecklist.value.summary; } catch (error) { onboardingMessage.value = String(error); } finally { onboardingBusy.value = false; } }
async function checkProviderAuth() { if (!canRunOnboardingAction.value) return; onboardingBusy.value = true; try { const result = await invoke<ProviderAuthState>("get_provider_auth_state", { providerId: activeProviderId.value }); onboardingMessage.value = result.summary; } catch (error) { onboardingMessage.value = String(error); } finally { onboardingBusy.value = false; } }
async function smokeProvider() { if (!activeWorkspace.value || !canRunOnboardingAction.value) return; onboardingBusy.value = true; try { const result = await invoke<ProviderProbeResult>("smoke_test_provider_command", { providerId: activeProviderId.value, workspaceId: activeWorkspace.value.id }); onboardingMessage.value = result.summary; } catch (error) { onboardingMessage.value = String(error); } finally { onboardingBusy.value = false; } }
async function bootstrapProvider() { if (!activeWorkspace.value || !canRunOnboardingAction.value) return; onboardingBusy.value = true; try { const result = await invoke<WorkspaceBootstrapResult>("bootstrap_provider_workspace", { providerId: activeProviderId.value, workspaceId: activeWorkspace.value.id, agentId: responseAgentId.value, overwrite: false }); onboardingMessage.value = result.summary; await refreshProviderChecklist(); } catch (error) { onboardingMessage.value = String(error); } finally { onboardingBusy.value = false; } }
</script>

<style scoped>
.ops-room-view,.stack{display:flex;flex-direction:column;gap:12px}.ops-shell{display:grid;grid-template-columns:minmax(240px,.8fr) minmax(460px,1.45fr) minmax(320px,.95fr);gap:16px;align-items:start}.glass-panel,.card,.card-button,.presence-card{border-radius:var(--radius-xl);border:1px solid color-mix(in srgb,var(--glass-border) 84%,transparent);background:color-mix(in srgb,var(--bg-secondary) 92%,transparent)}.glass-panel{padding:16px}.card,.card-button,.presence-card{padding:12px}.card-button{display:flex;flex-direction:column;align-items:flex-start;gap:6px;text-align:left}.card-button.active,.reply{border-color:color-mix(in srgb,var(--accent) 24%,transparent);background:color-mix(in srgb,var(--accent) 10%,transparent)}.channel-button,.rail-section,.presence-card,.message-card,.escalation-card{gap:8px}.row{display:flex;gap:8px;flex-wrap:wrap}.between{justify-content:space-between}.full-width{width:100%}.glass-input{width:100%;border-radius:var(--radius-lg);border:1px solid color-mix(in srgb,var(--glass-border) 88%,transparent);background:color-mix(in srgb,var(--bg-secondary) 94%,transparent);color:var(--text-primary);padding:11px 13px}.chip,.status-chip{border:none;background:color-mix(in srgb,var(--accent) 6%,var(--bg-surface));border-radius:999px;padding:6px 10px;font-family:var(--font-mono);font-size:10px;text-transform:uppercase;letter-spacing:.08em}.chip.toggle{display:inline-flex;align-items:center}.chip-alert{background:rgba(245,158,11,.12);color:#f59e0b}.eyebrow,.muted{color:var(--text-muted);font-size:12px;line-height:1.5}.eyebrow{font-family:var(--font-mono);font-size:10px;letter-spacing:.14em;text-transform:uppercase}.artifact-value{margin:8px 0 0;white-space:pre-wrap;font-size:12px;color:var(--text-secondary)}.error-card{border-radius:var(--radius-xl);border:1px solid rgba(239,68,68,.24);background:rgba(239,68,68,.08);color:var(--accent-error);padding:12px 14px}.message-list{gap:10px}.message-body{margin:0;color:var(--text-primary);line-height:1.65}.presence-strip{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:10px}.status-chip--success{color:#22c55e;background:rgba(34,197,94,.12)}.status-chip--warn{color:#f59e0b;background:rgba(245,158,11,.12)}.status-chip--danger{color:#ef4444;background:rgba(239,68,68,.10)}.status-chip--info{color:var(--accent);background:var(--accent-dim)}.status-chip--neutral{color:var(--text-secondary);background:color-mix(in srgb,var(--bg-surface) 80%,transparent)}@media (max-width:1280px){.ops-shell{grid-template-columns:1fr}}
</style>
