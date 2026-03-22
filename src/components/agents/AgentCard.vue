<template>
  <GlassCard :variant="agent.status === 'online' || agent.status === 'working' ? 'accent' : 'default'" clickable @click="$emit('select', agent)">
    <div class="agent-header">
      <div class="agent-name-row">
        <span class="agent-icon">{{ agentIcon }}</span>
        <div class="agent-copy">
          <span class="agent-name">{{ agent.name }}</span>
          <span class="agent-role">{{ agent.role || "generalist" }}</span>
        </div>
      </div>
      <StatusBadge :status="agent.status" />
    </div>

    <div class="agent-skills">
      <span v-for="skill in agent.skills.slice(0, 4)" :key="skill" class="skill-tag">
        {{ skill }}
      </span>
      <span v-if="agent.skills.length > 4" class="skill-more">+{{ agent.skills.length - 4 }}</span>
    </div>

    <div class="agent-footer">
      <span class="last-seen">
        {{
          agent.status === "offline"
            ? `last seen ${formatRelative(agent.lastSeen)}`
            : agent.status === "working"
              ? `working${agent.activeTaskId ? ` / ${agent.activeTaskId}` : ""}`
              : "active now"
        }}
      </span>
      <span class="tool-count">{{ agent.allowedTools.length }} tools</span>
    </div>
  </GlassCard>
</template>

<script setup lang="ts">
import type { Agent } from "@/interfaces";
import GlassCard from "@/components/ui/GlassCard.vue";
import StatusBadge from "@/components/ui/StatusBadge.vue";

const props = defineProps<{ agent: Agent }>();
defineEmits<{ select: [agent: Agent] }>();

const agentIcons: Record<string, string> = {
  prism: "PR",
  forge: "FG",
  jim: "JM",
};

const agentIcon = agentIcons[props.agent.id] ?? "AG";

function formatRelative(isoDate: string): string {
  const diff = Date.now() - new Date(isoDate).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}
</script>

<style scoped>
.agent-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}

.agent-name-row {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.agent-icon {
  width: 34px;
  height: 34px;
  border-radius: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 22%, transparent);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  flex-shrink: 0;
}

.agent-copy {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.agent-name {
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 700;
  line-height: 1.1;
}

.agent-role,
.last-seen,
.tool-count,
.skill-more {
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.5;
}

.agent-skills {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}

.skill-tag {
  padding: 4px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--glass-bg) 84%, transparent);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 10px;
}

.agent-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  padding-top: 10px;
}
</style>
