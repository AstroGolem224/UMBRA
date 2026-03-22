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
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  padding: 4px 8px;
  border-radius: var(--radius-pill);
  text-transform: uppercase;
  border: 1px solid transparent;
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

.working {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.08);
  border: 1px solid #f59e0b;
}
.working::before {
  background: #f59e0b;
  box-shadow: 0 0 4px #f59e0b;
  animation: spin-dot 1.2s linear infinite;
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

:global([data-theme="light"]) .online {
  color: #166534;
  background: rgba(220, 252, 231, 0.96);
  border-color: rgba(22, 101, 52, 0.16);
}

:global([data-theme="light"]) .working {
  color: #b45309;
  background: rgba(255, 247, 237, 0.96);
  border-color: rgba(180, 83, 9, 0.18);
}

:global([data-theme="light"]) .idle {
  color: #0f766e;
  background: rgba(240, 253, 250, 0.96);
  border-color: rgba(15, 118, 110, 0.16);
}

:global([data-theme="light"]) .offline {
  color: rgba(15, 23, 42, 0.72);
  background: rgba(248, 250, 252, 0.98);
  border-color: rgba(15, 23, 42, 0.12);
}

:global([data-theme="light"]) .error {
  color: #b91c1c;
  background: rgba(254, 226, 226, 0.96);
  border-color: rgba(185, 28, 28, 0.16);
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

@keyframes spin-dot {
  0% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.6); opacity: 0.5; }
  100% { transform: scale(1); opacity: 1; }
}
</style>
