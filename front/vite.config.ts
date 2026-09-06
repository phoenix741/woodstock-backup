import { fileURLToPath, URL } from 'node:url';

import { defineConfig } from 'vite';
import { version as appVersion } from './package.json';
import vue from '@vitejs/plugin-vue';
import vueDevTools from 'vite-plugin-vue-devtools';
import vuetify, { transformAssetUrls } from 'vite-plugin-vuetify';

// Markdown
import Markdown from 'unplugin-vue-markdown/vite';
import prism from 'markdown-it-prism';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue({
      template: { transformAssetUrls },
      include: [/\.vue$/, /\.md$/],
    }),
    vueDevTools(),
    // https://github.com/vuetifyjs/vuetify-loader/tree/next/packages/vite-plugin
    vuetify({
      autoImport: true,
    }),
    Markdown({
      markdownItUses: [prism],
    }),
  ],
  define: {
    'process.env': {},
    __APP_VERSION__: JSON.stringify(appVersion),
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
    extensions: ['.js', '.json', '.jsx', '.mjs', '.ts', '.tsx', '.vue'],
  },
  server: {
    port: 8080,
    proxy: {
      '^/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      '^/graphql': {
        target: 'http://localhost:3000',
        ws: true,
        changeOrigin: true,
      },
      // OIDC login/logout/callback routes (server-rs/src/auth/routes.rs) — without this,
      // `/auth/login` hits the Vite dev server instead of api_server, and its SPA fallback
      // serves index.html for that path (no matching Vue route -> blank page) instead of
      // proxying through to the real redirect.
      '^/auth': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
});
