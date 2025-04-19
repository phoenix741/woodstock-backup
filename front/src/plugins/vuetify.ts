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
    defaultTheme: 'light',
    themes: {
      light: {
        colors: {
          background: '#fdfaf6',
          'on-background': '#4e3b2c',

          surface: '#f4eee4',
          'on-surface': '#3b2a1a',

          'surface-bright': '#fffaf3',
          'on-surface-bright': '#5c4433',

          'surface-light': '#f9f1e7',
          'on-surface-light': '#523c2b',

          'surface-variant': '#523c2b',
          'on-surface-variant': '#3e2e20',

          primary: '#c59c6c',
          'on-primary': '#2e1e0f',

          'primary-darken-1': '#a57d4f',
          'on-primary-darken-1': '#fdfaf6',

          secondary: '#8c6b4c',
          'on-secondary': '#fffaf3',

          'secondary-darken-1': '#6b5139',
          'on-secondary-darken-1': '#f4eee4',

          success: '#4caf50',
          'on-success': '#ffffff',

          warning: '#fbc02d',
          'on-warning': '#3e2e20',

          error: '#b00020',
          'on-error': '#ffffff',

          info: '#4a86e8',
          'on-info': '#ffffff',
        },
      },
      dark: {
        dark: true,
        colors: {
          background: '#2c1f16',
          'on-background': '#f1e6d4',

          surface: '#3b2a20',
          'on-surface': '#e8dcc7',

          'surface-bright': '#4b382b',
          'on-surface-bright': '#fff7ec',

          'surface-light': '#5b4334',
          'on-surface-light': '#fef6e9',

          'surface-variant': '#735b4b',
          'on-surface-variant': '#fff5de',

          primary: '#a47a56',
          'on-primary': '#fdf5eb',

          'primary-darken-1': '#805d3c',
          'on-primary-darken-1': '#fff8ef',

          secondary: '#c9a77f',
          'on-secondary': '#2c1f16',

          'secondary-darken-1': '#a6845f',
          'on-secondary-darken-1': '#fff8ec',

          success: '#81c784',
          'on-success': '#1b1b1b',

          warning: '#fdd835',
          'on-warning': '#4E342E',

          error: '#cf6679',
          'on-error': '#2c1f16',

          info: '#5c9ded',
          'on-info': '#ffffff',
        },
      },
    },
  },
});
