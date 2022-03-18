import Vue from 'vue';

Vue.filter('unmangle', function (value: string) {
  return decodeURIComponent((value ?? '').toString());
});
