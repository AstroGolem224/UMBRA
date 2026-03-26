import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

import { useOpsStore } from "../useOpsStore";

describe("useOpsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    mocks.listen.mockResolvedValue(() => {});
  });

  it("loads channels and hydrates the active channel payload", async () => {
    mocks.invoke.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
      if (command === "list_ops_channels") {
        return [
          {
            id: "channel-1",
            name: "delivery",
            description: "",
            workspaceId: "core",
            defaultAgentId: "forge",
            createdAt: "2026-03-26T00:00:00Z",
            updatedAt: "2026-03-26T00:00:00Z",
          },
        ];
      }
      if (command === "list_ops_rules") return [];
      if (command === "list_ops_session_templates") return [];
      if (command === "list_ops_channel_messages_page" && args?.channelId === "channel-1") {
        return { items: [], nextBefore: null, hasMore: false };
      }
      if (command === "list_ops_jobs" && args?.channelId === "channel-1") return [];
      if (command === "list_ops_route_approvals" && args?.channelId === "channel-1") return [];
      if (command === "list_ops_sessions" && args?.channelId === "channel-1") return [];
      return null;
    });

    const store = useOpsStore();
    await store.loadChannels();

    expect(store.channels).toHaveLength(1);
    expect(store.activeChannelId).toBe("channel-1");
  });
});
