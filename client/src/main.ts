import '@mdi/font/css/materialdesignicons.css';
import 'roboto-fontface/css/roboto/roboto-fontface.css';

import Vue from 'vue';

import App from './App.vue';
import { createProvider } from './plugins/vue-apollo';
import vuetify from './plugins/vuetify';
import router from './router';

import './filters';

Vue.config.productionTip = false;

new Vue({
  router,
  vuetify,
  apolloProvider: createProvider(),
  render: h => h(App),
}).$mount('#app');
