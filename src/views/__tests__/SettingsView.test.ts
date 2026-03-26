import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  useConfigStore: vi.fn(),
  useAgentStore: vi.fn(),
  invoke: vi.fn(),
  dialogOpen: vi.fn(),
  shellOpen: vi.fn(),
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

vi.mock("@/stores/useAgentStore", () => ({
  useAgentStore: mocks.useAgentStore,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: mocks.dialogOpen,
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: mocks.shellOpen,
}));

import SettingsView from "../SettingsView.vue";

describe("SettingsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mocks.useAgentStore.mockReturnValue(
      reactive({
        agents: [
          {
            id: "codex-main",
            name: "Codex Main",
            role: "code agent",
            status: "idle",
            lastSeen: "2026-03-25T12:00:00Z",
            allowedTools: [],
            skills: [],
          },
        ],
        loadAgents: vi.fn().mockResolvedValue(undefined),
      })
    );

    mocks.invoke.mockImplementation((command: string) => {
      switch (command) {
        case "check_provider_setup":
          return Promise.resolve({
            providerId: "codex",
            workspaceId: "umbra",
            summary: "4/5 checklist item(s) ready",
            items: [],
          });
        default:
          return Promise.resolve(null);
      }
    });
    mocks.dialogOpen.mockResolvedValue(null);

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
          closeToTray: true,
          vaultPath: "D:/vault",
          notesSubdir: "UMBRA_Notes",
          repoRootPath: "C:/Users/matth/OneDrive/Dokumente/GitHub",
          workspacePresets: [
            {
              id: "umbra",
              name: "UMBRA",
              rootPath: "C:/Users/matth/OneDrive/Dokumente/GitHub/UMBRA",
              writable: true,
              allowedProviders: ["codex"],
              allowedAgents: ["codex-main"],
            },
          ],
          workspaceGrantRoots: ["C:/Users/matth/OneDrive/Dokumente/GitHub/UMBRA"],
          defaultWorkspaceId: "umbra",
          personaPresets: [
            {
              id: "implementer",
              name: "implementer",
              description: "ship code",
              prompt: "implement the requested change",
            },
          ],
          launchTargets: [],
          githubTargets: [],
          providerCommands: [
            { providerId: "codex", command: "codex" },
            { providerId: "claude", command: "claude" },
            { providerId: "gemini", command: "gemini" },
            { providerId: "kimi", command: "kimi" },
          ],
          pmToolUrl: "http://100.115.61.30:8000",
          pmToolDashboardUrl: "http://100.115.61.30:4173",
          pmToolPollSeconds: 30,
          updaterEndpoint: "",
          updaterPublicKey: "",
          autoCheckForUpdates: false,
          uapAdvertiseHost: "127.0.0.1",
          uapPort: 8765,
          uapToken: "umbra-uap-2026",
          githubPat: "",
          taskLanePrefs: {},
          agentNotes: {},
          agentAuthTokens: { forge: "forge-token", prism: "prism-token" },
          customAgents: [],
        },
        setTheme: vi.fn(),
        saveConfig: vi.fn().mockResolvedValue(undefined),
      })
    );
  });

  it("allows explicit lane prefs to be saved from settings", async () => {
    const wrapper = mount(SettingsView);
    await flushPromises();

    expect(wrapper.text()).toContain("lane defaults");

    const collapsedButtons = wrapper.findAll(".lane-mode").filter((button) => button.text() === "collapsed");
    expect(collapsedButtons.length).toBeGreaterThan(0);

    await collapsedButtons[0].trigger("click");
    await flushPromises();

    await wrapper.find("form").trigger("submit.prevent");
    await flushPromises();

    const store = mocks.useConfigStore.mock.results[0].value;
    expect(store.saveConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        closeToTray: true,
        taskLanePrefs: expect.objectContaining({
          backlog: true,
        }),
      })
    );
  });

  it("persists the close-to-tray preference", async () => {
    const wrapper = mount(SettingsView);
    await flushPromises();

    const checkbox = wrapper.findAll('input[type="checkbox"]')[0];
    expect((checkbox.element as HTMLInputElement).checked).toBe(true);

    await checkbox.setValue(false);
    await wrapper.find("form").trigger("submit.prevent");
    await flushPromises();

    const store = mocks.useConfigStore.mock.results[0].value;
    expect(store.saveConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        closeToTray: false,
      })
    );
  });

  it("persists provider command overrides for workbench providers", async () => {
    const wrapper = mount(SettingsView);
    await flushPromises();

    const providerInputs = wrapper.findAll(".provider-command");
    expect(providerInputs.length).toBeGreaterThan(0);

    await providerInputs[0].setValue("C:/tools/codex.exe");
    await wrapper.find("form").trigger("submit.prevent");
    await flushPromises();

    const store = mocks.useConfigStore.mock.results[0].value;
    expect(store.saveConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        providerCommands: expect.arrayContaining([
          expect.objectContaining({
            providerId: "codex",
            command: "C:/tools/codex.exe",
          }),
        ]),
      })
    );
  });

  it("renders provider checklist action in settings", async () => {
    const wrapper = mount(SettingsView);
    await flushPromises();

    expect(wrapper.text()).toContain("checklist");
    expect(mocks.invoke).toHaveBeenCalledWith("check_provider_setup", {
      providerId: "codex",
      workspaceId: "umbra",
    });
  });
});
