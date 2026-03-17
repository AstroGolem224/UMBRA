<template>
  <div class="notes-view">
    <aside class="notes-sidebar">
      <div class="sidebar-header">
        <span class="sidebar-title">NOTES</span>
        <NeonButton size="sm" @click="createNote">+ NEW</NeonButton>
      </div>

      <input
        v-model="notesStore.searchQuery"
        class="search-input glass-input"
        placeholder="Search notes..."
      />

      <div class="category-filters">
        <button
          class="cat-btn"
          :class="{ active: notesStore.activeCategory === null }"
          @click="notesStore.activeCategory = null"
        >ALL</button>
        <button
          v-for="cat in categories"
          :key="cat"
          class="cat-btn"
          :class="{ active: notesStore.activeCategory === cat }"
          @click="notesStore.activeCategory = cat"
        >{{ cat }}</button>
      </div>

      <div class="note-list">
        <button
          v-for="note in notesStore.filteredNotes"
          :key="note.id"
          class="note-item"
          :class="{ active: notesStore.activeNoteId === note.id }"
          @click="selectNote(note.id)"
        >
          <span class="note-item-title">{{ note.title || "Untitled" }}</span>
          <span class="note-item-cat">{{ note.category }}</span>
        </button>
        <div v-if="notesStore.filteredNotes.length === 0" class="empty-list">No notes found</div>
      </div>
    </aside>

    <div class="editor-area">
      <NoteEditor
        v-if="activeNote"
        :note="activeNote"
        :saving="saving"
        @change="patchNote"
        @save="saveNote"
        @delete="deleteNote"
      />
      <div v-else class="empty-editor">
        <span>Select or create a note</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useNotesStore } from "@/stores/useNotesStore";
import NoteEditor from "@/components/notes/NoteEditor.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import type { Note, NoteCategory } from "@/interfaces";

const notesStore = useNotesStore();
const saving = ref(false);

const categories: NoteCategory[] = ["prompts", "cli", "agents", "skills", "misc"];

const activeNote = computed(() =>
  notesStore.notes.find((n) => n.id === notesStore.activeNoteId) ?? null
);

function selectNote(id: string) {
  notesStore.activeNoteId = id;
}

function createNote() {
  const note = notesStore.newNote();
  notesStore.activeNoteId = note.id;
}

function patchNote(patch: Partial<Note>) {
  if (!notesStore.activeNoteId) return;
  const idx = notesStore.notes.findIndex((n) => n.id === notesStore.activeNoteId);
  if (idx !== -1) {
    notesStore.notes[idx] = { ...notesStore.notes[idx], ...patch };
  }
}

async function saveNote() {
  if (!activeNote.value) return;
  saving.value = true;
  try {
    await notesStore.saveNote(activeNote.value);
  } finally {
    saving.value = false;
  }
}

async function deleteNote() {
  if (!activeNote.value) return;
  await notesStore.deleteNote(activeNote.value.id);
}
</script>

<style scoped>
.notes-view {
  display: flex;
  gap: 0;
  height: calc(100vh - 80px);
  margin: -20px;
}

.notes-sidebar {
  width: 240px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 20px 12px;
  border-right: 1px solid var(--glass-border);
  background: var(--bg-surface);
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sidebar-title {
  font-family: "Iceland", monospace;
  font-size: 13px;
  letter-spacing: 0.18em;
  color: var(--text-muted);
}

.search-input {
  font-size: 12px;
  padding: 6px 10px;
}

.category-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cat-btn {
  font-family: "Iceland", monospace;
  font-size: 10px;
  letter-spacing: 0.08em;
  padding: 2px 8px;
  border-radius: 3px;
  border: 1px solid var(--glass-border);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.12s;
}

.cat-btn.active,
.cat-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-dim);
}

.note-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.note-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  width: 100%;
  text-align: left;
  transition: all 0.12s;
}

.note-item:hover {
  background: var(--bg-surface-hover);
  border-color: var(--glass-border);
}

.note-item.active {
  background: var(--accent-dim);
  border-color: var(--accent);
}

.note-item-title {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

.note-item-cat {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 0.06em;
}

.empty-list {
  font-size: 12px;
  color: var(--text-muted);
  padding: 8px 10px;
}

.editor-area {
  flex: 1;
  padding: 20px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.empty-editor {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  font-size: 13px;
  color: var(--text-muted);
  letter-spacing: 0.08em;
}
</style>
