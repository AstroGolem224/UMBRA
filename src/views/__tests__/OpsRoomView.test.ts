import { flushPromises, mount } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  useOpsStore: vi.fn(),
  useAgentStore: vi.fn(),
  useConfigStore: vi.fn(),
  useTaskStore: vi.fn(),
  invoke: vi.fn(),
}));

vi.mock("@/stores/useOpsStore", () => ({
  useOpsStore: mocks.useOpsStore,
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

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

import OpsRoomView from "../OpsRoomView.vue";

describe("OpsRoomView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "check_provider_setup") {
        return {
          providerId: "codex",
          workspaceId: "core",
          summary: "3/3 checklist item(s) ready",
          items: [
            { key: "command", label: "provider command configured", ready: true, detail: "configured" },
            { key: "auth", label: "per-agent auth provisioned", ready: true, detail: "1/1 token ready" },
            { key: "workspace", label: "workspace inside grant roots", ready: true, detail: "allowed" },
          ],
        };
      }
      if (command === "get_provider_auth_state") {
        return { providerId: "codex", agentIds: ["forge"], provisionedCount: 1, summary: "1/1 token ready" };
      }
      if (command === "smoke_test_provider_command") {
        return { providerId: "codex", command: "codex", launchable: true, exitCode: 0, summary: "smoke ok" };
      }
      if (command === "bootstrap_provider_workspace") {
        return { providerId: "codex", workspaceId: "core", files: ["AGENTS.md"], summary: "wrote 1 bootstrap file(s)" };
      }
      if (command === "get_dispatch_run") return null;
      if (command === "list_run_artifacts") return [];
      return null;
    });

    mocks.useOpsStore.mockReturnValue(
      reactive({
        channels: [
          {
            id: "channel-1",
            name: "delivery",
            description: "",
            workspaceId: "core",
            defaultAgentId: "forge",
            createdAt: "2026-03-26T00:00:00Z",
            updatedAt: "2026-03-26T00:00:00Z",
          },
        ],
        activeChannelId: "channel-1",
        activeChannel: {
          id: "channel-1",
          name: "delivery",
          description: "",
          workspaceId: "core",
          defaultAgentId: "forge",
          createdAt: "2026-03-26T00:00:00Z",
          updatedAt: "2026-03-26T00:00:00Z",
        },
        activeMessages: [],
        activeRuns: [
          {
            id: "run-1",
            mode: "task",
            agentId: "forge",
            providerId: "codex",
            workspaceId: "core",
            prompt: "finish delivery",
            outcomeStatus: "needs_input",
            status: "error",
            createdAt: "2026-03-26T00:00:00Z",
            updatedAt: "2026-03-26T00:05:00Z",
          },
        ],
        activeJobs: [],
        activeApprovals: [],
        activeSessions: [],
        activeSession: null,
        messagePageStateByChannel: {},
        rules: [],
        sessionTemplates: [],
        draft: { body: "", agentId: null, parentMessageId: null },
        loading: false,
        sending: false,
        error: null,
        loadChannels: vi.fn().mockResolvedValue(undefined),
        loadOlderMessages: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
        selectChannel: vi.fn(),
        setReplyTarget: vi.fn(),
        createChannel: vi.fn().mockResolvedValue(undefined),
        sendMessage: vi.fn().mockResolvedValue(undefined),
        createJob: vi.fn().mockResolvedValue(undefined),
        resolveApproval: vi.fn().mockResolvedValue(undefined),
        saveRule: vi.fn().mockResolvedValue(undefined),
        saveSessionTemplate: vi.fn().mockResolvedValue(undefined),
        startSession: vi.fn().mockResolvedValue(undefined),
        advanceSession: vi.fn().mockResolvedValue(undefined),
        pauseSession: vi.fn().mockResolvedValue(undefined),
      }),
    );

    mocks.useAgentStore.mockReturnValue(
      reactive({
        agents: [
          {
            id: "forge",
            name: "Forge",
            role: "builder",
            status: "idle",
            lastSeen: "2026-03-26T00:00:00Z",
            allowedTools: [],
            skills: [],
          },
        ],
        loadAgents: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      }),
    );

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          defaultWorkspaceId: "core",
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
        },
        saveConfig: vi.fn().mockResolvedValue(undefined),
      }),
    );

    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks: [],
        fetchTasks: vi.fn().mockResolvedValue(undefined),
      }),
    );
  });

  const stubs = {
    ViewHero: {
      props: ["title", "subtitle"],
      template: "<div><h1>{{ title }}</h1><p>{{ subtitle }}</p><slot name='meta' /></div>",
    },
    NeonButton: {
      props: ["disabled", "loading"],
      template: "<button :disabled='disabled || loading'><slot /></button>",
    },
  };

  it("renders the ops room shell with live visibility", async () => {
    const wrapper = mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(wrapper.text()).toContain("Ops Room");
    expect(wrapper.text()).toContain("channels");
    expect(wrapper.text()).toContain("sessions");
    expect(wrapper.text()).toContain("live runs");
    expect(wrapper.text()).toContain("blocked + needs input");
    expect(wrapper.text()).toContain("onboarding");
    expect(wrapper.text()).toContain("Forge");
  });

  it("loads channels and agents on mount", async () => {
    const opsStore = mocks.useOpsStore();
    const agentStore = mocks.useAgentStore();
    mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(opsStore.loadChannels).toHaveBeenCalled();
    expect(agentStore.loadAgents).toHaveBeenCalled();
    expect(agentStore.setupLiveUpdates).toHaveBeenCalled();
  });

  it("shows escalation rail for blocked/needs_input runs", async () => {
    const wrapper = mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(wrapper.text()).toContain("needs input");
  });

  it("shows channel in sidebar with delivery name", async () => {
    const wrapper = mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(wrapper.text()).toContain("delivery");
  });

  it("renders message compose area", async () => {
    const wrapper = mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(wrapper.text()).toContain("send");
    expect(wrapper.text()).toContain("auto route");
  });

  it("displays agent name in presence section", async () => {
    const wrapper = mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(wrapper.text()).toContain("Forge");
    expect(wrapper.text()).toContain("builder");
  });

  it("handles provider checklist invocation", async () => {
    mount(OpsRoomView, { global: { stubs } });
    await flushPromises();

    expect(mocks.invoke).toHaveBeenCalledWith(
      "check_provider_setup",
      expect.any(Object),
    );
  });
});
