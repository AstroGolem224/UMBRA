import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DispatchMode,
  DispatchRun,
  RunArtifact,
  RunEvent,
  RunEventPage,
} from "@/interfaces";

type DraftState = {
  prompt: string;
  agentId: string;
  workspaceId: string;
  pmTaskId: string | null;
  mode: DispatchMode;
  personaId: string | null;
  continueFromRunId: string | null;
};

export const useWorkbenchStore = defineStore("workbench", () => {
  const runs = ref<DispatchRun[]>([]);
  const eventsByRun = ref<Record<string, RunEvent[]>>({});
  const artifactsByRun = ref<Record<string, RunArtifact[]>>({});
  const eventsPageStateByRun = ref<Record<string, { nextBefore: string | null; hasMore: boolean }>>(
    {},
  );
  const activeRunId = ref<string | null>(null);
  const loading = ref(false);
  const sending = ref(false);
  const error = ref<string | null>(null);
  const draft = ref<DraftState>({
    prompt: "",
    agentId: "",
    workspaceId: "",
    pmTaskId: null,
    mode: "task",
    personaId: null,
    continueFromRunId: null,
  });
  let unlistenRunCreated: (() => void) | null = null;
  let unlistenRunUpdated: (() => void) | null = null;
  let unlistenEventAdded: (() => void) | null = null;
  let unlistenArtifactsReplaced: (() => void) | null = null;

  const activeRun = computed(() => runs.value.find((run) => run.id === activeRunId.value) ?? null);
  const activeEvents = computed(() =>
    activeRunId.value ? eventsByRun.value[activeRunId.value] ?? [] : [],
  );
  const activeArtifacts = computed(() =>
    activeRunId.value ? artifactsByRun.value[activeRunId.value] ?? [] : [],
  );

  async function loadRuns() {
    loading.value = true;
    error.value = null;
    try {
      runs.value = await invoke<DispatchRun[]>("list_dispatch_runs");
      if (!activeRunId.value && runs.value.length > 0) {
        activeRunId.value = runs.value[0].id;
        await loadEvents(runs.value[0].id);
        await loadArtifacts(runs.value[0].id);
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadEvents(runId: string) {
    try {
      const page = await invoke<RunEventPage>("list_run_events_page", {
        runId,
        limit: 80,
      });
      eventsByRun.value[runId] = page.items;
      eventsPageStateByRun.value[runId] = {
        nextBefore: page.nextBefore ?? null,
        hasMore: page.hasMore,
      };
    } catch (e) {
      error.value = String(e);
    }
  }

  async function loadOlderEvents(runId: string) {
    const pageState = eventsPageStateByRun.value[runId];
    if (!pageState?.hasMore || !pageState.nextBefore) return;
    try {
      const page = await invoke<RunEventPage>("list_run_events_page", {
        runId,
        before: pageState.nextBefore,
        limit: 80,
      });
      eventsByRun.value[runId] = [...page.items, ...(eventsByRun.value[runId] ?? [])];
      eventsPageStateByRun.value[runId] = {
        nextBefore: page.nextBefore ?? null,
        hasMore: page.hasMore,
      };
    } catch (e) {
      error.value = String(e);
    }
  }

  async function loadArtifacts(runId: string) {
    try {
      artifactsByRun.value[runId] = await invoke<RunArtifact[]>("list_run_artifacts", { runId });
    } catch (e) {
      error.value = String(e);
    }
  }

  async function createRun() {
    if (!canSend.value) return null;
    sending.value = true;
    error.value = null;
    try {
      const run = await invoke<DispatchRun>("create_dispatch_run", {
        input: {
          mode: draft.value.mode,
          agentId: draft.value.agentId,
          workspaceId: draft.value.workspaceId,
          pmTaskId: draft.value.pmTaskId,
          prompt: draft.value.prompt,
          personaId: draft.value.personaId,
          continueFromRunId: draft.value.continueFromRunId,
        },
      });
      upsertRun(run);
      activeRunId.value = run.id;
      draft.value.prompt = "";
      draft.value.continueFromRunId = null;
      await loadEvents(run.id);
      await loadArtifacts(run.id);
      return run;
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      sending.value = false;
    }
  }

  async function cancelRun(runId: string) {
    try {
      const run = await invoke<DispatchRun>("cancel_dispatch_run", { runId });
      upsertRun(run);
      await loadEvents(run.id);
      await loadArtifacts(run.id);
      return run;
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function retryRun(runId: string) {
    try {
      const run = await invoke<DispatchRun>("retry_dispatch_run", { runId });
      upsertRun(run);
      activeRunId.value = run.id;
      await loadEvents(run.id);
      await loadArtifacts(run.id);
      return run;
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function setupLiveUpdates() {
    if (unlistenRunCreated || unlistenRunUpdated || unlistenEventAdded || unlistenArtifactsReplaced) return;
    try {
      unlistenRunCreated = await listen<DispatchRun>("workbench:run-created", (event) => {
        upsertRun(event.payload);
      });
      unlistenRunUpdated = await listen<DispatchRun>("workbench:run-updated", (event) => {
        upsertRun(event.payload);
      });
      unlistenEventAdded = await listen<RunEvent>("workbench:event-added", (event) => {
        const runEvent = event.payload;
        const current = eventsByRun.value[runEvent.runId] ?? [];
        if (!current.some((entry) => entry.id === runEvent.id)) {
          eventsByRun.value[runEvent.runId] = [...current, runEvent];
        }
      });
      unlistenArtifactsReplaced = await listen<{ runId: string; artifacts: RunArtifact[] }>(
        "workbench:artifacts-replaced",
        (event) => {
          artifactsByRun.value[event.payload.runId] = event.payload.artifacts;
        },
      );
    } catch {
      // browser mode or tests without tauri events
    }
  }

  function selectRun(runId: string) {
    activeRunId.value = runId;
    if (!eventsByRun.value[runId]) {
      void loadEvents(runId);
    }
    if (!artifactsByRun.value[runId]) {
      void loadArtifacts(runId);
    }
  }

  function upsertRun(run: DispatchRun) {
    const index = runs.value.findIndex((entry) => entry.id === run.id);
    if (index >= 0) {
      runs.value[index] = run;
    } else {
      runs.value = [run, ...runs.value];
    }
    runs.value = [...runs.value].sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
  }

  function prepareContinuation(run: DispatchRun) {
    draft.value.agentId = run.agentId;
    draft.value.workspaceId = run.workspaceId;
    draft.value.mode = run.mode;
    draft.value.personaId = run.personaId ?? null;
    draft.value.pmTaskId = run.pmTaskId ?? null;
    draft.value.continueFromRunId = run.id;
  }

  function clearContinuation() {
    draft.value.continueFromRunId = null;
  }

  const canSend = computed(
    () =>
      draft.value.prompt.trim().length > 0 &&
      draft.value.agentId.trim().length > 0 &&
      draft.value.workspaceId.trim().length > 0,
  );

  return {
    runs,
    activeRunId,
    activeRun,
    activeEvents,
    activeArtifacts,
    loading,
    sending,
    error,
    draft,
    canSend,
    loadRuns,
    loadEvents,
    loadOlderEvents,
    loadArtifacts,
    createRun,
    cancelRun,
    retryRun,
    setupLiveUpdates,
    selectRun,
    prepareContinuation,
    clearContinuation,
    eventsPageStateByRun,
  };
});
