<template>
  <button
    class="neon-btn"
    :class="[variant, size, { loading, ghost }]"
    :disabled="disabled || loading"
    @click="$emit('click')"
  >
    <span v-if="loading" class="btn-spinner" />
    <slot />
  </button>
</template>

<script setup lang="ts">
defineProps<{
  variant?: "primary" | "secondary" | "danger";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  loading?: boolean;
  ghost?: boolean;
}>();
defineEmits(["click"]);
</script>

<style scoped>
.neon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 34px;
  font-family: var(--font-mono);
  font-weight: 600;
  letter-spacing: 0.06em;
  border-radius: var(--radius-sm);
  border: 1px solid currentColor;
  cursor: pointer;
  transition: all 0.15s;
  background: transparent;
  white-space: nowrap;
}

.neon-btn:disabled {
  opacity: 0.56;
  cursor: not-allowed;
  box-shadow: none;
}

/* Sizes */
.sm { min-height: 32px; font-size: 11px; padding: 6px 12px; }
.md, :not(.sm):not(.lg) { min-height: 38px; font-size: 12px; padding: 8px 14px; }
.lg { min-height: 44px; font-size: 13px; padding: 10px 18px; }

/* Variants */
.primary, :not(.secondary):not(.danger) {
  color: var(--accent);
  border-color: var(--accent);
}
.primary:not(:disabled):hover, :not(.secondary):not(.danger):not(:disabled):hover {
  background: var(--accent-dim);
  box-shadow: var(--glow-neon);
}

.secondary {
  color: var(--text-secondary);
  border-color: var(--glass-border);
}
.secondary:not(:disabled):hover {
  color: var(--text-primary);
  border-color: var(--text-secondary);
  background: var(--bg-surface-hover);
}

.danger {
  color: var(--accent-error);
  border-color: var(--accent-error);
}
.danger:not(:disabled):hover {
  background: rgba(255, 50, 50, 0.1);
  box-shadow: 0 0 8px rgba(255, 50, 50, 0.3);
}

.ghost {
  border-color: transparent;
}

:global([data-theme="light"]) .neon-btn.secondary,
:global([data-theme="light"]) .neon-btn.ghost {
  background: rgba(255, 255, 255, 0.92);
  border-color: rgba(15, 23, 42, 0.16);
  color: rgba(15, 23, 42, 0.8);
}

:global([data-theme="light"]) .neon-btn.primary {
  background: rgba(255, 255, 255, 0.9);
}

:global([data-theme="light"]) .neon-btn:disabled {
  background: rgba(248, 250, 252, 0.96);
  border-color: rgba(15, 23, 42, 0.14);
  color: rgba(15, 23, 42, 0.5);
}

:global([data-theme="light"]) .neon-btn.primary:not(:disabled):hover {
  box-shadow: 0 8px 20px rgba(11, 114, 133, 0.12);
}

.btn-spinner {
  width: 10px;
  height: 10px;
  border: 1px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
