<template>
  <Transition name="onboarding">
    <div v-if="visible" class="onboarding-backdrop">
      <div class="onboarding-card glass-panel">
        <div class="onboarding-header">
          <h2 class="onboarding-title">Welcome to UMBRA</h2>
          <p class="onboarding-subtitle">configure the essentials before you start.</p>
        </div>

        <div class="step-indicator">
          <button
            v-for="(s, i) in steps"
            :key="s.key"
            class="step-dot"
            :class="{ active: step === i, done: i < step }"
            type="button"
            :disabled="i > step"
            @click="step = i"
          >
            {{ i < step ? "done" : i + 1 }}
          </button>
        </div>

        <div class="step-body">
          <template v-if="step === 0">
            <label class="step-label">obsidian vault path</label>
            <p class="step-hint">where your notes are stored. UMBRA reads and writes markdown files here.</p>
            <div class="input-row">
              <input v-model="vaultPath" class="glass-input step-input" type="text" placeholder="D:/Obsidian/my-vault" />
              <button class="step-btn" type="button" @click="pickFolder('vault')">browse</button>
            </div>
          </template>

          <template v-if="step === 1">
            <label class="step-label">pm tool url</label>
            <p class="step-hint">the base API URL for the project management tool. leave empty if you don't use one.</p>
            <input v-model="pmToolUrl" class="glass-input step-input" type="text" placeholder="http://100.115.61.30:8000" />
            <div v-if="pmTestResult" class="test-result" :class="pmTestResult.ok ? 'ok' : 'fail'">
              {{ pmTestResult.message }}
            </div>
            <button v-if="pmToolUrl.trim()" class="step-btn" type="button" :disabled="pmTesting" @click="testPmConnection">
              {{ pmTesting ? "testing..." : "test connection" }}
            </button>
          </template>

          <template v-if="step === 2">
            <label class="step-label">github personal access token</label>
            <p class="step-hint">optional. enables repo browsing and launcher features. needs <code>public_repo</code> scope.</p>
            <input v-model="githubPat" class="glass-input step-input" type="password" placeholder="ghp_..." autocomplete="off" />
          </template>

          <template v-if="step === 3">
            <label class="step-label">repo root path</label>
            <p class="step-hint">where your git repositories live. UMBRA uses this to find local repos for the launcher.</p>
            <div class="input-row">
              <input v-model="repoRootPath" class="glass-input step-input" type="text" placeholder="C:/Users/you/repos" />
              <button class="step-btn" type="button" @click="pickFolder('repo')">browse</button>
            </div>
          </template>
        </div>

        <div class="onboarding-footer">
          <button v-if="step > 0" class="step-btn ghost" type="button" @click="step--">back</button>
          <div class="footer-right">
            <button class="step-btn ghost" type="button" @click="skip">skip setup</button>
            <button v-if="step < steps.length - 1" class="step-btn primary" type="button" @click="step++">next</button>
            <button v-else class="step-btn primary" type="button" :disabled="saving" @click="finish">
              {{ saving ? "saving..." : "finish setup" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useConfigStore } from "@/stores/useConfigStore";

const configStore = useConfigStore();
const emit = defineEmits<{ done: [] }>();

const visible = ref(true);
const step = ref(0);
const saving = ref(false);

const vaultPath = ref(configStore.config.vaultPath || "");
const pmToolUrl = ref(configStore.config.pmToolUrl || "");
const githubPat = ref((configStore.config as Record<string, unknown>).githubPat as string || "");
const repoRootPath = ref(configStore.config.repoRootPath || "");
const pmTesting = ref(false);
const pmTestResult = ref<{ ok: boolean; message: string } | null>(null);

const steps = [
  { key: "vault", label: "Vault" },
  { key: "pm", label: "PM Tool" },
  { key: "github", label: "GitHub" },
  { key: "repo", label: "Repos" },
];

async function pickFolder(target: "vault" | "repo") {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selection = await open({ directory: true, multiple: false, title: `select ${target} path` });
    if (typeof selection === "string") {
      if (target === "vault") vaultPath.value = selection;
      else repoRootPath.value = selection;
    }
  } catch {
    // Browser mode — no file picker
  }
}

async function testPmConnection() {
  pmTesting.value = true;
  pmTestResult.value = null;
  try {
    const resp = await fetch("/pm-api/projects");
    if (resp.ok) {
      const data = await resp.json();
      pmTestResult.value = { ok: true, message: `connected. ${data.length} projects found.` };
    } else {
      pmTestResult.value = { ok: false, message: `server responded with ${resp.status}` };
    }
  } catch (e) {
    pmTestResult.value = { ok: false, message: `connection failed: ${String(e).slice(0, 80)}` };
  } finally {
    pmTesting.value = false;
  }
}

function skip() {
  visible.value = false;
  localStorage.setItem("umbra-onboarding-done", "1");
  emit("done");
}

async function finish() {
  saving.value = true;
  try {
    const updates = {
      ...configStore.config,
      vaultPath: vaultPath.value.trim(),
      pmToolUrl: pmToolUrl.value.trim().replace(/\/+$/, ""),
      repoRootPath: repoRootPath.value.trim(),
    };
    if (githubPat.value.trim()) {
      (updates as Record<string, unknown>).githubPat = githubPat.value.trim();
    }
    await configStore.saveConfig(updates);
    localStorage.setItem("umbra-onboarding-done", "1");
    visible.value = false;
    emit("done");
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.onboarding-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9500;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(6px);
}

.onboarding-card {
  width: min(560px, 92vw);
  padding: 32px;
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.onboarding-header {
  text-align: center;
}

.onboarding-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
  color: var(--text-primary);
}

.onboarding-subtitle {
  margin: 6px 0 0;
  color: var(--text-muted);
  font-size: 13px;
}

.step-indicator {
  display: flex;
  justify-content: center;
  gap: 8px;
}

.step-dot {
  width: 32px;
  height: 32px;
  border-radius: 999px;
  border: 2px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.step-dot.active {
  border-color: var(--accent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.step-dot.done {
  border-color: var(--accent-success);
  color: var(--accent-success);
  background: rgba(34, 197, 94, 0.08);
  font-size: 9px;
}

.step-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 140px;
}

.step-label {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.step-hint {
  margin: 0;
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.step-hint code {
  padding: 1px 5px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--accent);
  font-size: 11px;
}

.step-input {
  width: 100%;
  font-size: 13px;
  padding: 10px 14px;
}

.input-row {
  display: flex;
  gap: 8px;
}

.input-row .step-input {
  flex: 1;
}

.step-btn {
  padding: 8px 16px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--accent) 26%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  cursor: pointer;
  white-space: nowrap;
}

.step-btn:hover { background: color-mix(in srgb, var(--accent) 14%, transparent); }
.step-btn:disabled { opacity: 0.5; cursor: default; }

.step-btn.primary {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.step-btn.primary:hover { filter: brightness(1.08); }

.step-btn.ghost {
  background: transparent;
  border-color: color-mix(in srgb, var(--glass-border) 80%, transparent);
  color: var(--text-secondary);
}

.step-btn.ghost:hover { color: var(--accent); border-color: var(--accent); }

.test-result {
  padding: 8px 12px;
  border-radius: var(--radius-md);
  font-family: var(--font-mono);
  font-size: 11px;
}

.test-result.ok {
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
  color: var(--accent-success);
}

.test-result.fail {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--accent-error);
}

.onboarding-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 8px;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 60%, transparent);
}

.footer-right {
  display: flex;
  gap: 8px;
}

.onboarding-enter-active,
.onboarding-leave-active {
  transition: opacity 0.2s ease;
}

.onboarding-enter-from,
.onboarding-leave-to {
  opacity: 0;
}
</style>
