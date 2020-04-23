import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    name: "Dashboard",
    path: "/dashboard",
    component: () =>
      import(/* webpackChunkName: "dashboard" */ "../views/Dashboard.vue")
  },
  {
    name: "Hosts",
    path: "/hosts",
    component: () =>
      import(/* webpackChunkName: "hosts" */ "../views/Hosts.vue")
  },
  {
    name: "RunningTasks",
    path: "/tasks",
    component: () =>
      import(/* webpackChunkName: "tasks" */ "../views/RunningTasks.vue")
  },
  {
    name: "Logs",
    path: "/logs",
    component: () => import(/* webpackChunkName: "logs" */ "../views/Logs.vue")
  },
  {
    name: "Backups",
    path: "/backups/:host",
    component: () =>
      import(/* webpackChunkName: "backups" */ "../views/Backups.vue")
  },
  {
    name: "BackupsBrowse",
    path: "/backups/:host/:number",
    component: () =>
      import(/* webpackChunkName: "browse" */ "../views/BackupsBrowse.vue")
  },
  {
    path: "/",
    redirect: "/dashboard"
  }
];

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes
});

export default router;
