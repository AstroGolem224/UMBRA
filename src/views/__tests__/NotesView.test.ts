import { mount, flushPromises } from "@vue/test-utils";
import { nextTick, reactive } from "vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Note } from "@/interfaces";

const mocks = vi.hoisted(() => ({
  useNotesStore: vi.fn(),
  useConfigStore: vi.fn(),
  useTaskStore: vi.fn(),
  useAgentStore: vi.fn(),
  useGithubStore: vi.fn(),
}));

vi.mock("@/stores/useNotesStore", () => ({
  useNotesStore: mocks.useNotesStore,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

vi.mock("@/stores/useTaskStore", () => ({
  useTaskStore: mocks.useTaskStore,
}));

vi.mock("@/stores/useAgentStore", () => ({
  useAgentStore: mocks.useAgentStore,
}));

vi.mock("@/stores/useGithubStore", () => ({
  useGithubStore: mocks.useGithubStore,
}));

import NotesView from "../NotesView.vue";

describe("NotesView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
          githubTargets: [],
          launchTargets: [],
        },
        setTheme: vi.fn(),
      })
    );
    mocks.useTaskStore.mockReturnValue(
      reactive({
        tasks: [],
        fetchTasks: vi.fn().mockResolvedValue(undefined),
      })
    );
    mocks.useAgentStore.mockReturnValue(
      reactive({
        agents: [],
        loadAgents: vi.fn().mockResolvedValue(undefined),
      })
    );
    mocks.useGithubStore.mockReturnValue(
      reactive({
        repos: [],
        loadRepos: vi.fn().mockResolvedValue(undefined),
      })
    );

    const note: Note = {
      id: "note-1",
      title: "",
      content: "",
      category: "misc",
      tags: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      filePath: "",
    };

    const store = reactive({
      notes: [note],
      activeNoteId: "note-1",
      activeCategory: null as Note["category"] | null,
      availableCategories: ["misc"],
      searchQuery: "",
      loading: false,
      error: null as string | null,
      loadNotes: vi.fn().mockResolvedValue(undefined),
      saveNote: vi.fn().mockResolvedValue(note),
      deleteNote: vi.fn().mockResolvedValue(undefined),
      newNote: vi.fn(() => note),
    }) as any;

    Object.defineProperty(store, "filteredNotes", {
      get() {
        return store.notes.filter((entry: Note) => {
          const matchesCategory = !store.activeCategory || entry.category === store.activeCategory;
          const query = String(store.searchQuery ?? "").toLowerCase();
          const matchesSearch =
            !query ||
            entry.title.toLowerCase().includes(query) ||
            entry.content.toLowerCase().includes(query) ||
            entry.tags.some((tag: string) => tag.toLowerCase().includes(query));
          return matchesCategory && matchesSearch;
        });
      },
    });

    mocks.useNotesStore.mockReturnValue(store);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads notes on mount and autosaves after an editor change", async () => {
    const wrapper = mount(NotesView, {
      global: {
        stubs: {
          NoteEditor: {
            template: `<button class="emit-change" @click="$emit('change', { title: 'Changed Title', tags: ['umbra'] })">emit</button>`,
          },
        },
      },
    });

    await flushPromises();

    const store = mocks.useNotesStore.mock.results[0].value;
    expect(store.loadNotes).toHaveBeenCalledTimes(1);

    await wrapper.find(".emit-change").trigger("click");
    await nextTick();
    await vi.advanceTimersByTimeAsync(750);
    await flushPromises();

    expect(store.saveNote).toHaveBeenCalledTimes(1);
    expect(store.saveNote.mock.calls[0][0].title).toBe("Changed Title");
    expect(store.saveNote.mock.calls[0][0].tags).toEqual(["umbra"]);
  });

  it("uses a category dropdown and seeds new notes from the active category", async () => {
    const wrapper = mount(NotesView, {
      global: {
        stubs: {
          NoteEditor: true,
        },
      },
    });

    await flushPromises();

    const store = mocks.useNotesStore.mock.results[0].value;
    store.notes.push({
      ...store.notes[0],
      id: "note-2",
      category: "field ops",
      title: "ops",
    });
    store.availableCategories = ["field ops", "misc"];
    await nextTick();

    const filter = wrapper.get("#notes-category-filter");
    expect(filter.findAll("option").map((option) => option.text())).toEqual([
      "all categories",
      "field ops",
      "misc",
    ]);

    await filter.setValue("field ops");
    expect(store.activeCategory).toBe("field ops");

    await wrapper.get(".view-hero .neon-btn").trigger("click");
    expect(store.newNote).toHaveBeenCalledWith("field ops");
  });
});
