<script lang="ts" setup>
import { ref } from "vue";
import LogComponent from "@/components/monitoring/LogComponent.vue";

const logfile = ref("application");
</script>

<template>
  <v-tabs v-model="logfile" fixed-tabs>
    <v-tab value="application"> Application logs </v-tab>
    <v-tab value="exceptions"> Exceptions logs </v-tab>
  </v-tabs>
  <v-container>
    <v-window v-model="logfile">
      <v-window-item value="application">
        <LogComponent
          :url="`/api/server/log/application.log?tailable=false`"
        ></LogComponent>
      </v-window-item>

      <v-window-item value="exceptions">
        <LogComponent
          :url="`/api/server/log/exceptions.log?tailable=false`"
        ></LogComponent>
      </v-window-item>
    </v-window>
  </v-container>
</template>

<style scoped>
.v-container {
  border: 0;
  height: calc(100% - 48px);
  width: 100%;
}

.v-window,
.v-window-item {
  border: 0;
  height: 100%;
  width: 100%;
}
</style>
