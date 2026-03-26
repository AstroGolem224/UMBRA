<template>
  <div class="app-layout">
    <CustomTitlebar />
    <div class="app-body">
      <AppSidebar />
      <main class="main-content">
        <div class="content-stage">
          <div v-if="renderError" class="render-error">
            <span class="error-icon">err</span>
            <span class="error-msg">{{ renderError }}</span>
            <button class="error-retry" @click="retry">retry</button>
          </div>
          <RouterView v-else />
        </div>
      </main>
    </div>
    <CommandPalette />
    <ToastContainer />
    <OnboardingWizard v-if="showOnboarding" @done="showOnboarding = false" />
  </div>
</template>

<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { computed, onBeforeUnmount, onErrorCaptured, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import CommandPalette from "@/components/shell/CommandPalette.vue";
import OnboardingWizard from "@/components/shell/OnboardingWizard.vue";
import ToastContainer from "@/components/ui/ToastContainer.vue";
import { useConfigStore } from "@/stores/useConfigStore";
import { useTaskStore } from "@/stores/useTaskStore";
import CustomTitlebar from "./CustomTitlebar.vue";
import AppSidebar from "./AppSidebar.vue";

const configStore = useConfigStore();

const router = useRouter();
const taskStore = useTaskStore();
const renderError = ref<string | null>(null);
const onboardingDismissed = ref(false);

const needsSetup = computed(() => {
  if (!configStore.loaded) return false;
  const c = configStore.config;
  return !c.vaultPath && !c.pmToolUrl && !c.repoRootPath;
});

const showOnboarding = computed({
  get: () =>
    needsSetup.value &&
    !onboardingDismissed.value &&
    localStorage.getItem("umbra-onboarding-done") !== "1",
  set: (v: boolean) => {
    if (!v) onboardingDismissed.value = true;
  },
});
let unlistenTraySync: (() => void) | null = null;

onErrorCaptured((err) => {
  renderError.value = String(err);
  return false;
});

function handleGlobalError(event: Event) {
  const detail = (event as CustomEvent<{ message?: string }>).detail;
  renderError.value = detail?.message ?? "unexpected application error";
}

onMounted(async () => {
  window.addEventListener("umbra:error", handleGlobalError as EventListener);
  try {
    unlistenTraySync = await listen("tray-sync-pm", async () => {
      await taskStore.fetchTasks();
    });
  } catch {
    unlistenTraySync = null;
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("umbra:error", handleGlobalError as EventListener);
  unlistenTraySync?.();
  unlistenTraySync = null;
});

function retry() {
  renderError.value = null;
  router.go(0);
}
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--bg-primary) 96%, black 4%), color-mix(in srgb, var(--bg-secondary) 94%, transparent));
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main-content {
  flex: 1;
  overflow: auto;
  padding: var(--stage-edge-pad);
  background:
    radial-gradient(circle at top right, color-mix(in srgb, var(--accent) 6%, transparent), transparent 28%),
    transparent;
}

.content-stage {
  min-height: 100%;
  padding: var(--stage-inner-pad);
}

.render-error {
  min-height: calc(100vh - 160px);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  text-align: center;
}

.error-icon {
  padding: 8px 12px;
  border-radius: var(--radius-pill);
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--accent-error);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.error-msg {
  max-width: 520px;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.6;
}

.error-retry {
  padding: 8px 16px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--accent) 26%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  cursor: pointer;
}

.error-retry:hover {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

@media (max-width: 960px) {
  .main-content {
    padding: 12px;
  }

  .content-stage {
    padding: 16px;
  }
}
</style>
