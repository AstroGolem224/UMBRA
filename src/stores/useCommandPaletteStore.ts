import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import { useGithubStore } from "@/stores/useGithubStore";
import { useNotesStore } from "@/stores/useNotesStore";
import { useSkillsStore } from "@/stores/useSkillsStore";
import { useTaskStore } from "@/stores/useTaskStore";

type CommandEntry = {
  id: string;
  kind: "command";
  title: string;
  subtitle: string;
  keywords: string[];
  commandId: string;
};

type EntityEntry =
  | {
      id: string;
      kind: "note";
      title: string;
      subtitle: string;
      keywords: string[];
      noteId: string;
    }
  | {
      id: string;
      kind: "task";
      title: string;
      subtitle: string;
      keywords: string[];
      taskId: string;
    }
  | {
      id: string;
      kind: "agent";
      title: string;
      subtitle: string;
      keywords: string[];
      agentId: string;
    }
  | {
      id: string;
      kind: "skill";
      title: string;
      subtitle: string;
      keywords: string[];
      skillId: string;
    }
  | {
      id: string;
      kind: "launcher";
      title: string;
      subtitle: string;
      keywords: string[];
      targetId: string;
    }
  | {
      id: string;
      kind: "repo";
      title: string;
      subtitle: string;
      keywords: string[];
      repoUrl: string;
    };

export type CommandPaletteEntry = CommandEntry | EntityEntry;

const MAX_RESULTS = 14;

function matchesQuery(query: string, fields: string[]) {
  if (!query) return true;
  return fields.some((value) => value.toLowerCase().includes(query));
}

function scoreEntry(query: string, entry: CommandPaletteEntry) {
  if (!query) return entry.kind === "command" ? 100 : 10;

  const title = entry.title.toLowerCase();
  if (title.startsWith(query)) return 220;
  if (title.includes(query)) return 160;
  if (entry.subtitle.toLowerCase().includes(query)) return 100;
  return 40;
}

export const useCommandPaletteStore = defineStore("command-palette", () => {
  const open = ref(false);
  const query = ref("");
  const loading = ref(false);
  const bootstrapped = ref(false);
  const selectedIndex = ref(0);

  const notesStore = useNotesStore();
  const taskStore = useTaskStore();
  const agentStore = useAgentStore();
  const githubStore = useGithubStore();
  const configStore = useConfigStore();
  const skillsStore = useSkillsStore();

  const entries = computed<CommandPaletteEntry[]>(() => {
    const commands: CommandEntry[] = [
      {
        id: "cmd:new-note",
        kind: "command",
        title: "new note",
        subtitle: "create a note and open the vault editor",
        keywords: ["new note", "create note", "notes"],
        commandId: "new-note",
      },
      {
        id: "cmd:sync-pm",
        kind: "command",
        title: "sync pm",
        subtitle: "refresh tasks from the PM API",
        keywords: ["sync pm", "refresh tasks", "pm tool", "tasks"],
        commandId: "sync-pm",
      },
      {
        id: "cmd:dashboard",
        kind: "command",
        title: "open dashboard",
        subtitle: "jump to mission control",
        keywords: ["dashboard", "home", "overview"],
        commandId: "go-dashboard",
      },
      {
        id: "cmd:workbench",
        kind: "command",
        title: "open workbench",
        subtitle: "jump to agent dispatch and run history",
        keywords: ["workbench", "dispatch", "agents", "chat", "tasks"],
        commandId: "go-workbench",
      },
      {
        id: "cmd:ops-room",
        kind: "command",
        title: "open ops room",
        subtitle: "jump to channels, routing and sessions",
        keywords: ["ops room", "channels", "routing", "jobs", "sessions"],
        commandId: "go-ops-room",
      },
      {
        id: "cmd:notes",
        kind: "command",
        title: "open notes",
        subtitle: "jump to the vault editor",
        keywords: ["notes", "vault", "markdown"],
        commandId: "go-notes",
      },
      {
        id: "cmd:tasks",
        kind: "command",
        title: "open tasks",
        subtitle: "jump to the PM board",
        keywords: ["tasks", "board", "kanban", "pm"],
        commandId: "go-tasks",
      },
      {
        id: "cmd:launcher",
        kind: "command",
        title: "open launcher",
        subtitle: "jump to repo and IDE launch targets",
        keywords: ["launcher", "repos", "ides", "workspace"],
        commandId: "go-launcher",
      },
      {
        id: "cmd:skills",
        kind: "command",
        title: "open skills",
        subtitle: "jump to indexed codex skills",
        keywords: ["skills", "capabilities", "skill index"],
        commandId: "go-skills",
      },
      {
        id: "cmd:settings",
        kind: "command",
        title: "open settings",
        subtitle: "jump to app configuration",
        keywords: ["settings", "config", "preferences"],
        commandId: "go-settings",
      },
    ];

    const notes = notesStore.notes.map<CommandPaletteEntry>((note) => ({
      id: `note:${note.id}`,
      kind: "note",
      title: note.title || "untitled note",
      subtitle: `${note.category} / ${note.tags.join(", ") || "no tags"}`,
      keywords: [note.title, note.content, note.category, ...note.tags],
      noteId: note.id,
    }));

    const tasks = taskStore.tasks.map<CommandPaletteEntry>((task) => ({
      id: `task:${task.id}`,
      kind: "task",
      title: task.title,
      subtitle: `${task.project} / ${task.priority} / ${task.status}`,
      keywords: [task.title, task.project, task.priority, task.status, task.description ?? ""],
      taskId: task.id,
    }));

    const agents = agentStore.agents.map<CommandPaletteEntry>((agent) => ({
      id: `agent:${agent.id}`,
      kind: "agent",
      title: agent.name,
      subtitle: `${agent.role || "no role"} / ${agent.status}`,
      keywords: [agent.name, agent.id, agent.role, agent.status, ...agent.skills, ...agent.allowedTools],
      agentId: agent.id,
    }));

    const skills = skillsStore.skills.map<CommandPaletteEntry>((skill) => ({
      id: `skill:${skill.id}`,
      kind: "skill",
      title: skill.name,
      subtitle: `${skill.category} / ${skill.folder}`,
      keywords: [skill.name, skill.category, skill.description, skill.folder, ...skill.agents],
      skillId: skill.id,
    }));

    const launchers = (configStore.config.launchTargets ?? []).map<CommandPaletteEntry>((target) => ({
      id: `launcher:${target.id}`,
      kind: "launcher",
      title: target.name,
      subtitle: target.path,
      keywords: [target.name, target.path, target.icon ?? ""],
      targetId: target.id,
    }));

    const pinnedRepos = (configStore.config.githubTargets ?? []).map<CommandPaletteEntry>((target) => ({
      id: `repo:${target.id}`,
      kind: "repo",
      title: target.name,
      subtitle: `${target.owner}/${target.repo}`,
      keywords: [target.name, target.owner, target.repo, `${target.owner}/${target.repo}`],
      repoUrl: `https://github.com/${target.owner}/${target.repo}`,
    }));

    const liveRepos = githubStore.repos.map<CommandPaletteEntry>((repo) => ({
      id: `repo-live:${repo.id}`,
      kind: "repo",
      title: repo.name,
      subtitle: `${repo.owner}/${repo.repo} / ${repo.openIssues} open`,
      keywords: [repo.name, repo.owner, repo.repo, repo.fullName],
      repoUrl: repo.htmlUrl,
    }));

    return [...commands, ...notes, ...tasks, ...agents, ...skills, ...launchers, ...pinnedRepos, ...liveRepos];
  });

  const filteredEntries = computed(() => {
    const trimmed = query.value.trim().toLowerCase();
    return entries.value
      .filter((entry) => matchesQuery(trimmed, [entry.title, entry.subtitle, ...entry.keywords]))
      .sort((a, b) => scoreEntry(trimmed, b) - scoreEntry(trimmed, a))
      .slice(0, MAX_RESULTS);
  });

  async function bootstrapSources() {
    if (loading.value) return;
    loading.value = true;
    try {
      await Promise.allSettled([
        notesStore.notes.length === 0 ? notesStore.loadNotes() : Promise.resolve(),
        taskStore.tasks.length === 0 ? taskStore.fetchTasks() : Promise.resolve(),
        agentStore.agents.length === 0 ? agentStore.loadAgents() : Promise.resolve(),
        skillsStore.skills.length === 0 ? skillsStore.loadSkills() : Promise.resolve(),
        githubStore.repos.length === 0 && configStore.config.githubTargets?.length
          ? githubStore.loadRepos()
          : Promise.resolve(),
      ]);
      bootstrapped.value = true;
    } finally {
      loading.value = false;
    }
  }

  function openPalette(initialQuery = "") {
    open.value = true;
    query.value = initialQuery;
    selectedIndex.value = 0;
    void bootstrapSources();
  }

  function closePalette() {
    open.value = false;
    query.value = "";
    selectedIndex.value = 0;
  }

  function moveSelection(delta: number) {
    const total = filteredEntries.value.length;
    if (total === 0) {
      selectedIndex.value = 0;
      return;
    }
    selectedIndex.value = (selectedIndex.value + delta + total) % total;
  }

  function resetSelection() {
    selectedIndex.value = 0;
  }

  return {
    open,
    query,
    loading,
    bootstrapped,
    selectedIndex,
    filteredEntries,
    openPalette,
    closePalette,
    moveSelection,
    resetSelection,
    bootstrapSources,
  };
});
