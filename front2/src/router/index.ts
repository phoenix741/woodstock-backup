import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/pool",
      name: "pool",
      component: () => import("../views/PoolView.vue"),
    },
    {
      path: "/hosts",
      name: "hosts",
      component: () => import("../views/HostsView.vue"),
    },
    {
      path: "/backups/:hostname",
      name: "backups",
      props: true,
      component: () => import("../views/BackupsView.vue"),
    },
    {
      path: "/backups/:hostname/:number",
      name: "backups-view",
      props: true,
      component: () => import("../views/BackupsDetail.vue"),
    },
    {
      path: "/backups/:hostname/browse",
      name: "backups-browse",
      props: true,
      component: () => import("../views/BackupsBrowseView.vue"),
    },
    {
      path: "/logs",
      name: "backups-logs",
      props: true,
      component: () => import("../views/LogView.vue"),
    },
    {
      path: "/tasks/:state",
      name: "queue-tasks",
      props: true,
      component: () => import("../views/QueueView.vue"),
    },
    {
      path: "/about",
      name: "about",
      component: () => import("../views/AboutView.vue"),
    },
  ],
});

export default router;
