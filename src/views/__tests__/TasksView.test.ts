import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "@/interfaces";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  useConfigStore: vi.fn(),
  useTaskStore: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@/stores/useTaskStore", () => ({
  useTaskStore: mocks.useTaskStore,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

import TasksView from "../TasksView.vue";

describe("TasksView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    const tasks: Task[] = [
      {
        id: "t-backlog",
        title: "Backlog task",
        status: "todo",
        priority: "medium",
        project: "UMBRA",
        projectId: "p1",
        columnId: "c-backlog",
        columnKind: "backlog",
      },
      {
        id: "t-doing",
        title: "Doing task",
        status: "in-progress",
        priority: "high",
        project: "UMBRA",
        projectId: "p1",
        columnId: "c-progress",
        columnKind: "in_progress",
      },
      {
        id: "t-done",
        title: "Done task",
        status: "done",
        priority: "low",
        project: "UMBRA",
        projectId: "p1",
        columnId: "c-done",
        columnKind: "done",
        description: "Should start collapsed",
      },
      {
        id: "t-review",
        title: "Review task",
        status: "in-progress",
        priority: "critical",
        project: "UMBRA",
        projectId: "p1",
        columnId: "c-review",
        columnKind: "review",
        description: "Should start hidden by collapsed review lane",
      },
    ];

    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks,
        loading: false,
        error: null,
        lastSync: "2026-03-20T10:00:00.000Z",
        fetchTasks: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          taskLanePrefs: {},
        },
        saveConfig: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "get_pm_projects") {
        return [{ id: "p1", name: "UMBRA" }];
      }
      if (command === "get_pm_columns") {
        return [
          { id: "c-backlog", name: "Backlog", kind: "backlog", projectId: "p1" },
          { id: "c-progress", name: "Doing", kind: "in_progress", projectId: "p1" },
          { id: "c-review", name: "Review", kind: "review", projectId: "p1" },
          { id: "c-done", name: "Done", kind: "done", projectId: "p1" },
        ];
      }
      return null;
    });
  });

  it("defaults to the UMBRA project, supports drag across lanes, exposes priority sorting and collapses done content by default", async () => {
    const wrapper = mount(TasksView, {
      global: {
        stubs: {
          Teleport: true,
          Transition: false,
        },
      },
    });

    await flushPromises();

    expect(wrapper.text()).toContain("SORT BY PRIORITY");

    const backlogCard = wrapper.findAll(".task-card").find((card) => card.text().includes("Backlog task"));
    const doingCard = wrapper.findAll(".task-card").find((card) => card.text().includes("Doing task"));

    expect(backlogCard?.attributes("draggable")).toBe("true");

    const reviewLaneToggle = wrapper.findAll(".lane-toggle").at(2);
    expect(reviewLaneToggle?.text()).toBe("+");
    expect(wrapper.text()).not.toContain("Review task");

    const doneLaneToggle = wrapper.findAll(".lane-toggle").at(3);
    expect(doneLaneToggle?.text()).toBe("+");
    expect(wrapper.text()).toContain("expand lane");
    expect(wrapper.text()).toContain("1 hidden");

    await reviewLaneToggle!.trigger("click");
    await flushPromises();

    const reviewCard = wrapper.findAll(".task-card").find((card) => card.text().includes("Review task"));
    expect(reviewCard).toBeTruthy();

    await doneLaneToggle!.trigger("click");
    await flushPromises();

    const doneCard = wrapper.findAll(".task-card").find((card) => card.text().includes("Done task"));
    expect(doneCard).toBeTruthy();
    expect(doneCard?.text()).not.toContain("Should start collapsed");

    const collapseButton = doneCard!.find(".collapse-btn");
    expect(collapseButton.text()).toBe("+");
    await collapseButton.trigger("click");
    expect(doneCard?.text()).toContain("Should start collapsed");

    await backlogCard!.trigger("dragstart");
    await doingCard!.trigger("dragover");
    await doingCard!.trigger("drop");
    await flushPromises();

    expect(mocks.invoke).toHaveBeenCalledWith("move_pm_task", {
      taskId: "t-backlog",
      columnId: "c-progress",
    });
    expect(mocks.invoke).toHaveBeenCalledWith("reorder_pm_tasks", {
      columnId: "c-progress",
      taskIds: ["t-backlog", "t-doing"],
    });

    const sortButton = wrapper.findAll("button").find((button) => button.text().includes("SORT BY PRIORITY"));
    expect(sortButton).toBeTruthy();
    await sortButton!.trigger("click");
    await flushPromises();

    expect(mocks.invoke).toHaveBeenCalledWith("reorder_pm_tasks", {
      columnId: "c-progress",
      taskIds: ["t-doing", "t-backlog"],
    });
  });

  it("auto-collapses large backlog lanes but keeps the toggle overridable", async () => {
    const saveConfig = vi.fn().mockResolvedValue(undefined);
    const backlogHeavy = Array.from({ length: 6 }, (_, index) => ({
      id: `b-${index}`,
      title: `Backlog ${index + 1}`,
      status: "todo" as const,
      priority: "medium" as const,
      project: "UMBRA",
      projectId: "p1",
      columnId: "c-backlog",
      columnKind: "backlog" as const,
    }));

    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks: backlogHeavy,
        loading: false,
        error: null,
        lastSync: "2026-03-20T10:00:00.000Z",
        fetchTasks: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          taskLanePrefs: {},
        },
        saveConfig,
      })
    );

    const wrapper = mount(TasksView, {
      global: {
        stubs: {
          Teleport: true,
          Transition: false,
        },
      },
    });

    await flushPromises();

    const backlogLaneToggle = wrapper.findAll(".lane-toggle").at(0);
    expect(backlogLaneToggle?.text()).toBe("+");
    expect(wrapper.text()).toContain("6 hidden");
    expect(wrapper.text()).toContain("expand lane");

    await backlogLaneToggle!.trigger("click");
    await flushPromises();

    expect(backlogLaneToggle?.text()).toBe("-");
    expect(wrapper.text()).toContain("Backlog 1");
    expect(saveConfig).toHaveBeenCalled();
  });
});
