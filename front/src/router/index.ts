// Composables
import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    component: () => import('@/layouts/default/DefaultLayout.vue'),
    children: [
      {
        path: '',
        name: 'Home',
        redirect: { name: 'Devices' },
      },
      {
        path: 'devices',
        name: 'Devices',
        // route level code-splitting
        // this generates a separate chunk (about.[hash].js) for this route
        // which is lazy-loaded when the route is visited.
        component: () => import(/* webpackChunkName: "devices" */ '@/views/DevicesView.vue'),
      },
      {
        path: 'backups/:deviceId',
        name: 'Backups',
        component: () => import(/* webpackChunkName: "backups" */ '@/views/BackupsView.vue'),
      },
      {
        path: 'backups/:deviceId/:backupId',
        name: 'BackupDetails',
        component: () => import(/* webpackChunkName: "backups" */ '@/views/BackupDetailsView.vue'),
      },
      {
        path: 'archive',
        name: 'Archive',
        component: () => import(/* webpackChunkName: "archive" */ '@/views/ArchiveView.vue'),
      },
      {
        path: 'archive/:profileName',
        name: 'ArchiveProfile',
        component: () => import(/* webpackChunkName: "archive" */ '@/views/ArchiveProfileView.vue'),
      },
      {
        path: 'tasks',
        name: 'Tasks',
        component: () => import(/* webpackChunkName: "tasks" */ '@/views/TasksView.vue'),
      },
      {
        // Rétrocompatibilité : anciennes URLs /tasks/started, /tasks/completed…
        path: 'tasks/:taskFilter',
        redirect: { name: 'Tasks' },
      },
      {
        path: 'pool',
        name: 'Pool',
        component: () => import(/* webpackChunkName: "pool" */ '@/views/PoolView.vue'),
      },
      {
        path: 'events',
        name: 'Events',
        component: () => import(/* webpackChunkName: "pool" */ '@/views/EventsView.vue'),
      },
      {
        path: 'about',
        name: 'About',
        component: () => import(/* webpackChunkName: "about" */ '@/views/AboutView.vue'),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

export default router;
