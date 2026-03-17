<template>
  <div class="note-editor">
    <div class="editor-toolbar">
      <input
        v-model="localTitle"
        class="title-input glass-input"
        placeholder="Note title..."
        @input="emitChange"
      />
      <select v-model="localCategory" class="category-select glass-input" @change="emitChange">
        <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
      </select>
      <NeonButton variant="primary" size="sm" :loading="saving" @click="$emit('save')">SAVE</NeonButton>
      <NeonButton v-if="note.id" variant="danger" size="sm" ghost @click="$emit('delete')">DELETE</NeonButton>
    </div>

    <div class="editor-panes">
      <textarea
        v-model="localContent"
        class="editor-pane raw-editor glass-input"
        placeholder="Write markdown here..."
        spellcheck="false"
        @input="emitChange"
      />
      <div class="editor-pane preview" v-html="renderedHtml" />
    </div>

    <div class="editor-meta">
      <span v-if="note.updatedAt" class="meta-text">
        Last saved: {{ new Date(note.updatedAt).toLocaleString() }}
      </span>
      <span v-if="note.filePath" class="meta-text">{{ note.filePath }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { marked } from "marked";
import DOMPurify from "dompurify";
import type { Note, NoteCategory } from "@/interfaces";
import NeonButton from "@/components/ui/NeonButton.vue";

const props = defineProps<{
  note: Note;
  saving?: boolean;
}>();

const emit = defineEmits<{
  change: [patch: Partial<Note>];
  save: [];
  delete: [];
}>();

const categories: NoteCategory[] = ["prompts", "cli", "agents", "skills", "misc"];

const localTitle = ref(props.note.title);
const localContent = ref(props.note.content);
const localCategory = ref<NoteCategory>(props.note.category);

watch(
  () => props.note,
  (n) => {
    localTitle.value = n.title;
    localContent.value = n.content;
    localCategory.value = n.category;
  }
);

const renderedHtml = computed(() => {
  const raw = marked.parse(localContent.value ?? "") as string;
  return DOMPurify.sanitize(raw);
});

function emitChange() {
  emit("change", {
    title: localTitle.value,
    content: localContent.value,
    category: localCategory.value,
  });
}
</script>

<style scoped>
.note-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 12px;
}

.editor-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.title-input {
  flex: 1;
  font-family: "Iceland", monospace;
  font-size: 14px;
  letter-spacing: 0.06em;
}

.category-select {
  font-family: "Iceland", monospace;
  font-size: 12px;
  letter-spacing: 0.08em;
  padding: 6px 10px;
  background: var(--glass-bg);
  color: var(--text-primary);
  border: 1px solid var(--glass-border);
  border-radius: 6px;
  cursor: pointer;
}
.category-select option {
  background: var(--bg-surface);
}

.editor-panes {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.editor-pane {
  border-radius: 8px;
  overflow-y: auto;
}

.raw-editor {
  font-family: "JetBrains Mono", "Consolas", monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  padding: 14px;
  color: var(--text-primary);
}

.preview {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  padding: 14px;
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-secondary);
}

.preview :deep(h1),
.preview :deep(h2),
.preview :deep(h3) {
  font-family: "Iceland", monospace;
  color: var(--text-primary);
  letter-spacing: 0.08em;
}

.preview :deep(code) {
  background: var(--bg-surface);
  padding: 1px 5px;
  border-radius: 3px;
  font-size: 12px;
  color: var(--accent);
}

.preview :deep(pre) {
  background: var(--bg-surface);
  padding: 12px;
  border-radius: 6px;
  border-left: 2px solid var(--accent);
  overflow-x: auto;
}

.editor-meta {
  display: flex;
  justify-content: space-between;
  flex-shrink: 0;
}

.meta-text {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 0.06em;
}
</style>
