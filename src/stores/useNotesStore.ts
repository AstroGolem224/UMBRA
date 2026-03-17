import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Note, NoteCategory } from "@/interfaces";

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

  async function loadNotes() {
    loading.value = true;
    error.value = null;
    try {
      notes.value = await invoke<Note[]>("list_notes");
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
  }

  function newNote(category: NoteCategory = "misc"): Note {
    const note: Note = {
      id: crypto.randomUUID(),
      title: "",
      content: "",
      category,
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
    loadNotes,
    saveNote,
    deleteNote,
    newNote,
  };
});
