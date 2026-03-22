import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  useConfigStore: vi.fn(),
  useGithubStore: vi.fn(),
  writeText: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

vi.mock("@/stores/useGithubStore", () => ({
  useGithubStore: mocks.useGithubStore,
}));

import LauncherView from "../LauncherView.vue";

describe("LauncherView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    Object.defineProperty(globalThis.navigator, "clipboard", {
      configurable: true,
      value: { writeText: mocks.writeText },
    });

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
          repoRootPath: "C:/Users/matth/OneDrive/Dokumente/GitHub",
          launchTargets: [{ id: "ide-1", name: "VS Code", path: "C:/Apps/Code.exe", icon: "VC" }],
          githubTargets: [{ id: "repo-1", name: "UMBRA", owner: "matth", repo: "UMBRA" }],
        },
        setTheme: vi.fn(),
      })
    );

    mocks.useGithubStore.mockReturnValue(
      reactive({
        repos: [
          {
            id: "repo-1",
            name: "UMBRA",
            owner: "matth",
            repo: "UMBRA",
            fullName: "matth/UMBRA",
            openIssues: 2,
            pushedAt: "2026-03-22T09:00:00.000Z",
            htmlUrl: "https://github.com/matth/UMBRA",
          },
        ],
        loading: false,
        error: null,
        loadRepos: vi.fn().mockResolvedValue(undefined),
        repoById: vi.fn((id: string) =>
          id === "repo-1"
            ? {
                id: "repo-1",
                name: "UMBRA",
                owner: "matth",
                repo: "UMBRA",
                fullName: "matth/UMBRA",
                openIssues: 2,
                pushedAt: "2026-03-22T09:00:00.000Z",
                htmlUrl: "https://github.com/matth/UMBRA",
              }
            : undefined
        ),
      })
    );

    mocks.invoke.mockImplementation(async (command: string, args?: Record<string, string>) => {
      if (command === "list_user_repos") {
        return [
          {
            name: "UMBRA",
            fullName: "matth/UMBRA",
            htmlUrl: "https://github.com/matth/UMBRA",
            private: true,
            description: "",
            pushedAt: "2026-03-22T09:00:00.000Z",
          },
        ];
      }
      return args ?? null;
    });
  });

  it("shows repo quick actions for launch, issues, local folder/terminal and clone helpers", async () => {
    const wrapper = mount(LauncherView);
    await flushPromises();

    expect(wrapper.text()).toContain("copy path");
    expect(wrapper.text()).toContain("copy link");
    expect(wrapper.text()).toContain("issues");
    expect(wrapper.text()).toContain("prs");
    expect(wrapper.text()).toContain("folder");
    expect(wrapper.text()).toContain("terminal");
    expect(wrapper.text()).toContain("copy ssh");

    const ideCopy = wrapper.findAll(".quick-btn").find((button) => button.text().includes("copy path"));
    expect(ideCopy).toBeTruthy();

    await ideCopy!.trigger("click");
    expect(mocks.writeText).toHaveBeenCalledWith("C:/Apps/Code.exe");

    const select = wrapper.find(".repo-select");
    await select.setValue("https://github.com/matth/UMBRA");

    const repoCopy = wrapper.findAll("button").find((button) => button.text().includes("copy link"));
    expect(repoCopy).toBeTruthy();

    await repoCopy!.trigger("click");
    expect(mocks.writeText).toHaveBeenCalledWith("https://github.com/matth/UMBRA");

    const issuesButton = wrapper.findAll(".quick-btn").find((button) => button.text().includes("issues"));
    const prsButton = wrapper.findAll(".quick-btn").find((button) => button.text().includes("prs"));
    const folderButton = wrapper.findAll(".quick-btn").find((button) => button.text().includes("folder"));
    const terminalButton = wrapper.findAll(".quick-btn").find((button) => button.text().includes("terminal"));
    const sshButton = wrapper.findAll(".quick-btn").find((button) => button.text().includes("copy ssh"));

    expect(issuesButton).toBeTruthy();
    expect(prsButton).toBeTruthy();
    expect(folderButton).toBeTruthy();
    expect(terminalButton).toBeTruthy();
    expect(sshButton).toBeTruthy();

    await issuesButton!.trigger("click");
    expect(mocks.invoke).toHaveBeenCalledWith("open_github_url", {
      url: "https://github.com/matth/UMBRA/issues",
    });

    await prsButton!.trigger("click");
    expect(mocks.invoke).toHaveBeenCalledWith("open_github_url", {
      url: "https://github.com/matth/UMBRA/pulls",
    });

    await folderButton!.trigger("click");
    expect(mocks.invoke).toHaveBeenCalledWith("open_local_repo_folder", {
      repoName: "UMBRA",
    });

    await terminalButton!.trigger("click");
    expect(mocks.invoke).toHaveBeenCalledWith("open_local_repo_terminal", {
      repoName: "UMBRA",
    });

    await sshButton!.trigger("click");
    expect(mocks.writeText).toHaveBeenCalledWith("git@github.com:matth/UMBRA.git");
  });
});
