import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { useAgentStore } from "../useAgentStore";
import { useCommandPaletteStore } from "../useCommandPaletteStore";
import { useConfigStore } from "../useConfigStore";
import { useGithubStore } from "../useGithubStore";
import { useNotesStore } from "../useNotesStore";
import { useSkillsStore } from "../useSkillsStore";
import { useTaskStore } from "../useTaskStore";

describe("useCommandPaletteStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("searches commands plus local entities from multiple stores", () => {
    const configStore = useConfigStore();
    const notesStore = useNotesStore();
    const taskStore = useTaskStore();
    const agentStore = useAgentStore();
    const skillsStore = useSkillsStore();
    const githubStore = useGithubStore();
    const palette = useCommandPaletteStore();

    configStore.config.launchTargets = [{ id: "vscode", name: "VS Code", path: "C:/Repos/UMBRA", icon: "VS" }];
    configStore.config.githubTargets = [{ id: "umbra", name: "UMBRA", owner: "cmg", repo: "umbra" }];

    notesStore.notes = [
      {
        id: "note-1",
        title: "deploy checklist",
        content: "portable and installer",
        category: "ops",
        tags: ["release"],
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        filePath: "D:/vault/UMBRA_Notes/ops/note-1.md",
      },
    ];

    taskStore.tasks = [
      {
        id: "task-1",
        title: "ship updater",
        status: "in-progress",
        priority: "high",
        project: "UMBRA",
        projectId: "umbra",
        columnId: "col-1",
      },
    ];

    agentStore.agents = [
      {
        id: "forge",
        name: "Forge",
        role: "builder",
        status: "working",
        lastSeen: new Date().toISOString(),
        allowedTools: ["shell"],
        skills: ["review"],
      },
    ];

    skillsStore.skills = [
      {
        id: "qa",
        name: "QA",
        version: "1.0.0",
        description: "systematic qa",
        category: "quality",
        agents: ["forge"],
        content: "qa content",
        folder: "qa",
      },
    ];

    githubStore.repos = [
      {
        id: "repo-1",
        name: "umbra",
        owner: "cmg",
        repo: "umbra",
        fullName: "cmg/umbra",
        openIssues: 2,
        htmlUrl: "https://github.com/cmg/umbra",
      },
    ];

    palette.openPalette();

    expect(palette.filteredEntries.some((entry) => entry.kind === "command")).toBe(true);

    palette.query = "umbra";
    expect(palette.filteredEntries.some((entry) => entry.kind === "repo")).toBe(true);

    palette.query = "deploy";
    expect(palette.filteredEntries[0]).toMatchObject({ kind: "note", title: "deploy checklist" });

    palette.query = "forge";
    expect(palette.filteredEntries.some((entry) => entry.kind === "agent")).toBe(true);

    palette.query = "vs code";
    expect(palette.filteredEntries[0]).toMatchObject({ kind: "launcher", title: "VS Code" });
  });
});
