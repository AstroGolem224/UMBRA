import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  useRouter: vi.fn(),
}));

vi.mock("vue-router", async () => {
  const actual = await vi.importActual<typeof import("vue-router")>("vue-router");
  return {
    ...actual,
    useRouter: mocks.useRouter,
  };
});

import AppLayout from "../AppLayout.vue";

describe("AppLayout", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.useRouter.mockReturnValue({ go: vi.fn() });
  });

  it("renders the global fallback when a global error event arrives", async () => {
    const wrapper = mount(AppLayout, {
      global: {
        stubs: {
          CustomTitlebar: true,
          AppSidebar: true,
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
});
