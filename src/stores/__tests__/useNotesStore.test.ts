import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useNotesStore } from "../useNotesStore";
import type { Note } from "@/interfaces";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

const makeNote = (overrides: Partial<Note> = {}): Note => ({
  id: "test-id",
  title: "Test Note",
  content: "Default content for testing",
  category: "prompts",
  tags: [],
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  filePath: "D:/vault/UMBRA_Notes/prompts/test-id.md",
  ...overrides,
});

describe("useNotesStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("loads notes via invoke", async () => {
    const notes = [makeNote(), makeNote({ id: "note-2", title: "Second" })];
    mockInvoke.mockResolvedValueOnce(notes);

    const store = useNotesStore();
    await store.loadNotes();

    expect(store.notes).toHaveLength(2);
    expect(mockInvoke).toHaveBeenCalledWith("list_notes");
  });

  it("filteredNotes filters by category", async () => {
    const store = useNotesStore();
    store.notes = [
      makeNote({ id: "1", category: "prompts" }),
      makeNote({ id: "2", category: "cli" }),
      makeNote({ id: "3", category: "prompts" }),
    ];

    store.activeCategory = "prompts";
    expect(store.filteredNotes).toHaveLength(2);

    store.activeCategory = "cli";
    expect(store.filteredNotes).toHaveLength(1);
  });

  it("filteredNotes filters by search query", async () => {
    const store = useNotesStore();
    store.notes = [
      makeNote({ id: "1", title: "Hello World" }),
      makeNote({ id: "2", title: "Goodbye" }),
    ];

    store.searchQuery = "hello";
    expect(store.filteredNotes).toHaveLength(1);
    expect(store.filteredNotes[0].id).toBe("1");
  });

  it("newNote returns a note with a new id and sets activeNoteId", () => {
    const store = useNotesStore();
    const note = store.newNote();

    expect(note.id).toBeTruthy();
    expect(note.category).toBe("misc");
    expect(store.notes).toHaveLength(1);
    expect(store.activeNoteId).toBe(note.id);
  });

  it("derives available categories and uses the active one for new notes", () => {
    const store = useNotesStore();
    store.notes = [
      makeNote({ id: "1", category: "field ops" }),
      makeNote({ id: "2", category: "misc" }),
      makeNote({ id: "3", category: "field ops" }),
    ];

    expect(store.availableCategories).toEqual(["field ops", "misc"]);

    store.activeCategory = "field ops";
    const note = store.newNote();
    expect(note.category).toBe("field ops");
  });

  it("saveNote calls invoke and updates note in list", async () => {
    const store = useNotesStore();
    const note = makeNote();
    store.notes = [note];

    const updated = { ...note, title: "Updated", updatedAt: new Date().toISOString() };
    mockInvoke.mockResolvedValueOnce(updated);

    await store.saveNote(note);

    expect(mockInvoke).toHaveBeenCalledWith("save_note", { note });
    expect(store.notes[0].title).toBe("Updated");
  });

  it("deleteNote calls invoke and removes note from list", async () => {
    const store = useNotesStore();
    const note = makeNote();
    store.notes = [note];
    mockInvoke.mockResolvedValueOnce(undefined);

    await store.deleteNote(note.id);

    expect(mockInvoke).toHaveBeenCalledWith("delete_note", {
      id: note.id,
      category: note.category,
    });
    expect(store.notes).toHaveLength(0);
    expect(store.activeNoteId).toBeNull();
  });
});
