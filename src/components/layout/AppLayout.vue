<template>
  <div class="app-layout">
    <CustomTitlebar />
    <div class="app-body">
      <AppSidebar />
      <main class="main-content">
        <RouterView />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import CustomTitlebar from "./CustomTitlebar.vue";
import AppSidebar from "./AppSidebar.vue";
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
  background: var(--bg-primary);
}
</style>
