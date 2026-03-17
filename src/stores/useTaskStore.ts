import { defineStore } from "pinia";
import { ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { Task } from "@/interfaces";

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<Task[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastSync = ref<string | null>(null);

  async function fetchTasks() {
    loading.value = true;
    error.value = null;
    try {
      tasks.value = await invoke<Task[]>("get_pm_tasks");
      lastSync.value = new Date().toISOString();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function setupLiveUpdates() {
    listen<Task[]>("tasks-updated", (event) => {
      tasks.value = event.payload;
      lastSync.value = new Date().toISOString();
    });
  }

  const activeTasks = () => tasks.value.filter((t) => t.status === "in-progress");
  const todoTasks = () => tasks.value.filter((t) => t.status === "todo");

  return { tasks, loading, error, lastSync, fetchTasks, setupLiveUpdates, activeTasks, todoTasks };
});
