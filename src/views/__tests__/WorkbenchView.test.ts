import { flushPromises, mount } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  useAgentStore: vi.fn(),
  useConfigStore: vi.fn(),
  useTaskStore: vi.fn(),
  useWorkbenchStore: vi.fn(),
  invoke: vi.fn(),
}));

vi.mock("@/stores/useAgentStore", () => ({
  useAgentStore: mocks.useAgentStore,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

vi.mock("@/stores/useTaskStore", () => ({
  useTaskStore: mocks.useTaskStore,
}));

vi.mock("@/stores/useWorkbenchStore", () => ({
  useWorkbenchStore: mocks.useWorkbenchStore,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

import WorkbenchView from "../WorkbenchView.vue";

describe("WorkbenchView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "check_provider_setup") {
        return {
          providerId: "custom",
          workspaceId: "core",
          summary: "2/2 checklist item(s) ready",
          items: [
            { key: "command", label: "provider command configured", ready: true, detail: "configured" },
            { key: "workspace", label: "workspace inside grant roots", ready: true, detail: "allowed" },
          ],
        };
      }
      if (command === "get_provider_auth_state") {
        return { providerId: "custom", agentIds: ["forge"], provisionedCount: 1, summary: "1/1 token ready" };
      }
      if (command === "smoke_test_provider_command") {
        return { providerId: "custom", command: "forge", launchable: true, exitCode: 0, summary: "smoke ok" };
      }
      if (command === "bootstrap_provider_workspace") {
        return { providerId: "custom", workspaceId: "core", files: ["AGENTS.md"], summary: "wrote 1 bootstrap file(s)" };
      }
      return null;
    });

    mocks.useAgentStore.mockReturnValue(
      reactive({
        agents: [
          {
            id: "forge",
            name: "Forge",
            role: "builder",
            status: "working",
            lastSeen: new Date().toISOString(),
            allowedTools: [],
            skills: [],
          },
        ],
        loadAgents: vi.fn().mockResolvedValue(undefined),
      }),
    );

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          workspacePresets: [
            {
              id: "core",
              name: "Core",
              rootPath: "C:/Repos/UMBRA",
              writable: true,
              allowedProviders: [],
              allowedAgents: [],
            },
          ],
          defaultWorkspaceId: "core",
          agentAuthTokens: { forge: "forge-token" },
          personaPresets: [
            {
              id: "implementer",
              name: "implementer",
              description: "ship code",
              prompt: "implement the task",
            },
          ],
        },
        saveConfig: vi.fn().mockResolvedValue(undefined),
      }),
    );

    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks: [
          {
            id: "task-1",
            title: "finish phase 4",
            status: "in-progress",
            priority: "high",
            project: "UMBRA",
            projectId: "p1",
            columnId: "c1",
          },
        ],
        fetchTasks: vi.fn().mockResolvedValue(undefined),
      }),
    );

    mocks.useWorkbenchStore.mockReturnValue(
      reactive({
        runs: [],
        activeRunId: null,
        activeRun: null,
        activeEvents: [],
        activeArtifacts: [],
        eventsPageStateByRun: {},
        loading: false,
        sending: false,
        error: null,
        draft: {
          prompt: "",
          agentId: "",
          workspaceId: "",
          pmTaskId: null,
          mode: "task",
          personaId: null,
          continueFromRunId: null,
        },
        canSend: false,
        loadRuns: vi.fn().mockResolvedValue(undefined),
        loadOlderEvents: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
        createRun: vi.fn().mockResolvedValue(undefined),
        cancelRun: vi.fn().mockResolvedValue(undefined),
        retryRun: vi.fn().mockResolvedValue(undefined),
        selectRun: vi.fn(),
        prepareContinuation: vi.fn(),
        clearContinuation: vi.fn(),
      }),
    );
  });

  it("renders persona options and disables send without required fields", async () => {
    const wrapper = mount(WorkbenchView, {
      global: {
        stubs: {
          ViewHero: { template: "<div><slot name='meta' /></div>" },
          NeonButton: {
            props: ["disabled", "loading"],
            template: "<button :disabled='disabled || loading'><slot /></button>",
          },
        },
      },
    });

    await flushPromises();

    expect(wrapper.text()).toContain("implementer");
    expect(wrapper.text()).toContain("linked pm task");
    expect(wrapper.text()).toContain("onboarding");
    expect(wrapper.text()).toContain("checklist");
    const sendButton = wrapper.findAll("button").find((button) => button.text().includes("send"));
    expect(sendButton?.attributes("disabled")).toBeDefined();
  });
});
