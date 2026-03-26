<template>
  <div class="skeleton-block" :class="variant" :style="style" />
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    width?: string;
    height?: string;
    variant?: "text" | "card" | "circle" | "bar";
    radius?: string;
  }>(),
  { width: "100%", height: "16px", variant: "text", radius: undefined }
);

const style = computed(() => ({
  width: props.width,
  height: props.height,
  borderRadius: props.radius ?? (props.variant === "circle" ? "999px" : props.variant === "card" ? "var(--radius-lg)" : "4px"),
}));
</script>

<style scoped>
.skeleton-block {
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--glass-bg) 60%, transparent) 0%,
    color-mix(in srgb, var(--glass-border) 40%, transparent) 50%,
    color-mix(in srgb, var(--glass-bg) 60%, transparent) 100%
  );
  background-size: 200% 100%;
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}

@keyframes skeleton-shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

:global([data-theme="light"]) .skeleton-block {
  background: linear-gradient(
    90deg,
    rgba(241, 245, 249, 0.8) 0%,
    rgba(226, 232, 240, 0.6) 50%,
    rgba(241, 245, 249, 0.8) 100%
  );
  background-size: 200% 100%;
}

@media (prefers-reduced-motion: reduce) {
  .skeleton-block { animation: none; }
}
</style>
