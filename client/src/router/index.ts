import Vue from 'vue';
import VueRouter, { RouteConfig } from 'vue-router';

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    name: 'Dashboard',
    path: '/dashboard',
    component: () => import(/* webpackChunkName: "dashboard" */ '../views/Dashboard.vue'),
  },
  {
    name: 'Hosts',
    path: '/hosts',
    component: () => import(/* webpackChunkName: "hosts" */ '../views/Hosts.vue'),
  },
  {
    name: 'QueueTasks',
    path: '/tasks/:state',
    props: true,
    component: () => import(/* webpackChunkName: "tasks" */ '../views/QueueTasks.vue'),
  },
  {
    name: 'Logs',
    path: '/logs',
    component: () => import(/* webpackChunkName: "logs" */ '../views/Logs.vue'),
  },
  {
    name: 'Backups',
    path: '/backups/:hostname',
    props: true,
    component: () => import(/* webpackChunkName: "backups" */ '../views/Backups.vue'),
  },
  {
    name: 'BackupsBrowse',
    path: '/backups/:hostname/:number',
    props: true,
    component: () => import(/* webpackChunkName: "browse" */ '../views/BackupsBrowse.vue'),
  },
  {
    path: '/',
    redirect: '/dashboard',
  },
];

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes,
});

export default router;
