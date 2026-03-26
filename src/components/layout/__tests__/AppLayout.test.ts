import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  useRouter: vi.fn(),
  listen: vi.fn(),
  fetchTasks: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("vue-router", async () => {
  const actual = await vi.importActual<typeof import("vue-router")>("vue-router");
  return {
    ...actual,
    useRouter: mocks.useRouter,
  };
});

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

vi.mock("@/stores/useTaskStore", () => ({
  useTaskStore: () => ({
    fetchTasks: mocks.fetchTasks,
  }),
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: () => ({
    config: { vaultPath: "/test", pmToolUrl: "http://localhost", repoRootPath: "/repos" },
    loaded: true,
  }),
}));

import AppLayout from "../AppLayout.vue";

describe("AppLayout", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.useRouter.mockReturnValue({ go: vi.fn() });
    mocks.listen.mockResolvedValue(() => {});
  });

  it("renders the global fallback when a global error event arrives", async () => {
    const wrapper = mount(AppLayout, {
      global: {
        stubs: {
          CustomTitlebar: true,
          AppSidebar: true,
          CommandPalette: true,
          OnboardingWizard: true,
          ToastContainer: true,
          RouterView: { template: '<div class="route-view">ok</div>' },
        },
      },
    });

    window.dispatchEvent(new CustomEvent("umbra:error", { detail: { message: "broken flow" } }));
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain("broken flow");

    await wrapper.find(".error-retry").trigger("click");
    expect(mocks.useRouter.mock.results[0].value.go).toHaveBeenCalledWith(0);
  });

  it("listens for tray sync events and refreshes tasks", async () => {
    const trayListenerRef: { current: ((payload?: unknown) => Promise<void> | void) | null } = { current: null };
    mocks.listen.mockImplementation(async (_event: string, handler: (payload?: unknown) => Promise<void> | void) => {
      trayListenerRef.current = handler;
      return () => {};
    });

    mount(AppLayout, {
      global: {
        stubs: {
          CustomTitlebar: true,
          AppSidebar: true,
          CommandPalette: true,
          OnboardingWizard: true,
          ToastContainer: true,
          RouterView: { template: '<div class="route-view">ok</div>' },
        },
      },
    });

    const triggerTraySync = trayListenerRef.current;
    if (typeof triggerTraySync === "function") {
      await triggerTraySync();
    }

    expect(mocks.fetchTasks).toHaveBeenCalledTimes(1);
  });
});
