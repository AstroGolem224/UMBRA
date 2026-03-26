<template>
  <div
    class="note-editor"
    :class="{ 'drag-active': isDraggingFiles }"
    @paste.capture="handlePaste"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <div class="editor-toolbar">
      <div class="toolbar-head">
        <input
          v-model="localTitle"
          class="title-input glass-input"
          placeholder="note title..."
          @input="emitChange"
        />
        <div class="toolbar-actions">
          <select
            v-model="localCategory"
            class="category-select glass-input"
            data-test="category-select"
            @change="emitChange"
          >
            <option v-for="cat in categoryOptions" :key="cat" :value="cat">{{ cat }}</option>
          </select>
          <NeonButton
            v-if="!creatingCategory"
            variant="secondary"
            size="sm"
            ghost
            data-test="add-category"
            @click="startCategoryCreate"
          >+ category</NeonButton>
          <NeonButton
            variant="secondary"
            size="sm"
            ghost
            :loading="attaching"
            data-test="attach-files"
            @click="openAttachmentPicker"
          >attach</NeonButton>
          <NeonButton variant="primary" size="sm" :loading="saving" @click="$emit('save')">save</NeonButton>
          <NeonButton v-if="note.id" variant="danger" size="sm" ghost @click="$emit('delete')">delete</NeonButton>
        </div>
      </div>

      <div v-if="creatingCategory" class="category-builder">
        <input
          v-model="categoryDraft"
          class="category-input glass-input"
          data-test="category-input"
          placeholder="new category..."
          @keydown.enter.prevent="applyCategoryCreate"
          @keydown.esc.prevent="cancelCategoryCreate"
        />
        <NeonButton
          variant="secondary"
          size="sm"
          :disabled="!normalizedCategoryDraft"
          data-test="apply-category"
          @click="applyCategoryCreate"
        >apply</NeonButton>
        <NeonButton
          variant="secondary"
          size="sm"
          ghost
          data-test="cancel-category"
          @click="cancelCategoryCreate"
        >cancel</NeonButton>
      </div>

      <input
        v-model="localTags"
        class="tags-input glass-input"
        placeholder="tags, comma separated..."
        @input="emitChange"
      />

      <div class="insert-row">
        <label class="insert-label" for="quick-link-select">insert</label>
        <select
          id="quick-link-select"
          v-model="selectedQuickLink"
          class="quick-link-select glass-input"
          data-test="quick-link-select"
          @change="insertSelectedQuickLink"
        >
          <option value="">quick link...</option>
          <optgroup v-for="group in quickLinkGroups" :key="group.id" :label="group.label">
            <option
              v-for="option in group.options"
              :key="quickLinkKey(option)"
              :value="quickLinkKey(option)"
            >
              {{ option.label }}{{ option.description ? ` · ${option.description}` : "" }}
            </option>
          </optgroup>
        </select>
        <span v-if="attachmentState" class="toolbar-status">{{ attachmentState }}</span>
        <span v-else class="toolbar-status muted">paste or drop files anywhere in the editor</span>
        <input
          ref="fileInputRef"
          class="attachment-input"
          data-test="attachment-input"
          type="file"
          multiple
          @change="handleFileInputChange"
        />
      </div>

      <div class="view-modes" role="tablist" aria-label="note view mode">
        <button
          v-for="mode in viewModes"
          :key="mode.id"
          class="view-mode-btn"
          :class="{ active: viewMode === mode.id }"
          type="button"
          role="tab"
          :aria-selected="viewMode === mode.id"
          @click="viewMode = mode.id"
        >
          {{ mode.label }}
        </button>
      </div>
    </div>

    <div class="editor-panes" :class="`mode-${viewMode}`">
      <textarea
        v-if="showMarkdown"
        ref="rawEditorRef"
        v-model="localContent"
        class="editor-pane raw-editor glass-input"
        placeholder="write markdown here..."
        spellcheck="false"
        @input="emitChange"
      />
      <div
        v-if="showPreview"
        ref="previewRef"
        class="editor-pane preview"
        v-html="renderedHtml"
        @click="handlePreviewClick"
      />
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
import { invoke } from "@tauri-apps/api/core";
import { computed, nextTick, ref, watch } from "vue";
import { useRouter } from "vue-router";
import DOMPurify from "dompurify";
import { marked } from "marked";
import type {
  Note,
  NoteCategory,
  NoteQuickLinkGroup,
  NoteQuickLinkOption,
  SavedNoteAttachment,
} from "@/interfaces";
import NeonButton from "@/components/ui/NeonButton.vue";
import { useAgentStore } from "@/stores/useAgentStore";
import { useTaskStore } from "@/stores/useTaskStore";

type ViewMode = "markdown" | "split" | "preview";
type MermaidApi = (typeof import("mermaid"))["default"];

type UmbraChipTarget = {
  kind: string;
  targetId: string;
  label: string;
  meta: string;
  url: string;
};

const DEFAULT_NOTE_CATEGORY = "misc";

const props = withDefaults(
  defineProps<{
    note: Note;
    categories?: NoteCategory[];
    quickLinkGroups?: NoteQuickLinkGroup[];
    saving?: boolean;
    autosaving?: boolean;
    autosaveState?: string | null;
  }>(),
  {
    categories: () => [],
    quickLinkGroups: () => [],
  }
);

const emit = defineEmits<{
  change: [patch: Partial<Note>];
  save: [];
  delete: [];
}>();

const router = useRouter();
const taskStore = useTaskStore();
const agentStore = useAgentStore();

const viewModes: Array<{ id: ViewMode; label: string }> = [
  { id: "markdown", label: "markdown" },
  { id: "split", label: "split" },
  { id: "preview", label: "preview" },
];

const localTitle = ref(props.note.title);
const localContent = ref(props.note.content);
const localCategory = ref<NoteCategory>(props.note.category);
const localTags = ref((props.note.tags ?? []).join(", "));
const categoryDraft = ref("");
const creatingCategory = ref(false);
const viewMode = ref<ViewMode>("split");
const selectedQuickLink = ref("");
const attachmentState = ref<string | null>(null);
const attaching = ref(false);
const isDraggingFiles = ref(false);
const previewRef = ref<HTMLElement | null>(null);
const rawEditorRef = ref<HTMLTextAreaElement | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);
const dragDepth = ref(0);
let mermaidApiPromise: Promise<MermaidApi> | null = null;

const showMarkdown = computed(() => viewMode.value !== "preview");
const showPreview = computed(() => viewMode.value !== "markdown");
const normalizedCategoryDraft = computed(() => categoryDraft.value.trim());
const categoryOptions = computed(() =>
  Array.from(
    new Set(
      [...props.categories, localCategory.value]
        .map((category) => category.trim())
        .filter(Boolean)
    )
  ).sort((a, b) => a.localeCompare(b))
);
const quickLinkLookup = computed(() => {
  const lookup = new Map<string, NoteQuickLinkOption>();
  for (const group of props.quickLinkGroups) {
    for (const option of group.options) {
      lookup.set(quickLinkKey(option), option);
    }
  }
  return lookup;
});

watch(
  () => props.note,
  (n) => {
    localTitle.value = n.title;
    localContent.value = n.content;
    localCategory.value = n.category;
    localTags.value = (n.tags ?? []).join(", ");
    attachmentState.value = null;
    selectedQuickLink.value = "";
  }
);

function quickLinkKey(option: NoteQuickLinkOption) {
  return `${option.kind}:${option.id}`;
}

function startCategoryCreate() {
  creatingCategory.value = true;
  categoryDraft.value = "";
}

function cancelCategoryCreate() {
  creatingCategory.value = false;
  categoryDraft.value = "";
}

function applyCategoryCreate() {
  if (!normalizedCategoryDraft.value) return;
  localCategory.value = normalizedCategoryDraft.value;
  cancelCategoryCreate();
  emitChange();
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function parseUmbraHref(href: string): UmbraChipTarget | null {
  try {
    const parsed = new URL(href);
    if (parsed.protocol !== "umbra:") return null;
    return {
      kind: parsed.hostname,
      targetId: decodeURIComponent(parsed.pathname.replace(/^\/+/, "")),
      label: parsed.searchParams.get("label")?.trim() || "",
      meta: parsed.searchParams.get("meta")?.trim() || "",
      url: parsed.searchParams.get("url")?.trim() || "",
    };
  } catch {
    return null;
  }
}

function renderUmbraChipHtml(href: string, fallbackLabel: string) {
  const target = parseUmbraHref(href);
  if (!target || !target.kind || !target.targetId) {
    return "";
  }

  const label = target.label || fallbackLabel || target.targetId;
  const meta = target.meta ? `<span class="umbra-chip-meta">${escapeHtml(target.meta)}</span>` : "";
  const urlAttr = target.url ? ` data-url="${escapeHtml(target.url)}"` : "";

  return [
    `<button type="button" class="umbra-chip" data-umbra-chip="true"`,
    ` data-kind="${escapeHtml(target.kind)}"`,
    ` data-target-id="${escapeHtml(target.targetId)}"`,
    urlAttr,
    target.meta ? ` title="${escapeHtml(target.meta)}"` : "",
    ">",
    `<span class="umbra-chip-label">${escapeHtml(label)}</span>`,
    meta,
    "</button>",
  ].join("");
}

function createRenderer() {
  const renderer = new marked.Renderer();
  const defaultCode = renderer.code.bind(renderer);
  const defaultLink = renderer.link.bind(renderer);

  renderer.code = (token) => {
    const lang = token.lang?.trim().toLowerCase();
    if (lang === "mermaid") {
      return `<div class="mermaid">${escapeHtml(token.text)}</div>`;
    }
    return defaultCode(token);
  };

  renderer.link = (token) => {
    if (token.href?.startsWith("umbra://")) {
      return renderUmbraChipHtml(token.href, token.text);
    }
    return defaultLink(token);
  };

  return renderer;
}

const renderedHtml = computed(() => {
  const raw = marked.parse(localContent.value ?? "", {
    async: false,
    gfm: true,
    renderer: createRenderer(),
  });
  const html = typeof raw === "string" ? raw : "";
  return DOMPurify.sanitize(html);
});

async function loadMermaid() {
  if (!mermaidApiPromise) {
    mermaidApiPromise = import("mermaid").then(({ default: mermaid }) => {
      mermaid.initialize({
        startOnLoad: false,
        securityLevel: "strict",
        suppressErrorRendering: false,
        theme: "default",
        flowchart: { useMaxWidth: true, htmlLabels: true },
      });
      return mermaid;
    });
  }

  return mermaidApiPromise;
}

async function renderMermaidDiagrams() {
  if (!showPreview.value || !previewRef.value) return;

  const nodes = Array.from(previewRef.value.querySelectorAll<HTMLElement>(".mermaid"));
  if (nodes.length === 0) return;

  const mermaid = await loadMermaid();
  await mermaid.run({ nodes });
}

watch(
  [renderedHtml, showPreview],
  async () => {
    if (!showPreview.value) return;
    await nextTick();
    await renderMermaidDiagrams();
  },
  { immediate: true }
);

function insertSnippet(snippet: string) {
  const trimmed = snippet.trim();
  if (!trimmed) return;

  const separator = localContent.value.trim().length > 0 ? "\n\n" : "";
  localContent.value = `${localContent.value}${separator}${trimmed}`;
  emitChange();
  void nextTick(() => {
    rawEditorRef.value?.focus();
    rawEditorRef.value?.setSelectionRange(localContent.value.length, localContent.value.length);
  });
}

function buildQuickLinkMarkdown(option: NoteQuickLinkOption) {
  const url = new URL(`umbra://${option.kind}/${encodeURIComponent(option.id)}`);
  url.searchParams.set("label", option.label);
  if (option.description) {
    url.searchParams.set("meta", option.description);
  }
  if (option.url) {
    url.searchParams.set("url", option.url);
  }
  return `[${option.label}](${url.toString()})`;
}

function insertSelectedQuickLink() {
  const option = quickLinkLookup.value.get(selectedQuickLink.value);
  if (!option) return;
  insertSnippet(buildQuickLinkMarkdown(option));
  selectedQuickLink.value = "";
}

function openAttachmentPicker() {
  fileInputRef.value?.click();
}

function currentCategory() {
  return localCategory.value.trim() || props.note.category || DEFAULT_NOTE_CATEGORY;
}

async function saveFiles(files: File[]) {
  if (files.length === 0 || !props.note.id) return;

  attaching.value = true;
  attachmentState.value = "saving attachments...";

  try {
    const markdownSnippets: string[] = [];

    for (const file of files) {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      const saved = await invoke<SavedNoteAttachment>("save_note_attachment", {
        noteId: props.note.id,
        category: currentCategory(),
        fileName: file.name,
        bytes,
        mimeType: file.type || null,
      });
      markdownSnippets.push(saved.markdown);
    }

    insertSnippet(markdownSnippets.join("\n"));
    attachmentState.value = `${files.length} attachment${files.length === 1 ? "" : "s"} added`;
  } catch (error) {
    attachmentState.value = String(error);
  } finally {
    attaching.value = false;
    if (fileInputRef.value) {
      fileInputRef.value.value = "";
    }
  }
}

async function handleFileInputChange(event: Event) {
  const target = event.target as HTMLInputElement | null;
  const files = Array.from(target?.files ?? []);
  await saveFiles(files);
}

function collectClipboardFiles(event: ClipboardEvent) {
  const directFiles = Array.from(event.clipboardData?.files ?? []);
  if (directFiles.length > 0) {
    return directFiles;
  }

  return Array.from(event.clipboardData?.items ?? [])
    .map((item) => item.getAsFile())
    .filter((file): file is File => Boolean(file));
}

function handlePaste(event: ClipboardEvent) {
  const files = collectClipboardFiles(event).filter((file) => file.size > 0);
  if (files.length === 0) return;
  event.preventDefault();
  void saveFiles(files);
}

function dragHasFiles(dataTransfer: DataTransfer | null) {
  return Array.from(dataTransfer?.types ?? []).includes("Files");
}

function handleDragEnter(event: DragEvent) {
  if (!dragHasFiles(event.dataTransfer)) return;
  dragDepth.value += 1;
  isDraggingFiles.value = true;
}

function handleDragOver(event: DragEvent) {
  if (!dragHasFiles(event.dataTransfer)) return;
  isDraggingFiles.value = true;
}

function handleDragLeave(event: DragEvent) {
  if (!dragHasFiles(event.dataTransfer)) return;
  dragDepth.value = Math.max(0, dragDepth.value - 1);
  if (dragDepth.value === 0) {
    isDraggingFiles.value = false;
  }
}

function handleDrop(event: DragEvent) {
  dragDepth.value = 0;
  isDraggingFiles.value = false;
  const files = Array.from(event.dataTransfer?.files ?? []).filter((file) => file.size > 0);
  if (files.length === 0) return;
  void saveFiles(files);
}

async function openUmbraChip(target: UmbraChipTarget) {
  switch (target.kind) {
    case "task":
      if (taskStore.tasks.length === 0) {
        await taskStore.fetchTasks();
      }
      await router.push("/tasks");
      return;
    case "agent":
      if (agentStore.agents.length === 0) {
        await agentStore.loadAgents();
      }
      await router.push("/agents");
      return;
    case "repo":
      if (target.url) {
        await invoke("open_github_url", { url: target.url });
      }
      return;
    case "launcher":
      await invoke("launch_target", { targetId: target.targetId });
      return;
  }
}

function handlePreviewClick(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  const chip = target?.closest<HTMLElement>("[data-umbra-chip='true']");
  if (!chip) return;

  event.preventDefault();
  void openUmbraChip({
    kind: chip.dataset.kind ?? "",
    targetId: chip.dataset.targetId ?? "",
    label: chip.dataset.label ?? "",
    meta: chip.getAttribute("title") ?? "",
    url: chip.dataset.url ?? "",
  });
}

function emitChange() {
  const nextCategory = localCategory.value.trim() || props.note.category || DEFAULT_NOTE_CATEGORY;
  localCategory.value = nextCategory;
  emit("change", {
    title: localTitle.value,
    content: localContent.value,
    category: nextCategory,
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
  transition: box-shadow 0.16s ease, border-color 0.16s ease;
}

.note-editor.drag-active .editor-panes {
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 24%, transparent);
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

.toolbar-actions,
.category-builder,
.insert-row {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.insert-label {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.toolbar-status {
  color: var(--accent);
  font-size: 11px;
  line-height: 1.5;
}

.toolbar-status.muted {
  color: var(--text-muted);
}

.attachment-input {
  display: none;
}

.view-modes {
  display: inline-flex;
  align-self: flex-start;
  gap: 6px;
  padding: 4px;
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--glass-bg) 84%, transparent);
}

.view-mode-btn {
  min-height: 28px;
  padding: 5px 12px;
  border: 0;
  border-radius: var(--radius-pill);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  transition: background 0.15s ease, color 0.15s ease, box-shadow 0.15s ease;
}

.view-mode-btn:hover {
  color: var(--text-primary);
}

.view-mode-btn.active {
  background: color-mix(in srgb, var(--accent-dim) 78%, transparent);
  color: var(--accent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 32%, transparent);
}

.category-select,
.quick-link-select {
  min-width: 130px;
  font-size: 12px;
  padding: 9px 12px;
  background: var(--glass-bg);
  color: var(--text-primary);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.category-select option,
.quick-link-select option {
  background: var(--bg-surface);
}

.category-input {
  min-width: min(220px, 100%);
  font-size: 12px;
}

.editor-panes {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.editor-panes.mode-markdown,
.editor-panes.mode-preview {
  grid-template-columns: minmax(0, 1fr);
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

.preview :deep(.mermaid) {
  margin: 0 0 16px;
  padding: 12px;
  overflow-x: auto;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  background: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-primary);
}

.preview :deep(.mermaid[data-processed="true"]) {
  padding: 0;
  border: 0;
  background: transparent;
}

.preview :deep(.mermaid svg) {
  display: block;
  max-width: 100%;
  height: auto;
  margin: 0 auto;
}

.preview :deep(.umbra-chip) {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin: 0 6px 10px 0;
  padding: 8px 12px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--accent) 28%, transparent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--text-primary);
  cursor: pointer;
  transition: transform 0.16s ease, background 0.16s ease, border-color 0.16s ease;
}

.preview :deep(.umbra-chip:hover) {
  transform: translateY(-1px);
  background: color-mix(in srgb, var(--accent) 14%, transparent);
}

.preview :deep(.umbra-chip-label) {
  font-weight: 700;
}

.preview :deep(.umbra-chip-meta) {
  color: var(--text-muted);
  font-size: 12px;
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

:global([data-theme="light"]) .view-modes {
  background: rgba(255, 255, 255, 0.94);
  border-color: rgba(15, 23, 42, 0.12);
}

:global([data-theme="light"]) .view-mode-btn.active {
  background: rgba(11, 114, 133, 0.12);
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
