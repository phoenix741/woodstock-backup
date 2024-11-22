/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import '@mdi/font/css/materialdesignicons.css';
import 'vuetify/styles';
import { md3 } from 'vuetify/blueprints';

// Composables
import { createVuetify } from 'vuetify';

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
export default createVuetify({
  blueprint: md3,
  theme: {
    defaultTheme: 'dark',
    themes: {
      light: {
        colors: {
          background: '#FAF9F7',
          'on-background': '#4E342E',

          surface: '#FFFFFF',
          'on-surface': '#6D4C41',

          primary: '#6D4C41',
          'on-primary': '#FFFFFF',

          secondary: '#FFC107',
          'on-secondary': '#4E342E',

          success: '#2E7D32',
          'on-success': '#FFFFFF',

          warning: '#F9A825',
          'on-warning': '#4E342E',

          error: '#C62828',
          'on-error': '#FFFFFF',

          info: '#1976D2',
          'on-info': '#FFFFFF',
        },
      },
      dark: {
        dark: true,
        colors: {
          background: '#2E2A26',
          'on-background': '#D7CCC8',

          surface: '#3E3936',
          'on-surface': '#D7CCC8',

          primary: '#D7CCC8',
          'on-primary': '#4E342E',

          secondary: '#FFC107',
          'on-secondary': '#3E3936',

          success: '#66BB6A',
          'on-success': '#2E2A26',

          warning: '#FFEB3B',
          'on-warning': '#2E2A26',

          error: '#EF5350',
          'on-error': '#2E2A26',

          info: '#BCAAA4',
          'on-info': '#2E2A26',
        },
      },
    },
  },
});
