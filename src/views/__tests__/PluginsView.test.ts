import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Agent, Task } from "@/interfaces";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  useGithubStore: vi.fn(),
  useAgentStore: vi.fn(),
  useTaskStore: vi.fn(),
  useConfigStore: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@/stores/useGithubStore", () => ({
  useGithubStore: mocks.useGithubStore,
}));

vi.mock("@/stores/useAgentStore", () => ({
  useAgentStore: mocks.useAgentStore,
}));

vi.mock("@/stores/useTaskStore", () => ({
  useTaskStore: mocks.useTaskStore,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

import PluginsView from "../PluginsView.vue";

describe("PluginsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
        },
        setTheme: vi.fn(),
      })
    );

    const agents: Agent[] = [
      {
        id: "forge",
        name: "Forge",
        role: "build automation",
        status: "idle",
        activeTaskId: undefined,
        lastSeen: new Date().toISOString(),
        allowedTools: ["python", "git"],
        skills: ["build", "release", "automation"],
      },
    ];

    const tasks: Task[] = [
      {
        id: "task-1",
        title: "build release digest",
        status: "todo",
        priority: "high",
        project: "UMBRA",
        projectId: "p1",
        columnId: "backlog",
        description: "automation build release",
        labels: ["build"],
      },
    ];

    mocks.useGithubStore.mockReturnValue(
      reactive({
        repos: [{ id: "repo-1", name: "UMBRA", openIssues: 3 }],
        loading: false,
        loadRepos: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useAgentStore.mockReturnValue(
      reactive({
        agents,
        loadAgents: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks,
        fetchTasks: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "get_obsidian_stats") {
        return {
          connected: true,
          totalNotes: 4,
          recentNotes: [{ name: "release digest", modified: new Date().toISOString() }],
          vaultPath: "D:/vault/UMBRA_Notes",
        };
      }
      if (command === "get_tmlite_tasks") {
        return [{ id: "tm-1", title: "Track release", status: "open", project: "UMBRA", priority: "high" }];
      }
      return null;
    });
  });

  it("renders assignment broker recommendations and roadmap plugins", async () => {
    const wrapper = mount(PluginsView);
    await flushPromises();

    expect(wrapper.text()).toContain("ASSIGNMENT BROKER");
    expect(wrapper.text()).toContain("ACTIVE");
    expect(wrapper.text()).toContain("build release digest");
    expect(wrapper.text()).toContain("Forge");
    expect(wrapper.text()).toContain("NEXT PLUGINS");
    expect(wrapper.text()).toContain("release radar");
  });
});
