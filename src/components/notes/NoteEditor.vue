<template>
  <div class="note-editor">
    <div class="editor-toolbar">
      <div class="toolbar-head">
        <input
          v-model="localTitle"
          class="title-input glass-input"
          placeholder="note title..."
          @input="emitChange"
        />
        <div class="toolbar-actions">
          <select v-model="localCategory" class="category-select glass-input" @change="emitChange">
            <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
          </select>
          <NeonButton variant="primary" size="sm" :loading="saving" @click="$emit('save')">save</NeonButton>
          <NeonButton v-if="note.id" variant="danger" size="sm" ghost @click="$emit('delete')">delete</NeonButton>
        </div>
      </div>

      <input
        v-model="localTags"
        class="tags-input glass-input"
        placeholder="tags, comma separated..."
        @input="emitChange"
      />
    </div>

    <div class="editor-panes">
      <textarea
        v-model="localContent"
        class="editor-pane raw-editor glass-input"
        placeholder="write markdown here..."
        spellcheck="false"
        @input="emitChange"
      />
      <div class="editor-pane preview" v-html="renderedHtml" />
    </div>

    <div class="editor-meta">
      <span v-if="note.updatedAt" class="meta-text">
        last saved: {{ new Date(note.updatedAt).toLocaleString() }}
      </span>
      <span v-if="autosaving" class="meta-text accent">autosaving...</span>
      <span v-else-if="autosaveState" class="meta-text accent">{{ autosaveState }}</span>
      <span v-if="note.filePath" class="meta-text path">{{ note.filePath }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import DOMPurify from "dompurify";
import { marked } from "marked";
import type { Note, NoteCategory } from "@/interfaces";
import NeonButton from "@/components/ui/NeonButton.vue";

const props = defineProps<{
  note: Note;
  saving?: boolean;
  autosaving?: boolean;
  autosaveState?: string | null;
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
const localTags = ref((props.note.tags ?? []).join(", "));

watch(
  () => props.note,
  (n) => {
    localTitle.value = n.title;
    localContent.value = n.content;
    localCategory.value = n.category;
    localTags.value = (n.tags ?? []).join(", ");
  }
);

const renderedHtml = computed(() => {
  const raw = marked.parse(localContent.value ?? "");
  const html = typeof raw === "string" ? raw : "";
  return DOMPurify.sanitize(html);
});

function emitChange() {
  emit("change", {
    title: localTitle.value,
    content: localContent.value,
    category: localCategory.value,
    tags: localTags.value.split(",").map((tag) => tag.trim()).filter(Boolean),
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

.editor-toolbar,
.editor-meta {
  flex-shrink: 0;
}

.editor-toolbar {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toolbar-head {
  display: flex;
  gap: 10px;
  align-items: center;
}

.title-input {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  font-weight: 600;
}

.tags-input {
  width: 100%;
  font-size: 12px;
  font-family: var(--font-mono), monospace;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.category-select {
  min-width: 130px;
  font-size: 12px;
  padding: 9px 12px;
  background: var(--glass-bg);
  color: var(--text-primary);
  border-radius: var(--radius-sm);
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
  border-radius: var(--radius-lg);
  overflow-y: auto;
  border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 82%, transparent);
}

.raw-editor {
  resize: none;
  padding: 14px;
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.65;
  color: var(--text-primary);
}

.preview {
  padding: 18px 20px;
  font-family: var(--font-sans);
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-secondary);
}

.preview :deep(h1) {
  margin: 0 0 16px;
  padding-bottom: 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  color: var(--text-primary);
  font-size: 28px;
  font-weight: 700;
}

.preview :deep(h2) {
  margin: 20px 0 10px;
  color: var(--text-primary);
  font-size: 21px;
  font-weight: 700;
}

.preview :deep(h3) {
  margin: 16px 0 8px;
  color: var(--text-primary);
  font-size: 16px;
  font-weight: 700;
}

.preview :deep(p) {
  margin: 0 0 12px;
}

.preview :deep(ul),
.preview :deep(ol) {
  margin: 0 0 12px;
  padding-left: 22px;
}

.preview :deep(li) {
  margin: 3px 0;
}

.preview :deep(strong) {
  color: var(--text-primary);
  font-weight: 700;
}

.preview :deep(code) {
  background: color-mix(in srgb, var(--bg-elevated) 64%, transparent);
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  font-size: 12px;
  color: var(--accent);
}

.preview :deep(pre) {
  background: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
  padding: 14px;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  overflow-x: auto;
}

.editor-meta {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.meta-text {
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.6;
}

.meta-text.accent {
  color: var(--accent);
}

.meta-text.path {
  font-family: var(--font-mono);
}

:global([data-theme="light"]) .editor-pane {
  background: rgba(255, 255, 255, 0.94);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .preview {
  color: rgba(15, 23, 42, 0.76);
}

:global([data-theme="light"]) .preview :deep(code),
:global([data-theme="light"]) .preview :deep(pre) {
  background: rgba(241, 245, 249, 0.92);
}

@media (max-width: 980px) {
  .toolbar-head {
    flex-direction: column;
    align-items: stretch;
  }

  .editor-panes {
    grid-template-columns: 1fr;
  }
}
</style>
