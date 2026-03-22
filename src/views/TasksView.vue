<template>
  <div class="tasks-view">
    <ViewHero
      kicker="planning"
      title="Tasks"
      subtitle="pm board with drag, comments and lane sorting."
    >
      <template #meta>
        <span v-if="taskStore.lastSync" class="view-hero-pill">synced {{ relativeSync }}</span>
        <NeonButton size="sm" variant="secondary" ghost :loading="taskStore.loading" @click="sync">SYNC</NeonButton>
        <NeonButton size="sm" variant="primary" @click="showNewTask = true">+ NEW</NeonButton>
      </template>
    </ViewHero>

    <div v-if="taskStore.error" class="error-bar">{{ taskStore.error }}</div>
    <div v-if="actionError" class="error-bar">{{ actionError }}</div>

    <div v-if="pmProjects.length > 0" class="project-filter">
      <button class="proj-pill" :class="{ active: !activeProjectId }" @click="activeProjectId = null">ALL</button>
      <button
        v-for="project in pmProjects"
        :key="project.id"
        class="proj-pill"
        :class="{ active: activeProjectId === project.id }"
        @click="activeProjectId = project.id"
      >
        {{ project.name }}
      </button>
    </div>

    <div class="board-toolbar">
      <span class="drag-hint">drag between lanes to move tasks. priority sort rewrites order inside each lane.</span>
      <NeonButton size="sm" variant="secondary" ghost :loading="sorting" :disabled="taskStore.tasks.length === 0" @click="sortByPriority">
        SORT BY PRIORITY
      </NeonButton>
    </div>

    <div v-if="taskStore.loading && taskStore.tasks.length === 0" class="loading-state">Loading tasks...</div>

    <div v-else class="kanban">
      <div v-for="column in columns" :key="column.kind" class="kanban-col">
        <div class="col-header">
          <span class="col-dot" :class="column.kind" />
          <span class="col-title">{{ column.label }}</span>
          <span class="col-count">{{ column.tasks.length }}</span>
          <button
            class="lane-toggle"
            type="button"
            :aria-label="isLaneCollapsed(column) ? `expand ${column.label}` : `collapse ${column.label}`"
            @click="toggleLane(column.kind)"
          >
            {{ isLaneCollapsed(column) ? "+" : "-" }}
          </button>
        </div>

        <div
          v-if="!isLaneCollapsed(column)"
          class="col-body"
          :class="{ 'drop-target': dropTarget?.columnKind === column.kind && !dropTarget.taskId }"
          @dragover.prevent="onColumnDragOver(column.kind)"
          @dragleave="clearDropTarget"
          @drop.prevent="dropTask(column.kind)"
        >
          <div v-if="column.tasks.length === 0" class="col-empty">no tasks</div>

          <GlassCard
            v-for="task in column.tasks"
            :key="task.id"
            class="task-card"
            :class="{
              dragging: draggingTaskId === task.id,
              'drop-target': dropTarget?.columnKind === column.kind && dropTarget?.taskId === task.id,
            }"
            :variant="task.status === 'blocked' ? 'danger' : 'default'"
            :draggable="dragEnabled"
            @dragstart="startDrag($event, task)"
            @dragend="finishDrag"
            @dragover.prevent="onTaskDragOver(column.kind, task.id)"
            @dragleave="clearDropTarget"
            @drop.prevent="dropTask(column.kind, task.id)"
          >
            <div class="task-head">
              <div class="task-topline">
                <span v-if="showProjectLabel" class="task-project">{{ task.project }}</span>
                <div v-if="taskSupportMeta(task).length" class="task-support">
                  <span v-for="item in taskSupportMeta(task)" :key="item">{{ item }}</span>
                </div>
                <span class="task-priority" :class="task.priority">{{ task.priority }}</span>
              </div>
              <button
                class="collapse-btn"
                type="button"
                :aria-label="isCollapsed(task) ? 'expand task' : 'collapse task'"
                @click.stop="toggleTaskCollapse(task)"
              >
                {{ isCollapsed(task) ? "+" : "-" }}
              </button>
            </div>
            <div class="task-title">{{ task.title }}</div>
            <template v-if="!isCollapsed(task)">
              <p v-if="taskSummary(task)" class="task-summary">{{ taskSummary(task) }}</p>

              <div class="task-actions">
                <button
                  v-for="move in moveOptions(task)"
                  :key="move.label"
                  class="move-btn"
                  :disabled="moving === task.id"
                  @click="moveTask(task, move.columnKind)"
                >
                  {{ move.label }}
                </button>
                <div class="task-actions-right">
                  <button class="edit-btn" title="Edit task" @click="openEdit(task)">EDIT</button>
                  <button class="comment-btn" title="Add comment" @click="openComment(task)">COMMENT</button>
                </div>
              </div>
            </template>
          </GlassCard>
        </div>

        <button
          v-else
          class="lane-collapsed"
          type="button"
          @click="toggleLane(column.kind)"
        >
          <span>{{ column.tasks.length }} hidden</span>
          <span>expand lane</span>
        </button>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showNewTask" class="modal-backdrop" @click.self="showNewTask = false">
          <div class="modal glass-panel">
            <div class="modal-header">
              <span class="modal-title">NEW TASK</span>
              <button class="close-btn" @click="showNewTask = false">X</button>
            </div>

            <div class="form-field">
              <label class="form-label">TITLE</label>
              <input v-model="newTitle" class="glass-input" placeholder="Task title..." autofocus />
            </div>

            <div class="form-row">
              <div class="form-field">
                <label class="form-label">PROJECT</label>
                <select v-model="newProjectId" class="glass-input" @change="onProjectChange">
                  <option value="" disabled>Select project...</option>
                  <option v-for="project in pmProjects" :key="project.id" :value="project.id">{{ project.name }}</option>
                </select>
              </div>

              <div class="form-field">
                <label class="form-label">COLUMN</label>
                <select v-model="newColumnId" class="glass-input" :disabled="!newProjectId">
                  <option value="" disabled>Select column...</option>
                  <option v-for="column in pmColumns" :key="column.id" :value="column.id">{{ column.name }}</option>
                </select>
              </div>
            </div>

            <div class="form-row">
              <div class="form-field">
                <label class="form-label">PRIORITY</label>
                <select v-model="newPriority" class="glass-input">
                  <option value="urgent">Urgent</option>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                  <option value="critical">Critical</option>
                </select>
              </div>
            </div>

            <div class="form-field">
              <label class="form-label">DESCRIPTION</label>
              <textarea v-model="newDescription" class="glass-input desc-input" placeholder="Context..." />
            </div>

            <div class="modal-footer">
              <NeonButton variant="secondary" ghost size="sm" @click="showNewTask = false">CANCEL</NeonButton>
              <NeonButton
                variant="primary"
                size="sm"
                :loading="creating"
                :disabled="!newTitle.trim() || !newProjectId || !newColumnId"
                @click="createTask"
              >
                CREATE
              </NeonButton>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Teleport to="body">
      <Transition name="modal">
        <div v-if="editTaskState" class="modal-backdrop" @click.self="editTaskState = null">
          <div class="modal glass-panel">
            <div class="modal-header">
              <span class="modal-title">EDIT TASK</span>
              <button class="close-btn" @click="editTaskState = null">X</button>
            </div>

            <div class="form-field">
              <label class="form-label">TITLE</label>
              <input v-model="editTitle" class="glass-input" placeholder="Task title..." autofocus />
            </div>

            <div class="form-row">
              <div class="form-field">
                <label class="form-label">PRIORITY</label>
                <select v-model="editPriority" class="glass-input">
                  <option value="urgent">Urgent</option>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                  <option value="critical">Critical</option>
                </select>
              </div>
            </div>

            <div class="form-field">
              <label class="form-label">DESCRIPTION</label>
              <textarea v-model="editDescription" class="glass-input desc-input" placeholder="Context..." />
            </div>

            <div class="modal-footer">
              <NeonButton variant="secondary" ghost size="sm" @click="editTaskState = null">CANCEL</NeonButton>
              <NeonButton variant="primary" size="sm" :loading="editing" :disabled="!editTitle.trim()" @click="submitEdit">
                SAVE
              </NeonButton>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Teleport to="body">
      <Transition name="modal">
        <div v-if="commentTaskState" class="modal-backdrop" @click.self="commentTaskState = null">
          <div class="modal glass-panel">
            <div class="modal-header">
              <span class="modal-title">ADD COMMENT</span>
              <button class="close-btn" @click="commentTaskState = null">X</button>
            </div>

            <div class="form-field">
              <label class="form-label">{{ commentTaskState.title }}</label>
              <textarea v-model="commentText" class="glass-input desc-input" placeholder="Write comment..." autofocus />
            </div>

            <div class="modal-footer">
              <NeonButton variant="secondary" ghost size="sm" @click="commentTaskState = null">CANCEL</NeonButton>
              <NeonButton
                variant="primary"
                size="sm"
                :loading="commenting"
                :disabled="!commentText.trim()"
                @click="submitComment"
              >
                POST
              </NeonButton>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ViewHero from "@/components/layout/ViewHero.vue";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";
import { useConfigStore } from "@/stores/useConfigStore";
import { useTaskStore } from "@/stores/useTaskStore";
import type { PmColumn, PmProject, Task } from "@/interfaces";

const configStore = useConfigStore();
const taskStore = useTaskStore();
const actionError = ref<string | null>(null);
const moving = ref<string | null>(null);
const sorting = ref(false);
const activeProjectId = ref<string | null>(null);
const draggingTaskId = ref<string | null>(null);
const dropTarget = ref<{ columnKind: NonNullable<Task["columnKind"]>; taskId?: string } | null>(null);
const collapsedTaskIds = ref<Record<string, boolean>>({});
const laneOverrides = ref<Partial<Record<NonNullable<Task["columnKind"]>, boolean>>>({ ...(configStore.config.taskLanePrefs ?? {}) });
const dragEnabled = computed(() => filteredTasks.value.length > 0);
const BACKLOG_AUTO_COLLAPSE_MIN = 6;
const PRIORITY_WEIGHT: Record<Task["priority"], number> = {
  urgent: 5,
  critical: 4,
  high: 3,
  medium: 2,
  low: 1,
};

const COLUMNS: { kind: NonNullable<Task["columnKind"]>; label: string }[] = [
  { kind: "backlog", label: "BACKLOG" },
  { kind: "in_progress", label: "IN PROGRESS" },
  { kind: "review", label: "REVIEW" },
  { kind: "done", label: "DONE" },
];

function taskColumnKind(task: Task): NonNullable<Task["columnKind"]> {
  if (task.columnKind) return task.columnKind;
  if (task.status === "done") return "done";
  if (task.status === "in-progress") return "in_progress";
  return "backlog";
}

const filteredTasks = computed(() =>
  activeProjectId.value
    ? taskStore.tasks.filter((task) => task.projectId === activeProjectId.value)
    : taskStore.tasks
);
const showProjectLabel = computed(() => !activeProjectId.value);

const columns = computed(() =>
  COLUMNS.map((column) => ({
    ...column,
    tasks: [...filteredTasks.value]
      .filter((task) => taskColumnKind(task) === column.kind)
      .sort((a, b) => {
        const aPosition = a.position ?? Number.MAX_SAFE_INTEGER;
        const bPosition = b.position ?? Number.MAX_SAFE_INTEGER;
        return aPosition - bPosition || PRIORITY_WEIGHT[b.priority] - PRIORITY_WEIGHT[a.priority];
      }),
  }))
);

const MOVE_MAP: Record<NonNullable<Task["columnKind"]>, { label: string; columnKind: NonNullable<Task["columnKind"]> }[]> =
  {
    backlog: [{ label: "START", columnKind: "in_progress" }],
    in_progress: [
      { label: "REVIEW", columnKind: "review" },
      { label: "BACKLOG", columnKind: "backlog" },
    ],
    review: [
      { label: "DONE", columnKind: "done" },
      { label: "WIP", columnKind: "in_progress" },
    ],
    done: [{ label: "REOPEN", columnKind: "in_progress" }],
  };

function moveOptions(task: Task) {
  return MOVE_MAP[taskColumnKind(task)] ?? [];
}

function defaultLaneCollapsed(column: { kind: NonNullable<Task["columnKind"]>; tasks: Task[] }) {
  if (column.tasks.length === 0) return false;
  if (column.kind === "done" || column.kind === "review") return true;
  if (column.kind === "backlog") return column.tasks.length >= BACKLOG_AUTO_COLLAPSE_MIN;
  return false;
}

function isLaneCollapsed(column: { kind: NonNullable<Task["columnKind"]>; tasks: Task[] }) {
  const override = laneOverrides.value[column.kind];
  return (override ?? defaultLaneCollapsed(column)) && column.tasks.length > 0;
}

function toggleLane(kind: NonNullable<Task["columnKind"]>) {
  const column = columns.value.find((entry) => entry.kind === kind);
  if (!column) return;

  const nextPrefs = {
    ...laneOverrides.value,
    [kind]: !isLaneCollapsed(column),
  };
  laneOverrides.value = nextPrefs;
  void persistLanePrefs(nextPrefs);
}

async function persistLanePrefs(nextPrefs: Partial<Record<NonNullable<Task["columnKind"]>, boolean>>) {
  try {
    await configStore.saveConfig({ ...configStore.config, taskLanePrefs: nextPrefs });
  } catch {
    // Keep the local preference even if config persistence fails.
  }
}

function isCollapsed(task: Task) {
  return collapsedTaskIds.value[task.id] ?? taskColumnKind(task) === "done";
}

function toggleTaskCollapse(task: Task) {
  collapsedTaskIds.value = {
    ...collapsedTaskIds.value,
    [task.id]: !isCollapsed(task),
  };
}

function taskSummary(task: Task) {
  const raw = task.description?.replace(/\s+/g, " ").trim() ?? "";
  if (!raw) return "";
  return raw.length > 92 ? `${raw.slice(0, 89)}...` : raw;
}

function taskSupportMeta(task: Task) {
  const meta: string[] = [];
  const due = task.nextDueDate ?? task.deadline;
  if (due) {
    const date = new Date(due);
    if (!Number.isNaN(date.getTime())) meta.push(`due ${date.toLocaleDateString()}`);
  }
  if (task.comments?.length) meta.push(`${task.comments.length} comments`);
  return meta;
}

async function resolveProjectColumns(projectId: string) {
  return invoke<PmColumn[]>("get_pm_columns", { projectId });
}

async function moveTask(task: Task, targetKind: NonNullable<Task["columnKind"]>) {
  moving.value = task.id;
  actionError.value = null;
  try {
    const columns = await resolveProjectColumns(task.projectId);
    const target = columns.find((column) => column.kind === targetKind);
    if (!target) throw new Error(`No column with kind "${targetKind}" found`);
    await invoke("move_pm_task", { taskId: task.id, columnId: target.id });
    await taskStore.fetchTasks();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    moving.value = null;
  }
}

function startDrag(event: DragEvent, task: Task) {
  if (!dragEnabled.value) return;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", task.id);
  }
  draggingTaskId.value = task.id;
}

function finishDrag() {
  draggingTaskId.value = null;
  dropTarget.value = null;
}

function onColumnDragOver(columnKind: NonNullable<Task["columnKind"]>) {
  if (!dragEnabled.value || !draggingTaskId.value) return;
  dropTarget.value = { columnKind };
}

function onTaskDragOver(columnKind: NonNullable<Task["columnKind"]>, taskId: string) {
  if (!dragEnabled.value || !draggingTaskId.value || draggingTaskId.value === taskId) return;
  dropTarget.value = { columnKind, taskId };
}

function clearDropTarget() {
  dropTarget.value = null;
}

async function dropTask(targetKind: NonNullable<Task["columnKind"]>, overTaskId?: string) {
  if (!dragEnabled.value || !draggingTaskId.value) return;

  const task = filteredTasks.value.find((item) => item.id === draggingTaskId.value);
  if (!task) return;

  moving.value = task.id;
  actionError.value = null;

  try {
    const columns = await resolveProjectColumns(task.projectId);
    const target = columns.find((column) => column.kind === targetKind);
    if (!target) throw new Error(`No column with kind "${targetKind}" found`);

    if (task.columnId !== target.id) {
      await invoke("move_pm_task", { taskId: task.id, columnId: target.id });
    }

    const laneIds = filteredTasks.value
      .filter((item) => item.projectId === task.projectId && taskColumnKind(item) === targetKind)
      .filter((item) => item.id !== task.id)
      .map((item) => item.id);

    const overTask = overTaskId ? filteredTasks.value.find((item) => item.id === overTaskId) : null;
    const insertTargetId = overTask?.projectId === task.projectId ? overTask.id : undefined;
    const insertIndex = insertTargetId ? laneIds.indexOf(insertTargetId) : -1;
    if (insertIndex >= 0) laneIds.splice(insertIndex, 0, task.id);
    else laneIds.push(task.id);

    if (laneIds.length > 1) {
      await invoke("reorder_pm_tasks", { columnId: target.id, taskIds: laneIds });
    }

    await taskStore.fetchTasks();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    finishDrag();
    moving.value = null;
  }
}

async function sortByPriority() {
  if (filteredTasks.value.length === 0) return;
  sorting.value = true;
  actionError.value = null;

  try {
    const projectIds = [...new Set(filteredTasks.value.map((task) => task.projectId))];
    const projectColumns = new Map<string, PmColumn[]>();

    await Promise.all(
      projectIds.map(async (projectId) => {
        projectColumns.set(projectId, await resolveProjectColumns(projectId));
      })
    );

    for (const projectId of projectIds) {
      for (const lane of COLUMNS) {
        const laneTasks = filteredTasks.value
          .filter((task) => task.projectId === projectId && taskColumnKind(task) === lane.kind)
          .sort((a, b) => {
            const priorityDelta = PRIORITY_WEIGHT[b.priority] - PRIORITY_WEIGHT[a.priority];
            if (priorityDelta !== 0) return priorityDelta;
            return (a.position ?? Number.MAX_SAFE_INTEGER) - (b.position ?? Number.MAX_SAFE_INTEGER);
          });

        if (laneTasks.length < 2) continue;

        const column = projectColumns.get(projectId)?.find((item) => item.kind === lane.kind);
        if (!column) continue;

        await invoke("reorder_pm_tasks", {
          columnId: column.id,
          taskIds: laneTasks.map((task) => task.id),
        });
      }
    }

    await taskStore.fetchTasks();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    sorting.value = false;
  }
}

const showNewTask = ref(false);
const newTitle = ref("");
const newProjectId = ref("");
const newColumnId = ref("");
const newPriority = ref<Task["priority"]>("medium");
const newDescription = ref("");
const creating = ref(false);
const pmProjects = ref<PmProject[]>([]);
const pmColumns = ref<PmColumn[]>([]);

async function loadProjects() {
  try {
    pmProjects.value = await invoke<PmProject[]>("get_pm_projects");
    if (!activeProjectId.value) {
      activeProjectId.value = pmProjects.value.find((project) => project.name === "UMBRA")?.id ?? null;
    }
  } catch {
    pmProjects.value = [];
  }
}

async function onProjectChange() {
  newColumnId.value = "";
  pmColumns.value = [];
  if (!newProjectId.value) return;
  try {
    pmColumns.value = await resolveProjectColumns(newProjectId.value);
  } catch {
    pmColumns.value = [];
  }
}

async function createTask() {
  if (!newTitle.value.trim() || !newProjectId.value || !newColumnId.value) return;
  creating.value = true;
  actionError.value = null;
  try {
    await invoke("create_pm_task", {
      title: newTitle.value.trim(),
      projectId: newProjectId.value,
      columnId: newColumnId.value,
      priority: newPriority.value,
      description: newDescription.value || null,
    });
    newTitle.value = "";
    newDescription.value = "";
    newColumnId.value = "";
    showNewTask.value = false;
    await taskStore.fetchTasks();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    creating.value = false;
  }
}

const editTaskState = ref<Task | null>(null);
const editTitle = ref("");
const editPriority = ref<Task["priority"]>("medium");
const editDescription = ref("");
const editing = ref(false);

function openEdit(task: Task) {
  editTaskState.value = task;
  editTitle.value = task.title;
  editPriority.value = task.priority;
  editDescription.value = task.description ?? "";
}

async function submitEdit() {
  if (!editTaskState.value || !editTitle.value.trim()) return;
  editing.value = true;
  actionError.value = null;
  try {
    await invoke("update_pm_task", {
      taskId: editTaskState.value.id,
      title: editTitle.value.trim(),
      description: editDescription.value || null,
      priority: editPriority.value,
    });
    editTaskState.value = null;
    await taskStore.fetchTasks();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    editing.value = false;
  }
}

const commentTaskState = ref<Task | null>(null);
const commentText = ref("");
const commenting = ref(false);

function openComment(task: Task) {
  commentTaskState.value = task;
  commentText.value = "";
}

async function submitComment() {
  if (!commentTaskState.value || !commentText.value.trim()) return;
  commenting.value = true;
  actionError.value = null;
  try {
    await invoke("add_pm_comment", { taskId: commentTaskState.value.id, content: commentText.value.trim() });
    commentTaskState.value = null;
    commentText.value = "";
  } catch (e) {
    actionError.value = String(e);
  } finally {
    commenting.value = false;
  }
}

async function sync() {
  await taskStore.fetchTasks();
}

const relativeSync = computed(() => {
  if (!taskStore.lastSync) return "";
  const diff = Date.now() - new Date(taskStore.lastSync).getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return `${secs}s ago`;
  return `${Math.floor(secs / 60)}m ago`;
});

watch(
  () => configStore.config.taskLanePrefs,
  (prefs) => {
    laneOverrides.value = { ...(prefs ?? {}) };
  },
  { deep: true }
);

onMounted(async () => {
  await Promise.all([taskStore.fetchTasks(), taskStore.setupLiveUpdates(), loadProjects()]);
});
</script>

<style scoped>
.tasks-view {
  max-width: 1480px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.header-left,
.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.page-kicker {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.page-title {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
  color: var(--text-primary);
  margin: 0;
}

.page-subtitle,
.sync-time,
.drag-hint,
.loading-state,
.col-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}

.error-bar {
  margin-bottom: 16px;
  padding: 10px 14px;
  border-radius: 6px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid var(--accent-error);
  color: var(--accent-error);
  font-family: var(--font-mono);
  font-size: 12px;
}

.project-filter {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.proj-pill {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.12em;
  padding: 6px 10px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 80%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
}

.proj-pill.active,
.proj-pill:hover {
  background: var(--accent-dim);
  border-color: var(--accent);
  color: var(--accent);
}

.drag-hint {
  flex: 1;
}

.board-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 2px;
}

.kanban {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 14px;
  align-items: start;
}

.kanban-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.col-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px;
}

.col-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
}

.col-dot.backlog {
  background: var(--text-secondary);
}

.col-dot.in_progress {
  background: var(--accent);
  box-shadow: 0 0 6px var(--accent-dim);
}

.col-dot.review {
  background: var(--neon-amber);
}

.col-dot.done {
  background: var(--neon-green);
}

.col-title {
  font-family: var(--font-display);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--text-secondary);
  text-transform: uppercase;
  flex: 1;
}

.col-count {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  background: color-mix(in srgb, var(--glass-bg) 82%, transparent);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  border-radius: var(--radius-pill);
  padding: 4px 8px;
}

.lane-toggle {
  width: 22px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 2px;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 80%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
}

.col-body {
  min-height: 160px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 6px 0 0;
  transition: background 0.15s ease;
}

.lane-collapsed {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 4px;
  min-height: 44px;
  padding: 6px 4px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  cursor: pointer;
  text-align: left;
}

.col-body.drop-target {
  background: rgba(212, 82, 10, 0.05);
}

.task-card.drop-target {
  background: rgba(212, 82, 10, 0.08);
  border-color: var(--accent);
}

.task-card {
  cursor: default;
  padding: 12px;
}

.task-card[draggable="true"] {
  cursor: grab;
}

.task-card.dragging {
  opacity: 0.45;
}

.task-head,
.task-topline,
.task-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-head {
  align-items: flex-start;
  justify-content: space-between;
}

.task-topline {
  flex: 1;
  flex-wrap: wrap;
  min-width: 0;
  margin-bottom: 8px;
}

.task-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.4;
  margin-bottom: 4px;
}

.task-summary {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.55;
}

.task-project,
.task-priority,
.task-support {
  font-family: var(--font-mono);
  font-size: 10px;
}

.task-project {
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.task-priority {
  margin-left: auto;
  text-transform: uppercase;
  padding: 3px 7px;
  border-radius: var(--radius-pill);
  border: 1px solid;
}

.task-support {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
}

.task-support span + span::before {
  content: "/";
  margin-right: 6px;
}

.collapse-btn {
  width: 22px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 80%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  flex-shrink: 0;
}

.lane-toggle:hover,
.lane-collapsed:hover,
.collapse-btn:hover {
  color: var(--accent);
}

:global([data-theme="light"]) .lane-toggle,
:global([data-theme="light"]) .collapse-btn {
  background: rgba(255, 255, 255, 0.88);
  border-color: rgba(15, 23, 42, 0.08);
}

.task-priority.critical {
  color: var(--accent-error);
  border-color: rgba(239, 68, 68, 0.3);
  background: rgba(239, 68, 68, 0.08);
}

.task-priority.urgent {
  color: #ff5f7a;
  border-color: rgba(255, 95, 122, 0.35);
  background: rgba(255, 95, 122, 0.1);
}

.task-priority.high {
  color: var(--neon-amber);
  border-color: rgba(245, 158, 11, 0.3);
  background: rgba(245, 158, 11, 0.08);
}

.task-priority.medium {
  color: var(--accent);
  border-color: var(--glass-border);
  background: var(--bg-surface);
}

.task-priority.low {
  color: var(--text-muted);
  border-color: var(--glass-border);
}

:global([data-theme="light"]) .task-priority.critical {
  color: #b91c1c;
  border-color: rgba(185, 28, 28, 0.2);
  background: rgba(254, 226, 226, 0.92);
}

:global([data-theme="light"]) .task-priority.urgent {
  color: #be123c;
  border-color: rgba(190, 24, 93, 0.18);
  background: rgba(255, 228, 230, 0.94);
}

:global([data-theme="light"]) .task-priority.high {
  color: #b45309;
  border-color: rgba(180, 83, 9, 0.18);
  background: rgba(255, 247, 237, 0.94);
}

:global([data-theme="light"]) .task-priority.medium {
  color: #0f766e;
  border-color: rgba(15, 118, 110, 0.16);
  background: rgba(240, 253, 250, 0.94);
}

:global([data-theme="light"]) .task-priority.low {
  color: rgba(15, 23, 42, 0.68);
  border-color: rgba(15, 23, 42, 0.12);
  background: rgba(248, 250, 252, 0.94);
}

.task-actions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  margin-top: 6px;
  align-items: center;
  justify-content: space-between;
  padding-top: 8px;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
}

.task-actions-right {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
}

.move-btn,
.edit-btn,
.comment-btn,
.close-btn {
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  padding: 4px 8px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 78%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
}

.move-btn:hover:not(:disabled),
.edit-btn:hover,
.comment-btn:hover,
.close-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

@media (max-width: 1180px) {
  .kanban {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .header-right {
    flex-wrap: wrap;
  }

  .board-toolbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .kanban {
    grid-template-columns: 1fr;
  }
}
</style>

<style>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
  backdrop-filter: blur(2px);
}

.modal {
  width: min(520px, 92vw);
  max-height: 85vh;
  overflow-y: auto;
  padding: 24px;
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}

.modal .form-label {
  display: block;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.modal .form-field {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.modal .form-row {
  display: flex;
  gap: 12px;
}

.modal .desc-input {
  min-height: 160px;
  resize: vertical;
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.6;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--glass-border);
}
</style>
