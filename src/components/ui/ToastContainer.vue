<template>
  <Teleport to="body">
    <TransitionGroup name="toast-slide" tag="div" class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="toast.level"
        @click="dismiss(toast.id)"
      >
        <span class="toast-icon">{{ iconFor(toast.level) }}</span>
        <div class="toast-body">
          <strong v-if="toast.title" class="toast-title">{{ toast.title }}</strong>
          <span class="toast-message">{{ toast.message }}</span>
        </div>
        <button class="toast-close" type="button" @click.stop="dismiss(toast.id)">x</button>
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<script setup lang="ts">
import { useToastStore } from "@/stores/useToastStore";

const { toasts, dismiss } = useToastStore();

function iconFor(level: string) {
  switch (level) {
    case "success": return "ok";
    case "error": return "err";
    case "warn": return "warn";
    default: return "info";
  }
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 9800;
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
  max-width: 420px;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 16px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 96%, transparent);
  backdrop-filter: blur(12px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  cursor: pointer;
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.toast-item:hover {
  transform: translateX(-2px);
}

.toast-item.success { border-left: 3px solid #22c55e; }
.toast-item.error { border-left: 3px solid #ef4444; }
.toast-item.warn { border-left: 3px solid #f59e0b; }
.toast-item.info { border-left: 3px solid var(--accent); }

.toast-icon {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 3px 6px;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  margin-top: 1px;
}

.success .toast-icon { color: #22c55e; background: rgba(34, 197, 94, 0.1); }
.error .toast-icon { color: #ef4444; background: rgba(239, 68, 68, 0.1); }
.warn .toast-icon { color: #f59e0b; background: rgba(245, 158, 11, 0.1); }
.info .toast-icon { color: var(--accent); background: var(--accent-dim); }

.toast-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toast-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.toast-message {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.toast-close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  padding: 2px 4px;
  flex-shrink: 0;
}

.toast-close:hover { color: var(--text-primary); }

.toast-slide-enter-active { transition: all 0.3s ease; }
.toast-slide-leave-active { transition: all 0.2s ease; }
.toast-slide-enter-from { transform: translateX(100%); opacity: 0; }
.toast-slide-leave-to { transform: translateX(100%); opacity: 0; }
</style>
