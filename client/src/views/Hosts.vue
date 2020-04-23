<template>
  <v-container>
    <v-data-table :headers="headers" :items="hosts" :items-per-page="15" class="elevation-1">
      <template slot="item.show" scope="props">
        <td class="align-center">
          <v-btn class="secondary" rounded :to="`/backups/${props.item.name}`">
            list
          </v-btn>
        </td>
      </template>
    </v-data-table>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';

@Component({})
export default class Hosts extends Vue {
  headers = [
    {
      text: 'Host',
      align: 'start',
      value: 'name',
    },
    { text: 'User', value: 'user' },
    { text: 'Last backup number', value: 'lastBackupNumber' },
    { text: 'Last backup age (days)', value: 'lastBackupAge' },
    { text: 'Last backup size (Gb)', value: 'lastBackupSize' },
    { text: 'State', value: 'state' },
    { text: '', value: 'show', sortable: false },
  ];
  hosts = [
    {
      name: 'pc-ulrich',
      user: 'phoenix',
      lastBackupNumber: 5,
      lastBackupAge: 3,
      lastBackupSize: 60,
      state: 'idle',
    },
    {
      name: 'pc-eve',
      user: 'eve',
      lastBackupNumber: 10,
      lastBackupAge: 4,
      lastBackupSize: 350,
      state: 'backup',
    },
    {
      name: 'server',
      user: 'eve',
      lastBackupNumber: 10,
      lastBackupAge: 1,
      lastBackupSize: 100,
      state: 'failed',
    },
  ];
}
</script>
