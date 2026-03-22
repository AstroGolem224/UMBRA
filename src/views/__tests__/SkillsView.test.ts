import { mount, flushPromises } from "@vue/test-utils";
import { nextTick, reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  useConfigStore: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

import SkillsView from "../SkillsView.vue";

describe("SkillsView", () => {
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
    mocks.invoke.mockResolvedValue([
      {
        id: "qa",
        name: "QA",
        version: "1.0.0",
        description: "Systematic QA",
        category: "gstack",
        agents: ["forge", "jim"],
        content: "qa content",
        folder: "gstack/qa",
      },
      {
        id: "ship",
        name: "Ship",
        version: "1.1.0",
        description: "Release workflow",
        category: "core",
        agents: ["jim"],
        content: "ship content",
        folder: "ship",
      },
      {
        id: "godot",
        name: "Godot",
        version: "0.9.0",
        description: "Game skill",
        category: "games",
        agents: ["prism"],
        content: "godot content",
        folder: "games/godot",
      },
    ]);
  });

  it("loads skills, supports keyboard navigation and category-agent filtering", async () => {
    const wrapper = mount(SkillsView, {
      global: {
        stubs: {
          Transition: false,
        },
      },
    });

    await flushPromises();

    expect(wrapper.find(".view-hero__subtitle").text()).toContain("3 indexed skills");

    const cards = wrapper.findAll(".skill-card");
    await cards[0].trigger("click");
    expect(wrapper.text()).toContain("QA");
    expect(wrapper.text()).toContain("qa content");

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight" }));
    await nextTick();
    expect(wrapper.text()).toContain("Ship");
    expect(wrapper.text()).toContain("ship content");

    const categoryButton = wrapper.findAll(".filter-pill").find((button) => button.text() === "gstack");
    const agentButton = wrapper.findAll(".filter-pill").find((button) => button.text() === "jim");

    await categoryButton!.trigger("click");
    await agentButton!.trigger("click");
    await nextTick();

    expect(wrapper.findAll(".skill-card")).toHaveLength(1);
    expect(wrapper.text()).toContain("QA");
    expect(wrapper.text()).not.toContain("Godot");
  });
});
