import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  useConfigStore: vi.fn(),
  shellOpen: vi.fn(),
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: mocks.shellOpen,
}));

import SettingsView from "../SettingsView.vue";

describe("SettingsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
          vaultPath: "D:/vault",
          notesSubdir: "UMBRA_Notes",
          repoRootPath: "C:/Users/matth/OneDrive/Dokumente/GitHub",
          launchTargets: [],
          githubTargets: [],
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
        taskLanePrefs: expect.objectContaining({
          backlog: true,
        }),
      })
    );
  });
});
