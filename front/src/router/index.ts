// Composables
import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    component: () => import('@/layouts/default/Default.vue'),
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
        component: () => import(/* webpackChunkName: "devices" */ '@/views/Devices.vue'),
      },
      {
        path: 'backups/:deviceId',
        name: 'Backups',
        component: () => import(/* webpackChunkName: "backups" */ '@/views/Backups.vue'),
      },
      {
        path: 'tasks/:taskFilter',
        name: 'Tasks',
        props: true,
        component: () => import(/* webpackChunkName: "tasks" */ '@/views/Tasks.vue'),
      },
      {
        path: 'pool',
        name: 'Pool',
        component: () => import(/* webpackChunkName: "pool" */ '@/views/Pool.vue'),
      },
      {
        path: 'about',
        name: 'About',
        component: () => import(/* webpackChunkName: "about" */ '@/views/About.vue'),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

export default router;
