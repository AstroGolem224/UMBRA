import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Note, NoteCategory } from "@/interfaces";

const DEFAULT_NOTE_CATEGORY = "misc";

function generateNoteId() {
  if (typeof globalThis.crypto?.randomUUID === "function") {
    return globalThis.crypto.randomUUID();
  }
  return `note-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export const useNotesStore = defineStore("notes", () => {
  const notes = ref<Note[]>([]);
  const activeNoteId = ref<string | null>(null);
  const activeCategory = ref<NoteCategory | null>(null);
  const searchQuery = ref("");
  const loading = ref(false);
  const error = ref<string | null>(null);

  const filteredNotes = computed(() => {
    let result = notes.value;
    if (activeCategory.value) {
      result = result.filter((n) => n.category === activeCategory.value);
    }
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(
        (n) =>
          n.title.toLowerCase().includes(q) ||
          n.content.toLowerCase().includes(q) ||
          n.tags.some((t) => t.toLowerCase().includes(q))
      );
    }
    return result;
  });

  const availableCategories = computed(() =>
    Array.from(
      new Set(
        notes.value
          .map((note) => note.category.trim())
          .filter(Boolean)
      )
    ).sort((a, b) => a.localeCompare(b))
  );

  function resetActiveCategoryIfMissing() {
    if (!activeCategory.value) return;
    if (!notes.value.some((note) => note.category === activeCategory.value)) {
      activeCategory.value = null;
    }
  }

  async function loadNotes() {
    loading.value = true;
    error.value = null;
    try {
      notes.value = await invoke<Note[]>("list_notes");
      resetActiveCategoryIfMissing();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function saveNote(note: Note) {
    error.value = null;
    const saved = await invoke<Note>("save_note", { note });
    const idx = notes.value.findIndex((n) => n.id === saved.id);
    if (idx >= 0) {
      notes.value[idx] = saved;
    } else {
      notes.value.unshift(saved);
      activeNoteId.value = saved.id;
    }
    resetActiveCategoryIfMissing();
    return saved;
  }

  async function deleteNote(id: string) {
    const note = notes.value.find((n) => n.id === id);
    if (!note) return;
    error.value = null;
    await invoke("delete_note", { id, category: note.category });
    notes.value = notes.value.filter((n) => n.id !== id);
    if (activeNoteId.value === id) {
      activeNoteId.value = null;
    }
    resetActiveCategoryIfMissing();
  }

  function newNote(category?: NoteCategory): Note {
    const nextCategory =
      category?.trim() ||
      activeCategory.value?.trim() ||
      availableCategories.value[0] ||
      DEFAULT_NOTE_CATEGORY;
    const note: Note = {
      id: generateNoteId(),
      title: "",
      content: "",
      category: nextCategory,
      tags: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      filePath: "",
    };
    notes.value.unshift(note);
    activeNoteId.value = note.id;
    return note;
  }

  return {
    notes,
    activeNoteId,
    activeCategory,
    searchQuery,
    loading,
    error,
    filteredNotes,
    availableCategories,
    loadNotes,
    saveNote,
    deleteNote,
    newNote,
  };
});
