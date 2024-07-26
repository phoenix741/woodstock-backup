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
          background: '#f5f5f5',
          'on-background': '#26293a',

          surface: '#ffffff',
          'on-surface': '#26293a',

          primary: '#7180e8',
          'on-primary': '#26293a',

          secondary: '#5ecce4',
          'on-secondary': '#26293a',

          success: '#60c177',
          'on-success': '#35444d',

          warning: '#e79d55',
          'on-warning': '#464049',

          error: '#d95e5a',
          'on-error': '#464049',

          info: '#5ecce4',
          'on-info': '#35455a',
        },
      },
      dark: {
        dark: true,
        colors: {
          background: '#26293a',
          'on-background': '#bbbfd5',

          surface: '#303347',
          'on-surface': '#bbbfd5',

          primary: '#7167e8',
          'on-primary': '#e7e5fb',

          secondary: '#5ecce4',
          'on-secondary': '#e7e5fb',

          success: '#60c177',
          'on-success': '#35444d',

          warning: '#e79d55',
          'on-warning': '#464049',

          error: '#d95e5a',
          'on-error': '#464049',

          info: '#5ecce4',
          'on-info': '#35455a',
        },
      },
    },
  },
});
