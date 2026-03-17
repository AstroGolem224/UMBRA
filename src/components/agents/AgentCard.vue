<template>
  <GlassCard :variant="agent.status === 'online' ? 'accent' : 'default'" clickable @click="$emit('select', agent)">
    <div class="agent-header">
      <div class="agent-name-row">
        <span class="agent-icon">{{ agentIcon }}</span>
        <span class="agent-name">{{ agent.name }}</span>
      </div>
      <StatusBadge :status="agent.status" />
    </div>

    <p class="agent-role">{{ agent.role }}</p>

    <div class="agent-skills">
      <span v-for="skill in agent.skills.slice(0, 4)" :key="skill" class="skill-tag">
        {{ skill }}
      </span>
      <span v-if="agent.skills.length > 4" class="skill-more">+{{ agent.skills.length - 4 }}</span>
    </div>

    <div class="agent-footer">
      <span class="last-seen">
        {{ agent.status === "offline" ? `Last seen ${formatRelative(agent.lastSeen)}` : "Active now" }}
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
  prism: "◈",
  forge: "◉",
  jim: "◎",
};

const agentIcon = agentIcons[props.agent.id] ?? "◆";

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
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.agent-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.agent-icon {
  font-size: 18px;
  color: var(--accent);
}

.agent-name {
  font-family: "Iceland", monospace;
  font-size: 16px;
  letter-spacing: 0.12em;
  color: var(--text-primary);
  text-transform: uppercase;
}

.agent-role {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0 0 12px;
}

.agent-skills {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 12px;
}

.skill-tag {
  font-size: 10px;
  font-family: "Iceland", monospace;
  letter-spacing: 0.06em;
  padding: 2px 7px;
  border-radius: 3px;
  background: var(--bg-surface);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
}

.skill-more {
  font-size: 10px;
  color: var(--text-muted);
  padding: 2px 4px;
}

.agent-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--glass-border);
  padding-top: 8px;
}

.last-seen,
.tool-count {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 0.06em;
}
</style>
