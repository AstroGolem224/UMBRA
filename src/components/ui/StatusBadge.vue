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
  padding: 4px 9px;
  border-radius: var(--radius-pill);
  text-transform: uppercase;
  border: none;
}

.status-badge::before {
  content: "";
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex-shrink: 0;
}

.online {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.12);
}
.online::before {
  background: #22c55e;
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
  animation: pulse-dot 2s ease-in-out infinite;
}

.working {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.12);
}
.working::before {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
  animation: spin-dot 1.2s linear infinite;
}

.idle {
  color: var(--accent);
  background: var(--accent-dim);
}
.idle::before {
  background: var(--accent);
}

.offline {
  color: var(--text-muted);
  background: color-mix(in srgb, var(--bg-surface) 80%, transparent);
}
.offline::before {
  background: var(--text-muted);
  opacity: 0.5;
}

.error {
  color: var(--accent-error);
  background: rgba(239, 68, 68, 0.10);
}
.error::before {
  background: var(--accent-error);
}

:global([data-theme="light"]) .online {
  color: #166534;
  background: rgba(220, 252, 231, 0.8);
}

:global([data-theme="light"]) .working {
  color: #b45309;
  background: rgba(255, 247, 237, 0.8);
}

:global([data-theme="light"]) .idle {
  color: #0e7490;
  background: rgba(207, 250, 254, 0.6);
}

:global([data-theme="light"]) .offline {
  color: #6e7881;
  background: rgba(241, 245, 249, 0.8);
}

:global([data-theme="light"]) .error {
  color: #b91c1c;
  background: rgba(254, 226, 226, 0.8);
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
