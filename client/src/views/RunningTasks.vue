<template>
  <v-tabs vertical dark>
    <v-tab>
      Failed
    </v-tab>
    <v-tab>
      Running
    </v-tab>
    <v-tab>
      Waiting
    </v-tab>

    <v-tabs-items>
      <v-subheader>Failed</v-subheader>
      <v-expansion-panels>
        <Task></Task>
        <Task></Task>
        <Task></Task>
      </v-expansion-panels>
    </v-tabs-items>
  </v-tabs>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import Task from '../components/Task.vue';

@Component({
  components: { Task },
})
export default class Hosts extends Vue {}
</script>

<style scoped>
.v-tabs {
  min-height: 93vh;
}
</style>
