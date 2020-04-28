<template>
  <v-container>
    <v-data-table :headers="headers" :items="hosts" :items-per-page="15" class="elevation-1">
      <template slot="item.show" slot-scope="props">
        <v-btn class="secondary" rounded :to="`/backups/${props.item.name}`">
          list
        </v-btn>
      </template>
      <template slot="item.lastBackupAge" slot-scope="props">
        {{ (props.item.lastBackupAge / (24 * 3600000)).toFixed(2) }}
      </template>
      <template slot="item.lastBackupSize" slot-scope="props">
        {{ props.item.lastBackupSize && calculateFileSize(props.item.lastBackupSize) }}
      </template>
    </v-data-table>
  </v-container>
</template>

<script lang="ts">
import filesize from 'filesize.js';
import { Component, Vue } from 'vue-property-decorator';
import hosts from './Hosts.graphql';
import { HostsQuery } from '../generated/graphql';

@Component({
  apollo: {
    hosts: {
      query: hosts,
      update: ({ hosts }: HostsQuery) =>
        hosts.map(host => ({
          name: host.name,
          lastBackupNumber: host.lastBackup?.number,
          lastBackupAge: host.lastBackup && new Date().getTime() - new Date(host.lastBackup?.startDate).getTime(),
          lastBackupSize: host.lastBackup?.fileSize,
          state: host.lastBackup?.complete ? 'sucess' : 'failed',
        })),
    },
  },
})
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
  hosts = [];
  /*
  async mounted() {
    const response = await axios.get("/api/hosts");
    this.hosts = response.data.map((host: any) => ({
      name: host.name,
      lastBackupNumber: host.lastBackup?.number,
      lastBackupAge:
        host.lastBackup &&
        new Date().getTime() - new Date(host.lastBackup?.startDate).getTime(),
      lastBackupSize: host.lastBackup?.fileSize,
      state: host.lastBackup?.complete ? "sucess" : "failed",
    }));
  }
  */

  calculateFileSize(size: number) {
    return filesize(size);
  }
}
</script>
