import type { AppConfig, WorkspacePreset } from "@/interfaces";

export function createEmptyWorkspacePreset(): WorkspacePreset {
  return {
    id: crypto.randomUUID(),
    name: "",
    rootPath: "",
    writable: true,
    allowedProviders: [],
    allowedAgents: [],
  };
}

export function normalizeWorkspacePresets(
  presets: WorkspacePreset[] | undefined,
): WorkspacePreset[] {
  return (presets ?? [])
    .map((preset) => ({
      id: preset.id?.trim() || crypto.randomUUID(),
      name: preset.name.trim(),
      rootPath: preset.rootPath.trim(),
      writable: preset.writable !== false,
      allowedProviders: [...new Set((preset.allowedProviders ?? []).map((entry) => entry.trim()).filter(Boolean))],
      allowedAgents: [...new Set((preset.allowedAgents ?? []).map((entry) => entry.trim()).filter(Boolean))],
    }))
    .filter((preset) => preset.name && preset.rootPath);
}

export function mergeWorkspaceGrantRoots(
  roots: string[] | undefined,
  presets: WorkspacePreset[] | undefined,
): string[] {
  const next = new Set<string>();
  for (const root of roots ?? []) {
    if (root.trim()) next.add(root.trim());
  }
  for (const preset of presets ?? []) {
    if (preset.rootPath?.trim()) next.add(preset.rootPath.trim());
  }
  return [...next].sort((left, right) => left.localeCompare(right));
}

export function buildWorkspaceConfigUpdate(
  config: AppConfig,
  workspacePresets: WorkspacePreset[],
  defaultWorkspaceId: string | null,
): AppConfig {
  const normalized = normalizeWorkspacePresets(workspacePresets);
  const resolvedDefault =
    defaultWorkspaceId && normalized.some((workspace) => workspace.id === defaultWorkspaceId)
      ? defaultWorkspaceId
      : normalized[0]?.id ?? null;

  return {
    ...config,
    workspacePresets: normalized,
    defaultWorkspaceId: resolvedDefault,
    workspaceGrantRoots: mergeWorkspaceGrantRoots(config.workspaceGrantRoots, normalized),
  };
}
