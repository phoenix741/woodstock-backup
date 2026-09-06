// Composables
import { createRouter, createWebHistory } from 'vue-router';
import { useAuth } from '@/composables/useAuth';

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
        // Server-side, running/listing archive profiles is admin-only (a profile's host
        // selection can include hosts the caller doesn't own — see
        // server-rs/src/graphql/resolvers/query.rs::archive_profiles). Hidden from
        // restricted users here too, not just from the menu, so navigating here directly
        // doesn't land on a page of buttons that all fail server-side.
        meta: { requiresAdmin: true },
        component: () => import(/* webpackChunkName: "archive" */ '@/views/ArchiveView.vue'),
      },
      {
        path: 'archive/:profileName',
        name: 'ArchiveProfile',
        meta: { requiresAdmin: true },
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
        // Cleanup/fsck (server-rs/src/graphql/resolvers/mutation.rs::cleanup_pool /
        // check_and_fix_pool) are admin-only, global maintenance operations with no host to
        // scope them to. Hidden from restricted users, same reasoning as Archive above.
        meta: { requiresAdmin: true },
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

declare module 'vue-router' {
  interface RouteMeta {
    /** Redirects a restricted (non-admin) user away instead of rendering the page —
     * the real enforcement is server-side (see the mutations/queries these pages call),
     * this only avoids showing a page of controls that would all fail. */
    requiresAdmin?: boolean;
  }
}

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

router.beforeEach(async (to) => {
  if (!to.meta.requiresAdmin) return true;

  const auth = useAuth();
  await auth.ensureLoaded();
  if (auth.enabled.value && !auth.isAdmin.value) {
    return { name: 'Devices' };
  }
  return true;
});

export default router;
