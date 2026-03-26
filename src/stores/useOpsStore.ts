import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DispatchRun,
  OpsChannel,
  OpsChannelMessage,
  OpsChannelMessagePage,
  OpsJob,
  OpsRouteApproval,
  OpsRule,
  OpsSession,
  OpsSessionTemplate,
} from "@/interfaces";

type ComposerDraft = {
  body: string;
  agentId: string | null;
  parentMessageId: string | null;
};

export const useOpsStore = defineStore("ops-room", () => {
  const channels = ref<OpsChannel[]>([]);
  const activeChannelId = ref<string | null>(null);
  const messagesByChannel = ref<Record<string, OpsChannelMessage[]>>({});
  const messagePageStateByChannel = ref<
    Record<string, { nextBefore: string | null; hasMore: boolean }>
  >({});
  const runsByChannel = ref<Record<string, DispatchRun[]>>({});
  const jobsByChannel = ref<Record<string, OpsJob[]>>({});
  const approvalsByChannel = ref<Record<string, OpsRouteApproval[]>>({});
  const sessionsByChannel = ref<Record<string, OpsSession[]>>({});
  const rules = ref<OpsRule[]>([]);
  const sessionTemplates = ref<OpsSessionTemplate[]>([]);
  const draft = ref<ComposerDraft>({ body: "", agentId: null, parentMessageId: null });
  const loading = ref(false);
  const sending = ref(false);
  const error = ref<string | null>(null);

  const unlistenHandles = ref<Array<() => void>>([]);

  const activeChannel = computed(
    () => channels.value.find((channel) => channel.id === activeChannelId.value) ?? null,
  );
  const activeMessages = computed(() =>
    activeChannelId.value ? messagesByChannel.value[activeChannelId.value] ?? [] : [],
  );
  const activeRuns = computed(() =>
    activeChannelId.value ? runsByChannel.value[activeChannelId.value] ?? [] : [],
  );
  const activeJobs = computed(() =>
    activeChannelId.value ? jobsByChannel.value[activeChannelId.value] ?? [] : [],
  );
  const activeApprovals = computed(() =>
    activeChannelId.value ? approvalsByChannel.value[activeChannelId.value] ?? [] : [],
  );
  const activeSessions = computed(() =>
    activeChannelId.value ? sessionsByChannel.value[activeChannelId.value] ?? [] : [],
  );
  const activeSession = computed(
    () => activeSessions.value.find((session) => session.state === "active") ?? null,
  );

  async function loadChannels() {
    loading.value = true;
    error.value = null;
    try {
      channels.value = await invoke<OpsChannel[]>("list_ops_channels");
      rules.value = await invoke<OpsRule[]>("list_ops_rules");
      sessionTemplates.value = await invoke<OpsSessionTemplate[]>("list_ops_session_templates");
      if (!activeChannelId.value && channels.value.length > 0) {
        activeChannelId.value = channels.value[0].id;
      }
      if (activeChannelId.value) {
        await loadChannelData(activeChannelId.value);
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadChannelData(channelId: string) {
    try {
      const [messages, runs, jobs, approvals, sessions] = await Promise.all([
        invoke<OpsChannelMessagePage>("list_ops_channel_messages_page", { channelId, limit: 100 }),
        invoke<DispatchRun[]>("list_channel_dispatch_runs", { channelId }),
        invoke<OpsJob[]>("list_ops_jobs", { channelId }),
        invoke<OpsRouteApproval[]>("list_ops_route_approvals", { channelId }),
        invoke<OpsSession[]>("list_ops_sessions", { channelId }),
      ]);
      messagesByChannel.value[channelId] = messages.items;
      runsByChannel.value[channelId] = sortRuns(runs);
      messagePageStateByChannel.value[channelId] = {
        nextBefore: messages.nextBefore ?? null,
        hasMore: messages.hasMore,
      };
      jobsByChannel.value[channelId] = jobs;
      approvalsByChannel.value[channelId] = approvals;
      sessionsByChannel.value[channelId] = sessions;
    } catch (e) {
      error.value = String(e);
    }
  }

  async function loadOlderMessages(channelId: string) {
    const pageState = messagePageStateByChannel.value[channelId];
    if (!pageState?.hasMore || !pageState.nextBefore) return;
    try {
      const page = await invoke<OpsChannelMessagePage>("list_ops_channel_messages_page", {
        channelId,
        before: pageState.nextBefore,
        limit: 100,
      });
      messagesByChannel.value[channelId] = [...page.items, ...(messagesByChannel.value[channelId] ?? [])];
      messagePageStateByChannel.value[channelId] = {
        nextBefore: page.nextBefore ?? null,
        hasMore: page.hasMore,
      };
    } catch (e) {
      error.value = String(e);
    }
  }

  async function createChannel(input: {
    name: string;
    description: string;
    workspaceId: string;
    defaultAgentId?: string | null;
  }) {
    try {
      const channel = await invoke<OpsChannel>("create_ops_channel", { input });
      upsertChannel(channel);
      activeChannelId.value = channel.id;
      await loadChannelData(channel.id);
      return channel;
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function sendMessage() {
    if (!activeChannelId.value || !draft.value.body.trim()) return null;
    sending.value = true;
    error.value = null;
    try {
      const message = await invoke<OpsChannelMessage>("send_ops_channel_message", {
        input: {
          channelId: activeChannelId.value,
          body: draft.value.body,
          agentId: draft.value.agentId,
          parentMessageId: draft.value.parentMessageId,
        },
      });
      draft.value.body = "";
      draft.value.agentId = null;
      draft.value.parentMessageId = null;
      appendMessage(message);
      return message;
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      sending.value = false;
    }
  }

  async function createJob(input: {
    sourceMessageId: string;
    title: string;
    summary: string;
    agentId: string;
    workspaceId: string;
    pmTaskId?: string | null;
  }) {
    if (!activeChannelId.value) return null;
    try {
      return await invoke<OpsJob>("create_ops_job", {
        input: {
          channelId: activeChannelId.value,
          ...input,
        },
      });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function resolveApproval(approvalId: string, approved: boolean) {
    try {
      return await invoke<OpsRouteApproval>("resolve_ops_route_approval", {
        approvalId,
        approved,
      });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function saveRule(input: {
    id?: string | null;
    name: string;
    pattern: string;
    targetAgentId?: string | null;
    workspaceId?: string | null;
    enabled: boolean;
    requiresHumanGate: boolean;
  }) {
    try {
      return await invoke<OpsRule>("save_ops_rule", { input });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function saveSessionTemplate(input: {
    id?: string | null;
    name: string;
    workspaceId: string;
    agentIds: string[];
    autoAdvance: boolean;
    requiresHumanGate: boolean;
  }) {
    try {
      return await invoke<OpsSessionTemplate>("save_ops_session_template", { input });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function startSession(templateId: string) {
    if (!activeChannelId.value) return null;
    try {
      return await invoke<OpsSession>("start_ops_session", {
        input: {
          channelId: activeChannelId.value,
          templateId,
        },
      });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function advanceSession(sessionId: string) {
    try {
      return await invoke<OpsSession>("advance_ops_session", { sessionId });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function pauseSession(sessionId: string, paused: boolean) {
    try {
      return await invoke<OpsSession>("pause_ops_session", { sessionId, paused });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function setupLiveUpdates() {
    if (unlistenHandles.value.length > 0) return;
    try {
      unlistenHandles.value.push(await listen<OpsChannel>("ops:channel-created", ({ payload }) => {
        upsertChannel(payload);
      }));
      unlistenHandles.value.push(await listen<OpsChannel>("ops:channel-updated", ({ payload }) => {
        upsertChannel(payload);
      }));
      unlistenHandles.value.push(await listen<OpsChannelMessage>("ops:message-added", ({ payload }) => {
        appendMessage(payload);
      }));
      unlistenHandles.value.push(await listen<OpsJob>("ops:job-updated", ({ payload }) => {
        upsertJob(payload);
      }));
      unlistenHandles.value.push(await listen<OpsRouteApproval>("ops:approval-updated", ({ payload }) => {
        upsertApproval(payload);
      }));
      unlistenHandles.value.push(await listen<OpsRule>("ops:rule-updated", ({ payload }) => {
        upsertRule(payload);
      }));
      unlistenHandles.value.push(await listen<OpsSessionTemplate>(
        "ops:session-template-updated",
        ({ payload }) => {
          upsertSessionTemplate(payload);
        },
      ));
      unlistenHandles.value.push(await listen<OpsSession>("ops:session-updated", ({ payload }) => {
        upsertSession(payload);
      }));
      unlistenHandles.value.push(await listen<DispatchRun>("workbench:run-created", ({ payload }) => {
        upsertRun(payload);
      }));
      unlistenHandles.value.push(await listen<DispatchRun>("workbench:run-updated", ({ payload }) => {
        upsertRun(payload);
      }));
    } catch {
      // tauri event api not available in browser-mode tests
    }
  }

  function selectChannel(channelId: string) {
    activeChannelId.value = channelId;
    if (!messagesByChannel.value[channelId] || !runsByChannel.value[channelId]) {
      void loadChannelData(channelId);
    }
  }

  function upsertChannel(channel: OpsChannel) {
    const index = channels.value.findIndex((entry) => entry.id === channel.id);
    if (index >= 0) {
      channels.value[index] = channel;
    } else {
      channels.value = [channel, ...channels.value];
    }
    channels.value = [...channels.value].sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
  }

  function appendMessage(message: OpsChannelMessage) {
    const current = messagesByChannel.value[message.channelId] ?? [];
    if (current.some((entry) => entry.id === message.id)) return;
    messagesByChannel.value[message.channelId] = [...current, message].sort((left, right) =>
      left.createdAt.localeCompare(right.createdAt),
    );
  }

  function upsertRun(run: DispatchRun) {
    if (!run.channelId) return;
    const current = runsByChannel.value[run.channelId] ?? [];
    const index = current.findIndex((entry) => entry.id === run.id);
    if (index >= 0) {
      current[index] = run;
      runsByChannel.value[run.channelId] = sortRuns(current);
      return;
    }
    runsByChannel.value[run.channelId] = sortRuns([run, ...current]);
  }

  function setReplyTarget(messageId: string | null) {
    draft.value.parentMessageId = messageId;
  }

  function upsertJob(job: OpsJob) {
    const current = jobsByChannel.value[job.channelId] ?? [];
    const index = current.findIndex((entry) => entry.id === job.id);
    if (index >= 0) {
      current[index] = job;
      jobsByChannel.value[job.channelId] = [...current];
      return;
    }
    jobsByChannel.value[job.channelId] = [job, ...current];
  }

  function upsertApproval(approval: OpsRouteApproval) {
    const current = approvalsByChannel.value[approval.channelId] ?? [];
    const index = current.findIndex((entry) => entry.id === approval.id);
    if (index >= 0) {
      current[index] = approval;
      approvalsByChannel.value[approval.channelId] = [...current];
      return;
    }
    approvalsByChannel.value[approval.channelId] = [approval, ...current];
  }

  function upsertRule(rule: OpsRule) {
    const index = rules.value.findIndex((entry) => entry.id === rule.id);
    if (index >= 0) {
      rules.value[index] = rule;
    } else {
      rules.value = [rule, ...rules.value];
    }
  }

  function upsertSessionTemplate(template: OpsSessionTemplate) {
    const index = sessionTemplates.value.findIndex((entry) => entry.id === template.id);
    if (index >= 0) {
      sessionTemplates.value[index] = template;
    } else {
      sessionTemplates.value = [template, ...sessionTemplates.value];
    }
  }

  function upsertSession(session: OpsSession) {
    const current = sessionsByChannel.value[session.channelId] ?? [];
    const index = current.findIndex((entry) => entry.id === session.id);
    if (index >= 0) {
      current[index] = session;
      sessionsByChannel.value[session.channelId] = [...current];
      return;
    }
    sessionsByChannel.value[session.channelId] = [session, ...current];
  }

  return {
    channels,
    activeChannelId,
    activeChannel,
    activeMessages,
    activeRuns,
    activeJobs,
    activeApprovals,
    activeSessions,
    activeSession,
    rules,
    sessionTemplates,
    draft,
    loading,
    sending,
    error,
    loadChannels,
    loadChannelData,
    loadOlderMessages,
    createChannel,
    sendMessage,
    createJob,
    resolveApproval,
    saveRule,
    saveSessionTemplate,
    startSession,
    advanceSession,
    pauseSession,
    setupLiveUpdates,
    selectChannel,
    setReplyTarget,
    messagePageStateByChannel,
  };
});

function sortRuns(runs: DispatchRun[]) {
  return [...runs].sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
}
