import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Agent, AgentCronJob, Note, Task } from "@/interfaces";

const mocks = vi.hoisted(() => ({
  useAgentStore: vi.fn(),
  useCronStore: vi.fn(),
  useNotesStore: vi.fn(),
  useTaskStore: vi.fn(),
  useConfigStore: vi.fn(),
}));

vi.mock("@/stores/useAgentStore", () => ({ useAgentStore: mocks.useAgentStore }));
vi.mock("@/stores/useCronStore", () => ({ useCronStore: mocks.useCronStore }));
vi.mock("@/stores/useNotesStore", () => ({ useNotesStore: mocks.useNotesStore }));
vi.mock("@/stores/useTaskStore", () => ({ useTaskStore: mocks.useTaskStore }));
vi.mock("@/stores/useConfigStore", () => ({ useConfigStore: mocks.useConfigStore }));

import DashboardView from "../DashboardView.vue";

describe("DashboardView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

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
      {
        id: "prism",
        name: "Prism",
        role: "ui systems",
        status: "working",
        activeTaskId: "task-2",
        lastSeen: new Date(Date.now() - 45 * 60_000).toISOString(),
        allowedTools: ["vue", "css"],
        skills: ["frontend", "dashboard"],
      },
    ];

    const cronJobs: AgentCronJob[] = [
      {
        id: "daily-build",
        agentId: "forge",
        agentName: "Forge",
        job: "daily build",
        timing: "09:00",
        recurrence: "weekdays",
        timezone: "Europe/Berlin",
        enabled: true,
        lastRun: new Date().toISOString(),
        nextRun: new Date(Date.now() + 60 * 60_000).toISOString(),
        lastStatus: "error",
        notes: "digest failed",
        source: "systemd timer",
        command: null,
        updatedAt: new Date().toISOString(),
      },
    ];

    const notes: Note[] = [
      {
        id: "note-1",
        title: "release digest",
        content: "updated",
        category: "misc",
        tags: ["release"],
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        filePath: "mock://release.md",
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
        deadline: new Date(Date.now() - 60 * 60_000).toISOString(),
      },
    ];

    mocks.useAgentStore.mockReturnValue(
      reactive({
        agents,
        error: null,
        loadAgents: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useCronStore.mockReturnValue(
      reactive({
        jobs: cronJobs,
        error: null,
        lastSync: new Date().toISOString(),
        loadJobs: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useNotesStore.mockReturnValue(
      reactive({
        notes,
        error: null,
        loadNotes: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks,
        error: null,
        lastSync: new Date().toISOString(),
        fetchTasks: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
          uapPort: 8765,
          pmToolPollSeconds: 30,
        },
        setTheme: vi.fn(),
      })
    );
  });

  it("renders the stitch-inspired dashboard layout and task volume chart", async () => {
    const wrapper = mount(DashboardView);
    await flushPromises();

    expect(wrapper.text()).toContain("Umbra Dashboard Overview");
    expect(wrapper.text()).toContain("Total Tasks");
    expect(wrapper.text()).toContain("Deployment Registry");
    expect(wrapper.text()).toContain("Priority Signals");
    expect(wrapper.text()).toContain("build release digest");
    expect(wrapper.text()).toContain("Forge");
    expect(wrapper.text()).toContain("daily build");
    expect(wrapper.text()).toContain("digest failed");
    expect(wrapper.text()).toContain("In Progress");
    expect(wrapper.text()).toContain("Completed");
    expect(wrapper.text()).toContain("Overdue");
    expect(wrapper.find('[aria-label="task volume chart"]').exists()).toBe(true);
  });

  it("cycles the hero theme pill", async () => {
    const wrapper = mount(DashboardView);
    await flushPromises();

    const heroButton = wrapper.find(".view-hero__pill--button");
    expect(heroButton.exists()).toBe(true);

    await heroButton.trigger("click");

    const store = mocks.useConfigStore.mock.results[0].value;
    expect(store.setTheme).toHaveBeenCalledWith("neon");
  });
});
