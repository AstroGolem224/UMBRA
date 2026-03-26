import { defineStore } from "pinia";
import { ref } from "vue";

export type ToastLevel = "info" | "success" | "warn" | "error";

export type Toast = {
  id: string;
  title?: string;
  message: string;
  level: ToastLevel;
  duration: number;
};

let counter = 0;

export const useToastStore = defineStore("toast", () => {
  const toasts = ref<Toast[]>([]);

  function show(message: string, options?: { title?: string; level?: ToastLevel; duration?: number }) {
    const id = `toast-${++counter}`;
    const level = options?.level ?? "info";
    const duration = options?.duration ?? 4000;

    const toast: Toast = { id, message, level, duration, title: options?.title };
    toasts.value.push(toast);

    if (duration > 0) {
      setTimeout(() => dismiss(id), duration);
    }
  }

  function dismiss(id: string) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  function success(message: string, title?: string) {
    show(message, { level: "success", title });
  }

  function error(message: string, title?: string) {
    show(message, { level: "error", title, duration: 6000 });
  }

  function warn(message: string, title?: string) {
    show(message, { level: "warn", title });
  }

  function info(message: string, title?: string) {
    show(message, { level: "info", title });
  }

  return { toasts, show, dismiss, success, error, warn, info };
});
