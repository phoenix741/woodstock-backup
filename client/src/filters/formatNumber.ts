import Vue from 'vue';
import numeral from 'numeral';

Vue.filter('formatPercent', function(value: number) {
  if (value === null || value === undefined) return '';
  return numeral(value / 100).format('0.00 %');
});
