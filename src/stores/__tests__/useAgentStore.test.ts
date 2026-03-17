import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useAgentStore } from "../useAgentStore";
import type { Agent } from "@/interfaces";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

const makeAgent = (overrides: Partial<Agent> = {}): Agent => ({
  id: "forge",
  name: "Forge",
  role: "Web / Code Agent",
  status: "online",
  allowedTools: ["vscode", "git"],
  skills: ["TypeScript", "Vue 3"],
  lastSeen: new Date().toISOString(),
  ...overrides,
});

describe("useAgentStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("loads agents via invoke", async () => {
    const agents = [makeAgent(), makeAgent({ id: "prism", name: "Prism", status: "offline" })];
    mockInvoke.mockResolvedValueOnce(agents);

    const store = useAgentStore();
    await store.loadAgents();

    expect(store.agents).toHaveLength(2);
    expect(store.loading).toBe(false);
    expect(store.error).toBeNull();
  });

  it("sets error on invoke failure", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Connection refused"));

    const store = useAgentStore();
    await store.loadAgents();

    expect(store.agents).toHaveLength(0);
    expect(store.error).toContain("Connection refused");
    expect(store.loading).toBe(false);
  });
});
