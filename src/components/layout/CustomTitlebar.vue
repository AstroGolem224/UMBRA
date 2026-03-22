<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-copy" data-tauri-drag-region>
      <p class="titlebar-kicker">umbra</p>
      <div class="titlebar-line">
        <span class="titlebar-title">mission control</span>
        <span class="titlebar-dot" />
        <span class="titlebar-sub">desktop orchestrator</span>
      </div>
    </div>
    <div class="titlebar-controls">
      <button class="ctrl-btn" title="Minimieren" @click="minimize">
        <span>-</span>
      </button>
      <button class="ctrl-btn" title="Maximieren" @click="toggleMaximize">
        <span>[]</span>
      </button>
      <button class="ctrl-btn close" title="Schliessen" @click="closeWindow">
        <span>x</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

const isTauri = Boolean((window as any).__TAURI_INTERNALS__);

async function minimize() {
  if (!isTauri) return;
  await getCurrentWindow().minimize();
}

async function toggleMaximize() {
  if (!isTauri) return;
  const win = getCurrentWindow();
  const maximized = await win.isMaximized();

  if (maximized) {
    await win.unmaximize();
    return;
  }

  await win.maximize();
}

async function closeWindow() {
  if (!isTauri) return;
  await getCurrentWindow().close();
}
</script>

<style scoped>
.titlebar {
  height: var(--titlebar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 0 16px;
  border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--bg-secondary) 92%, transparent), color-mix(in srgb, var(--bg-primary) 96%, transparent));
  user-select: none;
  -webkit-user-select: none;
}

.titlebar-copy {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.titlebar-kicker,
.titlebar-sub,
.ctrl-btn {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.titlebar-kicker,
.titlebar-sub {
  color: var(--text-muted);
}

.titlebar-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.titlebar-title {
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 600;
  text-transform: lowercase;
}

.titlebar-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--accent) 72%, transparent);
}

.titlebar-controls {
  display: flex;
  gap: 6px;
}

.ctrl-btn {
  width: 30px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
  border-radius: var(--radius-xs);
  background: color-mix(in srgb, var(--glass-bg) 92%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.ctrl-btn:hover {
  border-color: color-mix(in srgb, var(--accent) 24%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--text-primary);
}

.ctrl-btn.close:hover {
  border-color: rgba(239, 68, 68, 0.28);
  background: rgba(239, 68, 68, 0.12);
  color: var(--accent-error);
}
</style>
