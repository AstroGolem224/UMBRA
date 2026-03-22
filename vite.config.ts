import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

const host = process.env.TAURI_DEV_HOST;
const isTauriDev = Boolean(process.env.TAURI_ENV_TARGET_TRIPLE);
const isVitest = process.env.VITEST === "true";

// When running Vite standalone (browser preview), shim Tauri IPC modules
const tauriMock = path.resolve(__dirname, "./src/lib/tauri-mock.ts");
const tauriAliases = isTauriDev || isVitest
  ? {}
  : {
      "@tauri-apps/api/core": tauriMock,
      "@tauri-apps/api/event": tauriMock,
      "@tauri-apps/api/window": tauriMock,
    };

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      ...tauriAliases,
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
  },
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    allowedHosts: ["host.docker.internal"],
    hmr: host
      ? { protocol: "ws", host, port: 1431 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  preview: {
    host: host || "0.0.0.0",
    allowedHosts: ["host.docker.internal"],
  },
});
