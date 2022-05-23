<script lang="ts" setup>
import { ref } from "vue";
import LogComponent from "@/components/monitoring/LogComponent.vue";

const logfile = ref("error");

defineProps<{
  hostname: string;
  number: string;
}>();
</script>

<template>
  <v-tabs v-model="logfile" fixed-tabs>
    <v-tab value="error"> Error logs </v-tab>
    <v-tab value="transfert"> Transfert logs </v-tab>
  </v-tabs>
  <v-container>
    <v-window v-model="logfile">
      <v-window-item value="error">
        <LogComponent
          :url="`/api/hosts/${hostname}/backups/${number}/log/backup.error.log?tailable=false`"
        ></LogComponent>
      </v-window-item>

      <v-window-item value="transfert">
        <LogComponent
          :url="`/api/hosts/${hostname}/backups/${number}/log/backup.log?tailable=false`"
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
