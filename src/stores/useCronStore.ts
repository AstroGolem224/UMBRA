import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CronJob } from "@/interfaces";

export const useCronStore = defineStore("cron", () => {
  const jobs = ref<CronJob[]>([]);
  const loading = ref(false);
  const runningId = ref<string | null>(null);
  const lastOutput = ref<string | null>(null);
  const error = ref<string | null>(null);

  async function loadJobs() {
    loading.value = true;
    error.value = null;
    try {
      jobs.value = await invoke<CronJob[]>("list_cron_jobs");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function createJob(name: string, schedule: string, command: string) {
    const job = await invoke<CronJob>("create_cron_job", { name, schedule, command });
    jobs.value.push(job);
    return job;
  }

  async function toggleJob(id: string) {
    const enabled = await invoke<boolean>("toggle_cron_job", { id });
    const job = jobs.value.find((j) => j.id === id);
    if (job) job.enabled = enabled;
  }

  async function deleteJob(id: string) {
    await invoke("delete_cron_job", { id });
    jobs.value = jobs.value.filter((j) => j.id !== id);
  }

  async function runNow(id: string) {
    runningId.value = id;
    error.value = null;
    try {
      const output = await invoke<string>("run_cron_job_now", { id });
      lastOutput.value = output;
      const job = jobs.value.find((j) => j.id === id);
      if (job) {
        job.lastStatus = "ok";
        job.lastRun = new Date().toISOString();
        job.lastOutput = output;
      }
    } catch (e) {
      lastOutput.value = String(e);
      const job = jobs.value.find((j) => j.id === id);
      if (job) job.lastStatus = "error";
      error.value = String(e);
    } finally {
      runningId.value = null;
    }
  }

  function setupLiveUpdates() {
    listen<{ id: string; status: string; output: string }>("cron-job-ran", (event) => {
      const { id, status, output } = event.payload;
      const job = jobs.value.find((j) => j.id === id);
      if (job) {
        job.lastStatus = status as CronJob["lastStatus"];
        job.lastRun = new Date().toISOString();
        job.lastOutput = output;
      }
      lastOutput.value = output;
    });
  }

  return {
    jobs,
    loading,
    runningId,
    lastOutput,
    error,
    loadJobs,
    createJob,
    toggleJob,
    deleteJob,
    runNow,
    setupLiveUpdates,
  };
});
