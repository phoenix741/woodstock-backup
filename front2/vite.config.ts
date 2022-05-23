import { fileURLToPath, URL } from "url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from "@vuetify/vite-plugin";

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 8080,
    proxy: {
      "^/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "^/graphql": {
        target: "http://localhost:3000",
        ws: true,
        changeOrigin: true,
      },
    },
  },
  plugins: [vue(), vuetify({ autoImport: true, styles: "expose" })],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
});
