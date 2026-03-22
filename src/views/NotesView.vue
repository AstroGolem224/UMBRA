<template>
  <div class="notes-view">
    <ViewHero
      kicker="vault"
      title="Notes"
      subtitle="quiet note routing across the vault, autosave and category slices."
    >
      <template #meta>
        <span class="view-hero-pill">{{ notesStore.filteredNotes.length }} visible</span>
        <span class="view-hero-pill" :class="{ 'is-stale': !activeNote }">
          {{ activeNote ? activeNote.category : "no note selected" }}
        </span>
        <NeonButton size="sm" variant="primary" @click="createNote">+ new</NeonButton>
      </template>
    </ViewHero>

    <div class="notes-shell">
      <aside class="notes-sidebar">
        <div class="sidebar-header">
          <div>
            <p class="sidebar-kicker">vault index</p>
            <span class="sidebar-title">all notes</span>
          </div>
        </div>

        <input
          v-model="notesStore.searchQuery"
          class="search-input glass-input"
          placeholder="search notes..."
        />

        <div class="category-filters">
          <button
            class="cat-btn"
            :class="{ active: notesStore.activeCategory === null }"
            @click="notesStore.activeCategory = null"
          >all</button>
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
            <span class="note-item-title">{{ note.title || "untitled" }}</span>
            <span class="note-item-cat">{{ note.category }}</span>
          </button>
          <div v-if="notesStore.filteredNotes.length === 0" class="empty-list">no notes found</div>
        </div>
      </aside>

      <section class="editor-shell">
        <div v-if="activeNote" class="editor-frame">
          <NoteEditor
            :note="activeNote"
            :saving="saving"
            :autosaving="autosaving"
            :autosave-state="autosaveState"
            @change="patchNote"
            @save="saveNote"
            @delete="deleteNote"
          />
        </div>
        <div v-else class="empty-editor">
          <p class="empty-title">select or create a note</p>
          <p class="empty-copy">the vault stays quiet until you open a note.</p>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { Note, NoteCategory } from "@/interfaces";
import ViewHero from "@/components/layout/ViewHero.vue";
import NoteEditor from "@/components/notes/NoteEditor.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import { useNotesStore } from "@/stores/useNotesStore";

const notesStore = useNotesStore();
const saving = ref(false);
const autosaving = ref(false);
const autosaveState = ref<string | null>(null);
let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

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
    autosaveState.value = "saved";
  } finally {
    saving.value = false;
  }
}

async function deleteNote() {
  if (!activeNote.value) return;
  await notesStore.deleteNote(activeNote.value.id);
}

watch(
  () =>
    activeNote.value
      ? [
          activeNote.value.id,
          activeNote.value.title,
          activeNote.value.content,
          activeNote.value.category,
          activeNote.value.tags.join("|"),
        ]
      : null,
  () => {
    if (!activeNote.value) return;
    if (autosaveTimer) clearTimeout(autosaveTimer);

    const note = activeNote.value;
    const hasContent =
      Boolean(note.filePath) ||
      Boolean(note.title.trim()) ||
      Boolean(note.content.trim()) ||
      note.tags.length > 0;

    if (!hasContent) {
      autosaveState.value = null;
      return;
    }

    autosaveState.value = "pending";
    autosaveTimer = setTimeout(async () => {
      if (!activeNote.value) return;
      autosaving.value = true;
      try {
        await notesStore.saveNote(activeNote.value);
        autosaveState.value = "autosaved";
      } finally {
        autosaving.value = false;
      }
    }, 700);
  }
);

onMounted(() => {
  notesStore.loadNotes();
});

onBeforeUnmount(() => {
  if (autosaveTimer) clearTimeout(autosaveTimer);
});
</script>

<style scoped>
.notes-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.notes-shell {
  display: grid;
  grid-template-columns: 292px minmax(0, 1fr);
  gap: 16px;
  min-height: calc(100vh - 132px);
}

.notes-sidebar,
.editor-frame,
.empty-editor {
  border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
  border-radius: var(--radius-xl);
  background: color-mix(in srgb, var(--glass-bg) 88%, transparent);
}

.notes-sidebar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  min-height: 100%;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.sidebar-kicker,
.note-item-cat {
  margin: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.sidebar-title,
.empty-title {
  color: var(--text-primary);
  font-family: var(--font-display);
}

.sidebar-title {
  font-size: 26px;
  font-weight: 800;
  letter-spacing: 0.06em;
  line-height: 1;
}

.search-input {
  font-size: 12px;
  padding: 9px 12px;
}

.category-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.cat-btn {
  padding: 6px 10px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 80%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
}

.cat-btn.active,
.cat-btn:hover {
  border-color: color-mix(in srgb, var(--accent) 28%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--accent);
}

.note-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.note-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 11px 12px;
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  width: 100%;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease;
}

.note-item:hover {
  background: color-mix(in srgb, var(--glass-bg) 70%, transparent);
  border-color: color-mix(in srgb, var(--glass-border) 88%, transparent);
}

.note-item.active {
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent) 8%, transparent), transparent 72%),
    color-mix(in srgb, var(--glass-bg) 84%, transparent);
  border-color: color-mix(in srgb, var(--accent) 24%, transparent);
}

.note-item-title {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.4;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-list,
.empty-copy {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.editor-shell {
  min-width: 0;
}

.editor-frame {
  padding: 16px;
  height: 100%;
}

.empty-editor {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  text-align: center;
  padding: 24px;
}

.empty-title {
  margin: 0;
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

:global([data-theme="light"]) .notes-sidebar,
:global([data-theme="light"]) .editor-frame,
:global([data-theme="light"]) .empty-editor {
  background: rgba(255, 255, 255, 0.9);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .note-item:hover {
  background: rgba(255, 255, 255, 0.86);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .note-item.active {
  background:
    linear-gradient(135deg, rgba(8, 145, 178, 0.08), transparent 72%),
    rgba(255, 255, 255, 0.92);
  border-color: rgba(8, 145, 178, 0.18);
}

@media (max-width: 980px) {
  .notes-view {
    grid-template-columns: 1fr;
  }

  .notes-sidebar {
    min-height: 320px;
  }
}
</style>
