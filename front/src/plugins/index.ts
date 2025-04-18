/**
 * plugins/index.ts
 *
 * Automatically included in `./src/main.ts`
 */

// Plugins
import { loadFonts } from './webfontloader';
import vuetify from './vuetify';
import router from '../router';
import apollo from './apollo';
import setupEchartsTheme, { setupGlobalEchartsTheme } from './echarts-theme';

// Types
import type { App } from 'vue';

export function registerPlugins(app: App) {
  loadFonts();
  setupEchartsTheme();
  setupGlobalEchartsTheme(app); // Configurer le thème ECharts global
  app.use(vuetify).use(router).use(apollo);
}
