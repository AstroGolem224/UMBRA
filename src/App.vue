<template>
  <div id="umbra-root" :data-theme="configStore.config.theme">
    <EmberCanvas />
    <div class="scanlines" aria-hidden="true" />
    <div class="noise-overlay" aria-hidden="true" />
    <div class="app-content">
      <AppLayout />
    </div>
  </div>
</template>

<script setup lang="ts">
import AppLayout from "@/components/layout/AppLayout.vue";
import EmberCanvas from "@/components/atmosphere/EmberCanvas.vue";
import { useConfigStore } from "@/stores/useConfigStore";
import { onMounted } from "vue";

const configStore = useConfigStore();

onMounted(async () => {
  await configStore.load();
});
</script>

<style>
#umbra-root {
  width: 100vw;
  height: 100vh;
  background: var(--bg-primary);
  overflow: hidden;
}

.app-content {
  position: relative;
  z-index: 10;
  width: 100%;
  height: 100%;
}
</style>
