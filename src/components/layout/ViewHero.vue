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
  min-height: 28px;
  padding: 5px 11px;
  border-radius: var(--radius-pill);
  border: none;
  background: color-mix(in srgb, var(--accent) 8%, var(--bg-surface));
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.06em;
  line-height: 1;
  text-decoration: none;
  text-transform: uppercase;
}

.view-hero__pill--button {
  cursor: pointer;
  transition: transform 0.16s ease, border-color 0.16s ease, background 0.16s ease;
}

.view-hero__pill--button:hover {
  transform: translateY(-1px);
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-surface));
  color: var(--accent);
}

:deep(.view-hero-pill.is-stale),
:deep(.view-hero-pill.is-danger) {
  color: var(--accent-error);
  background: rgba(239, 68, 68, 0.10);
}

:deep(.view-hero-pill.nominal-pill) {
  color: var(--accent-success);
  background: rgba(76, 175, 80, 0.10);
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
  border-color: rgba(189, 200, 209, 0.48);
  background:
    radial-gradient(circle at top right, rgba(33, 188, 255, 0.16), transparent 34%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.92), rgba(242, 244, 246, 0.88));
  box-shadow: 0 24px 44px rgba(0, 101, 141, 0.08);
}

[data-theme="light"] .view-hero__pill,
[data-theme="light"] .view-hero-pill {
  background: rgba(0, 101, 141, 0.06);
  color: #3d4850;
}

[data-theme="light"] .view-hero-pill.nominal-pill {
  color: #166534;
  background: rgba(22, 163, 74, 0.08);
}

[data-theme="light"] .view-hero-pill.is-stale,
[data-theme="light"] .view-hero-pill.is-danger {
  color: #b91c1c;
  background: rgba(239, 68, 68, 0.08);
}
</style>
