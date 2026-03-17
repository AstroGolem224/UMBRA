<template>
  <div id="umbra-root" :data-theme="configStore.config.theme">
    <div class="scanlines" aria-hidden="true" />
    <div class="noise-overlay" aria-hidden="true" />
    <AppLayout />
  </div>
</template>

<script setup lang="ts">
import AppLayout from "@/components/layout/AppLayout.vue";
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
</style>
