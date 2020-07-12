import Vue from 'vue';
import { format } from 'date-fns';

Vue.filter('date', function (value: number) {
  if (value === null || value === undefined) return '';
  return format(value, 'MM/dd/yyyy HH:mm');
});

Vue.filter('age', function (value: number) {
  if (value === null || value === undefined) return '';
  return (value / (24 * 3600000)).toFixed(2);
});
