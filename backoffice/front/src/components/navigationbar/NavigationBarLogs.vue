<template>
  <v-list-group prepend-icon="mdi-file-document-outline" to="/logs">
    <template v-slot:activator>
      <v-list-item-title>Logs</v-list-item-title>
    </template>

    <v-list-item to="/logs/application.log">
      <v-list-item-title>application.log</v-list-item-title>
    </v-list-item>
    <v-list-item to="/logs/exceptions.log">
      <v-list-item-title>exceptions.log</v-list-item-title>
    </v-list-item>
  </v-list-group>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';

@Component({})
export default class NavigationBarLogs extends Vue {}
</script>
