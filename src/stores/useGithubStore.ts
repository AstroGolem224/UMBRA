import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RepoInfo } from "@/interfaces";

export const useGithubStore = defineStore("github", () => {
  const repos = ref<RepoInfo[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastFetch = ref<Date | null>(null);

  function repoById(id: string): RepoInfo | undefined {
    return repos.value.find((r) => r.id === id);
  }

  async function loadRepos() {
    loading.value = true;
    error.value = null;
    try {
      const data = await invoke<RepoInfo[]>("get_github_repos");
      repos.value = data ?? [];
      lastFetch.value = new Date();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  return { repos, loading, error, lastFetch, repoById, loadRepos };
});
