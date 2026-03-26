<template>
  <Teleport to="body">
    <Transition name="palette-fade">
      <div v-if="palette.open" class="palette-backdrop" @click.self="palette.closePalette()">
        <div class="palette-shell glass-panel" role="dialog" aria-modal="true" aria-label="command palette">
          <div class="palette-head">
            <input
              ref="inputRef"
              v-model="palette.query"
              class="palette-input glass-input"
              placeholder="search commands, notes, tasks, skills, repos..."
              @keydown.down.prevent="palette.moveSelection(1)"
              @keydown.up.prevent="palette.moveSelection(-1)"
              @keydown.enter.prevent="runSelected"
              @keydown.esc.prevent="palette.closePalette()"
              @input="palette.resetSelection()"
            />
            <span class="palette-hint">ctrl+k / alt+1..7</span>
          </div>

          <div v-if="palette.loading" class="palette-empty">indexing local entities...</div>
          <div v-else-if="palette.filteredEntries.length === 0" class="palette-empty">no matching commands or entities</div>

          <div v-else class="palette-list">
            <button
              v-for="(entry, index) in palette.filteredEntries"
              :key="entry.id"
              class="palette-item"
              :class="{ active: palette.selectedIndex === index }"
              type="button"
              @mouseenter="palette.selectedIndex = index"
              @click="runEntry(entry)"
            >
              <span class="item-kind">{{ entry.kind }}</span>
              <div class="item-copy">
                <strong>{{ entry.title }}</strong>
                <small>{{ entry.subtitle }}</small>
              </div>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";
import type { CommandPaletteEntry } from "@/stores/useCommandPaletteStore";
import { useAgentStore } from "@/stores/useAgentStore";
import { useCommandPaletteStore } from "@/stores/useCommandPaletteStore";
import { useNotesStore } from "@/stores/useNotesStore";
import { useSkillsStore } from "@/stores/useSkillsStore";
import { useTaskStore } from "@/stores/useTaskStore";

const router = useRouter();
const palette = useCommandPaletteStore();
const notesStore = useNotesStore();
const taskStore = useTaskStore();
const agentStore = useAgentStore();
const skillsStore = useSkillsStore();
const inputRef = ref<HTMLInputElement | null>(null);

watch(
  () => palette.open,
  async (isOpen) => {
    if (!isOpen) return;
    await nextTick();
    inputRef.value?.focus();
    inputRef.value?.select();
  }
);

function handleGlobalKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    if (palette.open) palette.closePalette();
    else palette.openPalette();
    return;
  }

  const target = event.target as HTMLElement | null;
  const typingTarget =
    target?.tagName === "INPUT" ||
    target?.tagName === "TEXTAREA" ||
    target?.tagName === "SELECT" ||
    target?.isContentEditable;

  if (!palette.open && !typingTarget && event.ctrlKey && event.shiftKey && event.key.toLowerCase() === "n") {
    event.preventDefault();
    void runCommand("new-note");
    return;
  }

  if (!palette.open && !typingTarget && event.altKey) {
    const routeCommands: Record<string, string> = {
      "1": "go-dashboard",
      "2": "go-workbench",
      "3": "go-ops-room",
      "4": "go-notes",
      "5": "go-tasks",
      "6": "go-launcher",
      "7": "go-settings",
    };
    const commandId = routeCommands[event.key];
    if (commandId) {
      event.preventDefault();
      void runCommand(commandId);
      return;
    }
  }

  if (!palette.open) return;
  if (event.key === "Escape") {
    event.preventDefault();
    palette.closePalette();
  }
}

window.addEventListener("keydown", handleGlobalKeydown);

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleGlobalKeydown);
});

async function runSelected() {
  const selected = palette.filteredEntries[palette.selectedIndex];
  if (!selected) return;
  await runEntry(selected);
}

async function runEntry(entry: CommandPaletteEntry) {
  switch (entry.kind) {
    case "command":
      await runCommand(entry.commandId);
      break;
    case "note":
      notesStore.activeNoteId = entry.noteId;
      await router.push("/notes");
      break;
    case "task":
      await router.push("/tasks");
      break;
    case "agent":
      if (agentStore.agents.length === 0) {
        await agentStore.loadAgents();
      }
      await router.push("/agents");
      break;
    case "skill":
      skillsStore.selectSkill(entry.skillId);
      await router.push("/skills");
      break;
    case "launcher":
      await invoke("launch_target", { targetId: entry.targetId });
      break;
    case "repo":
      await invoke("open_github_url", { url: entry.repoUrl });
      break;
  }

  palette.closePalette();
}

async function runCommand(commandId: string) {
  switch (commandId) {
    case "new-note": {
      const note = notesStore.newNote();
      notesStore.activeNoteId = note.id;
      await router.push("/notes");
      break;
    }
    case "sync-pm":
      await taskStore.fetchTasks();
      await router.push("/tasks");
      break;
    case "go-dashboard":
      await router.push("/dashboard");
      break;
    case "go-notes":
      await router.push("/notes");
      break;
    case "go-workbench":
      await router.push("/workbench");
      break;
    case "go-ops-room":
      await router.push("/ops-room");
      break;
    case "go-tasks":
      await router.push("/tasks");
      break;
    case "go-launcher":
      await router.push("/launcher");
      break;
    case "go-skills":
      await router.push("/skills");
      break;
    case "go-settings":
      await router.push("/settings");
      break;
  }
}
</script>

<style scoped>
.palette-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9500;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 10vh 16px 24px;
  background: rgba(2, 6, 23, 0.58);
  backdrop-filter: blur(8px);
}

.palette-shell {
  width: min(760px, 100%);
  max-height: 78vh;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 18px;
  border-radius: var(--radius-2xl);
  border: 1px solid color-mix(in srgb, var(--accent) 18%, var(--glass-border));
}

.palette-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.palette-input {
  flex: 1;
  font-size: 14px;
  padding: 12px 14px;
}

.palette-hint,
.item-kind,
.palette-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.palette-hint,
.palette-empty {
  color: var(--text-muted);
}

.palette-empty {
  padding: 10px 4px;
}

.palette-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  min-height: 0;
}

.palette-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  width: 100%;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--glass-bg) 86%, transparent);
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.16s ease, background 0.16s ease, transform 0.16s ease;
}

.palette-item:hover,
.palette-item.active {
  border-color: color-mix(in srgb, var(--accent) 28%, transparent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  transform: translateY(-1px);
}

.item-kind {
  min-width: 72px;
  color: var(--accent);
}

.item-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.item-copy strong {
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.45;
}

.item-copy small {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.palette-fade-enter-active,
.palette-fade-leave-active {
  transition: opacity 0.16s ease;
}

.palette-fade-enter-from,
.palette-fade-leave-to {
  opacity: 0;
}

@media (max-width: 720px) {
  .palette-backdrop {
    padding-top: 6vh;
  }

  .palette-head,
  .palette-item {
    flex-direction: column;
    align-items: stretch;
  }

  .item-kind {
    min-width: 0;
  }
}
</style>
