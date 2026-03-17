<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-logo" data-tauri-drag-region>
      <span class="logo-text">UMBRA</span>
      <span class="logo-sub">MISSION CONTROL</span>
    </div>
    <div class="titlebar-controls">
      <button class="ctrl-btn minimize" title="Minimieren" @click="minimize">─</button>
      <button class="ctrl-btn maximize" title="Maximieren" @click="toggleMaximize">□</button>
      <button class="ctrl-btn close" title="Schließen" @click="closeWindow">✕</button>
    </div>
  </div>
</template>

<script setup lang="ts">
// getCurrentWindow() only works inside Tauri — guard for browser dev preview
const isTauri = Boolean((window as any).__TAURI_INTERNALS__);

async function minimize() {
  if (!isTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().minimize();
}
async function toggleMaximize() {
  if (!isTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const win = getCurrentWindow();
  const maximized = await win.isMaximized();
  if (maximized) win.unmaximize(); else win.maximize();
}
async function closeWindow() {
  if (!isTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().close();
}
</script>

<style scoped>
.titlebar {
  height: var(--titlebar-height);
  background: rgba(5, 5, 12, 0.95);
  border-bottom: 1px solid var(--glass-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 0 16px;
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
}

.titlebar-logo {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.logo-text {
  font-family: "Iceland", monospace;
  font-size: 16px;
  color: var(--accent-ember);
  text-shadow: 0 0 8px var(--accent-ember-dim);
  letter-spacing: 0.15em;
}

.logo-sub {
  font-family: "Iceland", monospace;
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 0.2em;
}

.titlebar-controls {
  display: flex;
  gap: 4px;
}

.ctrl-btn {
  width: 28px;
  height: 24px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s;
}

.ctrl-btn:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.ctrl-btn.close:hover {
  background: rgba(255, 45, 85, 0.25);
  color: var(--accent-error);
}
</style>
