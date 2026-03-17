import { createRouter, createWebHashHistory } from "vue-router";
import DashboardView from "@/views/DashboardView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/dashboard" },
    { path: "/dashboard", component: DashboardView },
    {
      path: "/agents",
      component: () => import("@/views/AgentsView.vue"),
    },
    {
      path: "/notes",
      component: () => import("@/views/NotesView.vue"),
    },
    {
      path: "/launcher",
      component: () => import("@/views/LauncherView.vue"),
    },
    {
      path: "/tasks",
      component: () => import("@/views/TasksView.vue"),
    },
    {
      path: "/cron",
      component: () => import("@/views/CronView.vue"),
    },
    {
      path: "/skills",
      component: () => import("@/views/SkillsView.vue"),
    },
    {
      path: "/settings",
      component: () => import("@/views/SettingsView.vue"),
    },
  ],
});
