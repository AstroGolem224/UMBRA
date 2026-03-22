import { mount, flushPromises } from "@vue/test-utils";
import { reactive } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentCronJob } from "@/interfaces";

const mocks = vi.hoisted(() => ({
  useCronStore: vi.fn(),
  useConfigStore: vi.fn(),
}));

vi.mock("@/stores/useCronStore", () => ({
  useCronStore: mocks.useCronStore,
}));

vi.mock("@/stores/useConfigStore", () => ({
  useConfigStore: mocks.useConfigStore,
}));

import CronView from "../CronView.vue";

describe("CronView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.useConfigStore.mockReturnValue(
      reactive({
        config: {
          theme: "ember",
        },
        setTheme: vi.fn(),
      })
    );

    const jobs: AgentCronJob[] = [
      {
        id: "daily-build",
        agentId: "forge",
        agentName: "Forge",
        job: "Daily Build",
        timing: "09:00",
        recurrence: "weekdays",
        timezone: "Europe/Berlin",
        enabled: true,
        lastRun: "2026-03-20T08:00:00.000Z",
        nextRun: "2026-03-21T08:00:00.000Z",
        lastStatus: "ok",
        notes: "publishes digest",
        source: "systemd timer",
        command: null,
        updatedAt: "2026-03-20T08:01:00.000Z",
      },
      {
        id: "nightly-sync",
        agentId: "prism",
        agentName: "Prism",
        job: "Nightly Sync",
        timing: "02:00",
        recurrence: "daily",
        timezone: "UTC",
        enabled: false,
        lastRun: null,
        nextRun: null,
        lastStatus: "error",
        notes: null,
        source: "launchd",
        command: null,
        updatedAt: "2026-03-20T07:00:00.000Z",
      },
    ];

    mocks.useCronStore.mockReturnValue(
      reactive({
        jobs,
        loading: false,
        error: null,
        lastSync: "2026-03-20T10:00:00.000Z",
        loadJobs: vi.fn().mockResolvedValue(undefined),
        setupLiveUpdates: vi.fn().mockResolvedValue(undefined),
      })
    );
  });

  it("renders agent cron telemetry, summary cards and the agent API contract", async () => {
    const wrapper = mount(CronView);

    await flushPromises();

    expect(wrapper.text()).toContain("agent schedule telemetry");
    expect(wrapper.text()).toContain("reporting agents");
    expect(wrapper.text()).toContain("POST /api/agents/:id/cron-jobs");
    expect(wrapper.text()).toContain("Forge");
    expect(wrapper.text()).toContain("Daily Build");
    expect(wrapper.text()).toContain("Nightly Sync");
    expect(wrapper.text()).toContain("publishes digest");
    expect(wrapper.findAll(".agent-group")).toHaveLength(2);
    expect(wrapper.findAll(".job-status.error")).toHaveLength(1);
  });
});
