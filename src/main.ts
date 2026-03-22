import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import "@/assets/styles/base.css";
import "@/assets/styles/glassmorphism.css";
import "@/assets/styles/neon.css";

const app = createApp(App);

function emitGlobalError(message: string) {
  window.dispatchEvent(new CustomEvent("umbra:error", { detail: { message } }));
}

app.config.errorHandler = (err, _instance, info) => {
  console.error("[umbra] vue error", err, info);
  emitGlobalError(`${String(err)}${info ? ` (${info})` : ""}`);
};

window.addEventListener("unhandledrejection", (event) => {
  emitGlobalError(String(event.reason ?? "Unhandled promise rejection"));
});

app.use(createPinia());
app.use(router);
app.mount("#app");
