import Vue from 'vue';
import filesize from 'filesize.js';

Vue.filter('filesize', function(value: number) {
  if (value === null || value === undefined) return '';
  return filesize(value);
});
