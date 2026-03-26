import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

import { useWorkbenchStore } from "../useWorkbenchStore";

describe("useWorkbenchStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    mocks.listen.mockResolvedValue(() => {});
  });

  it("creates a run and resets the prompt", async () => {
    const now = new Date().toISOString();
    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "create_dispatch_run") {
        return {
          id: "run-1",
          parentRunId: null,
          mode: "task",
          agentId: "forge",
          providerId: "custom",
          workspaceId: "core",
          pmTaskId: "task-7",
          prompt: "ship it",
          personaId: "implementer",
          outcomeStatus: null,
          status: "queued",
          createdAt: now,
          updatedAt: now,
        };
      }
      if (command === "list_run_events_page") {
        return { items: [], nextBefore: null, hasMore: false };
      }
      if (command === "list_run_artifacts") {
        return [];
      }
      if (command === "list_dispatch_runs") {
        return [];
      }
      return null;
    });

    const store = useWorkbenchStore();
    store.draft.prompt = "ship it";
    store.draft.agentId = "forge";
    store.draft.workspaceId = "core";
    store.draft.pmTaskId = "task-7";
    store.draft.personaId = "implementer";

    const run = await store.createRun();

    expect(run?.id).toBe("run-1");
    expect(run?.personaId).toBe("implementer");
    expect(run?.pmTaskId).toBe("task-7");
    expect(store.runs[0]?.id).toBe("run-1");
    expect(store.draft.prompt).toBe("");
    expect(store.draft.continueFromRunId).toBeNull();
  });

  it("tracks canSend only when all required fields are present", () => {
    const store = useWorkbenchStore();
    expect(store.canSend).toBe(false);

    store.draft.prompt = "hello";
    store.draft.agentId = "forge";
    store.draft.workspaceId = "core";

    expect(store.canSend).toBe(true);
  });

  it("prepares a continuation from an existing run", () => {
    const store = useWorkbenchStore();
    store.prepareContinuation({
      id: "run-9",
      parentRunId: null,
      mode: "chat",
      agentId: "forge",
      providerId: "codex",
      workspaceId: "core",
      pmTaskId: "task-9",
      prompt: "hello",
      personaId: "implementer",
      outcomeStatus: "needs_input",
      status: "done",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });

    expect(store.draft.agentId).toBe("forge");
    expect(store.draft.workspaceId).toBe("core");
    expect(store.draft.pmTaskId).toBe("task-9");
    expect(store.draft.continueFromRunId).toBe("run-9");
  });

  it("retries a failed run and selects the new attempt", async () => {
    const now = new Date().toISOString();
    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "retry_dispatch_run") {
        return {
          id: "run-2",
          parentRunId: "run-1",
          mode: "task",
          agentId: "forge",
          providerId: "codex",
          workspaceId: "core",
          pmTaskId: null,
          prompt: "retry this",
          personaId: null,
          outcomeStatus: null,
          status: "queued",
          createdAt: now,
          updatedAt: now,
        };
      }
      if (command === "list_run_events_page") {
        return { items: [], nextBefore: null, hasMore: false };
      }
      if (command === "list_run_artifacts") {
        return [];
      }
      return [];
    });

    const store = useWorkbenchStore();
    const run = await store.retryRun("run-1");

    expect(run?.id).toBe("run-2");
    expect(store.activeRunId).toBe("run-2");
  });
});
