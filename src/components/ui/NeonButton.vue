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
  gap: 6px;
  font-family: "Iceland", monospace;
  letter-spacing: 0.08em;
  border-radius: 6px;
  border: 1px solid currentColor;
  cursor: pointer;
  transition: all 0.15s;
  background: transparent;
  white-space: nowrap;
}

.neon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Sizes */
.sm { font-size: 11px; padding: 4px 10px; }
.md, :not(.sm):not(.lg) { font-size: 12px; padding: 7px 14px; }
.lg { font-size: 14px; padding: 10px 20px; }

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
