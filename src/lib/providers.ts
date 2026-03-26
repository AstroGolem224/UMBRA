export function deriveProviderIdFromAgentId(agentId?: string | null) {
  const normalized = agentId?.trim().toLowerCase() ?? "";
  if (!normalized) return "custom";
  if (normalized.startsWith("codex")) return "codex";
  if (normalized.startsWith("claude")) return "claude";
  if (normalized.startsWith("gemini")) return "gemini";
  if (normalized.startsWith("kimi")) return "kimi";
  return "custom";
}
