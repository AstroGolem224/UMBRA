<template>
  <header class="view-hero">
    <div class="view-hero__copy">
      <p class="view-hero__kicker">{{ kicker }}</p>
      <h1 class="view-hero__title">{{ title }}</h1>
      <p v-if="subtitle" class="view-hero__subtitle">{{ subtitle }}</p>
    </div>

    <div class="view-hero__meta">
      <button
        v-if="themeToggle"
        class="view-hero__pill view-hero__pill--button"
        type="button"
        @click="cycleTheme"
      >
        {{ currentThemeLabel }}
      </button>
      <slot name="meta" />
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useConfigStore } from "@/stores/useConfigStore";

interface Props {
  kicker: string;
  title: string;
  subtitle?: string;
  themeToggle?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  subtitle: "",
  themeToggle: true,
});

void props;

const configStore = useConfigStore();
const themeOrder = ["ember", "neon", "light"] as const;

const currentThemeLabel = computed(() => `${configStore.config.theme} theme`);

function cycleTheme() {
  const currentTheme = configStore.config.theme as (typeof themeOrder)[number];
  const currentIndex = themeOrder.includes(currentTheme) ? themeOrder.indexOf(currentTheme) : 0;
  const nextTheme = themeOrder[(currentIndex + 1) % themeOrder.length];
  configStore.setTheme(nextTheme);
}
</script>

<style scoped>
.view-hero {
  min-height: var(--view-hero-min-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: var(--view-hero-padding-y) var(--view-hero-padding-x);
  border-radius: var(--radius-2xl);
  border: 1px solid color-mix(in srgb, var(--glass-border) 90%, transparent);
  background:
    radial-gradient(circle at top right, color-mix(in srgb, var(--accent) 10%, transparent), transparent 36%),
    linear-gradient(135deg, color-mix(in srgb, var(--bg-secondary) 93%, transparent), color-mix(in srgb, var(--bg-primary) 96%, transparent));
  box-shadow: 0 18px 36px rgba(0, 0, 0, 0.12);
}

.view-hero__copy {
  max-width: 720px;
}

.view-hero__kicker {
  margin: 0;
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--text-muted) 86%, var(--text-secondary));
}

.view-hero__title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 40px;
  font-weight: 800;
  letter-spacing: 0.08em;
  line-height: 0.95;
  color: var(--text-primary);
}

.view-hero__subtitle {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.view-hero__meta {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
}

.view-hero__pill,
:deep(.view-hero-pill) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 32px;
  padding: 6px 12px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 84%, transparent);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.04em;
  line-height: 1;
  text-decoration: none;
}

.view-hero__pill--button {
  cursor: pointer;
  transition: transform 0.16s ease, border-color 0.16s ease, background 0.16s ease;
}

.view-hero__pill--button:hover {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--accent) 22%, var(--glass-border));
  background: color-mix(in srgb, var(--accent) 8%, var(--glass-bg));
}

:deep(.view-hero-pill.is-stale),
:deep(.view-hero-pill.is-danger) {
  color: var(--accent-error);
  border-color: rgba(239, 68, 68, 0.28);
}

@media (max-width: 960px) {
  .view-hero {
    flex-direction: column;
    align-items: flex-start;
  }

  .view-hero__meta {
    justify-content: flex-start;
  }
}
</style>

<style>
[data-theme="light"] .view-hero {
  border-color: rgba(8, 145, 178, 0.14);
  background:
    radial-gradient(circle at top right, rgba(8, 145, 178, 0.12), transparent 34%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.98), rgba(236, 243, 248, 0.94));
  box-shadow: 0 18px 32px rgba(15, 23, 42, 0.06);
}

[data-theme="light"] .view-hero__pill,
[data-theme="light"] .view-hero-pill {
  background: rgba(255, 255, 255, 0.8);
}
</style>
