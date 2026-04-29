/**
 * main.ts
 *
 * Bootstraps Vuetify and other plugins then mounts the App`
 */

// Components
import 'json-bigint-patch';
import App from './App.vue';

import 'prismjs';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-powershell';
import 'prismjs/components/prism-markdown';
import 'prismjs/components/prism-markup';
import 'prismjs/components/prism-markup-templating';

// Composables
import { createApp } from 'vue';

// Plugins
import { registerPlugins } from '@/plugins';
import { initializeAppLocale } from '@/utils/locale';

const app = createApp(App);

initializeAppLocale();
registerPlugins(app);

app.mount('#app');
