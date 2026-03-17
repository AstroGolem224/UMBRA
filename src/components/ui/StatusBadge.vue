<template>
  <span class="status-badge" :class="status">{{ label ?? status }}</span>
</template>

<script setup lang="ts">
import type { AgentStatus } from "@/interfaces";

defineProps<{
  status: AgentStatus;
  label?: string;
}>();
</script>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-family: "Iceland", monospace;
  font-size: 10px;
  letter-spacing: 0.12em;
  padding: 2px 8px;
  border-radius: 4px;
  text-transform: uppercase;
}

.status-badge::before {
  content: "";
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex-shrink: 0;
}

.online {
  color: var(--accent-success);
  background: rgba(0, 245, 100, 0.08);
  border: 1px solid var(--accent-success);
}
.online::before {
  background: var(--accent-success);
  box-shadow: 0 0 4px var(--accent-success);
  animation: pulse-dot 2s ease-in-out infinite;
}

.idle {
  color: var(--accent);
  background: var(--accent-dim);
  border: 1px solid var(--accent);
}
.idle::before {
  background: var(--accent);
}

.offline {
  color: var(--text-muted);
  background: var(--bg-surface);
  border: 1px solid var(--glass-border);
}
.offline::before {
  background: var(--text-muted);
}

.error {
  color: var(--accent-error);
  background: rgba(255, 50, 50, 0.08);
  border: 1px solid var(--accent-error);
}
.error::before {
  background: var(--accent-error);
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
