import { flushPromises, mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Note } from "@/interfaces";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  push: vi.fn(),
  loadAgents: vi.fn().mockResolvedValue(undefined),
  fetchTasks: vi.fn().mockResolvedValue(undefined),
}));

const mermaidMock = vi.hoisted(() => ({
  initialize: vi.fn(),
  run: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("vue-router", () => ({
  useRouter: () => ({
    push: mocks.push,
  }),
}));

vi.mock("@/stores/useAgentStore", () => ({
  useAgentStore: () => ({
    agents: [],
    loadAgents: mocks.loadAgents,
  }),
}));

vi.mock("@/stores/useTaskStore", () => ({
  useTaskStore: () => ({
    tasks: [],
    fetchTasks: mocks.fetchTasks,
  }),
}));

vi.mock("mermaid", () => ({
  default: mermaidMock,
}));

import NoteEditor from "../NoteEditor.vue";

function makeNote(content = "## hello") : Note {
  const now = new Date().toISOString();
  return {
    id: "note-1",
    title: "Test note",
    content,
    category: "misc",
    tags: [],
    createdAt: now,
    updatedAt: now,
    filePath: "D:/Obsidian/UMBRA_Notes/misc/note-1.md",
  };
}

describe("NoteEditor", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("switches between markdown, split and preview modes", async () => {
    const wrapper = mount(NoteEditor, {
      props: {
        note: makeNote(),
        categories: ["misc", "agents"],
      },
    });

    expect(wrapper.find("textarea.raw-editor").exists()).toBe(true);
    expect(wrapper.find(".preview").exists()).toBe(true);

    await wrapper.findAll(".view-mode-btn")[2].trigger("click");
    await nextTick();

    expect(wrapper.find("textarea.raw-editor").exists()).toBe(false);
    expect(wrapper.find(".preview").exists()).toBe(true);

    await wrapper.findAll(".view-mode-btn")[0].trigger("click");
    await nextTick();

    expect(wrapper.find("textarea.raw-editor").exists()).toBe(true);
    expect(wrapper.find(".preview").exists()).toBe(false);

    await wrapper.findAll(".view-mode-btn")[1].trigger("click");
    await nextTick();

    expect(wrapper.find("textarea.raw-editor").exists()).toBe(true);
    expect(wrapper.find(".preview").exists()).toBe(true);
  });

  it("runs mermaid rendering for fenced mermaid blocks", async () => {
    const wrapper = mount(NoteEditor, {
      props: {
        note: makeNote("```mermaid\ngraph TD\n  A[write] --> B[preview]\n```"),
        categories: ["misc"],
      },
    });

    await flushPromises();
    await nextTick();

    expect(mermaidMock.initialize).toHaveBeenCalledTimes(1);
    expect(mermaidMock.run).toHaveBeenCalledTimes(1);
    expect(wrapper.find(".preview").html()).toContain("class=\"mermaid\"");
  });

  it("lets you create a custom category inline", async () => {
    const wrapper = mount(NoteEditor, {
      props: {
        note: makeNote(),
        categories: ["misc"],
      },
    });

    await wrapper.get('[data-test="add-category"]').trigger("click");
    await wrapper.get('[data-test="category-input"]').setValue("field ops");
    await wrapper.get('[data-test="apply-category"]').trigger("click");
    await nextTick();

    const emissions = wrapper.emitted("change");
    expect(emissions?.at(-1)?.[0]).toMatchObject({ category: "field ops" });
    expect(wrapper.get('[data-test="category-select"]').text()).toContain("field ops");
  });

  it("inserts structured quick links and renders them as preview chips", async () => {
    const wrapper = mount(NoteEditor, {
      props: {
        note: makeNote(""),
        categories: ["misc"],
        quickLinkGroups: [
          {
            id: "tasks",
            label: "tasks",
            options: [
              {
                id: "task-1",
                kind: "task",
                label: "ship updater",
                description: "UMBRA / in-progress",
              },
            ],
          },
        ],
      },
    });

    await wrapper.get('[data-test="quick-link-select"]').setValue("task:task-1");
    await nextTick();

    const emissions = wrapper.emitted("change");
    expect(emissions?.at(-1)?.[0]).toMatchObject({
      content: expect.stringContaining("umbra://task/task-1"),
    });
    expect(wrapper.find(".preview").text()).toContain("ship updater");
    expect(wrapper.find(".preview").html()).toContain("umbra-chip");
  });

  it("saves file attachments through tauri and appends markdown to the note", async () => {
    mocks.invoke.mockResolvedValue({
      fileName: "diagram.png",
      absolutePath: "D:/vault/UMBRA_Notes/_attachments/note-1/diagram.png",
      relativePath: "../_attachments/note-1/diagram.png",
      markdown: "![diagram](../_attachments/note-1/diagram.png)",
      isImage: true,
    });

    const wrapper = mount(NoteEditor, {
      props: {
        note: makeNote(""),
        categories: ["misc"],
      },
    });

    const file = new File(["pixel"], "diagram.png", { type: "image/png" });
    Object.defineProperty(file, "arrayBuffer", {
      configurable: true,
      value: vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3]).buffer),
    });
    const input = wrapper.get('[data-test="attachment-input"]').element as HTMLInputElement;
    Object.defineProperty(input, "files", {
      configurable: true,
      value: [file],
    });

    await wrapper.get('[data-test="attachment-input"]').trigger("change");
    await flushPromises();

    expect(mocks.invoke).toHaveBeenCalledWith("save_note_attachment", {
      noteId: "note-1",
      category: "misc",
      fileName: "diagram.png",
      bytes: expect.any(Array),
      mimeType: "image/png",
    });

    const emissions = wrapper.emitted("change");
    expect(emissions?.at(-1)?.[0]).toMatchObject({
      content: expect.stringContaining("![diagram](../_attachments/note-1/diagram.png)"),
    });
  });
});
