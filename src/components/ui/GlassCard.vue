<template>
  <div class="glass-card" :class="[variant, { clickable }]" @click="clickable && $emit('click')">
    <slot />
  </div>
</template>

<script setup lang="ts">
defineProps<{
  variant?: "default" | "accent" | "danger";
  clickable?: boolean;
}>();
defineEmits(["click"]);
</script>

<style scoped>
.glass-card {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  padding: 16px;
  contain: layout style paint;
}

.glass-card.clickable {
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.glass-card.clickable:hover {
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--accent-dim);
}

.glass-card.accent {
  border-color: var(--accent);
  box-shadow: 0 0 8px var(--accent-dim);
}

.glass-card.danger {
  border-color: var(--accent-error);
  box-shadow: 0 0 8px rgba(255, 50, 50, 0.15);
}

:global([data-theme="light"]) .glass-card.clickable:hover {
  box-shadow: 0 10px 24px rgba(148, 163, 184, 0.16);
}
</style>
