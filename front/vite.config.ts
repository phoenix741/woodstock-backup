import { fileURLToPath, URL } from 'node:url';

import { defineConfig } from 'vite';
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
  define: { 'process.env': {} },
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
    },
  },
});
