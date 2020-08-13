<template>
  <v-container>
    <v-data-table
      :headers="headers"
      :items="hosts"
      :items-per-page="10"
      item-key="name"
      class="elevation-1"
      :loading="$apollo.queries.hosts.loading"
      loading-text="Loading... Please wait"
      @click:row="navigateTo($event.name)"
    >
      <template slot="item.lastBackupAge" slot-scope="props">
        {{ (props.item.lastBackupAge / (24 * 3600000)).toFixed(2) }}
      </template>
      <template slot="item.lastBackupSize" slot-scope="props">
        {{ props.item.lastBackupSize | filesize }}
      </template>
      <template slot="item.state" slot-scope="props">
        <v-chip v-if="props.item.state" :color="getColor(props.item.state)" dark>{{ props.item.state }}</v-chip>
        <v-chip v-if="props.item.configuration && props.item.configuration.activated" dark>{{
          props.configuration.activated
        }}</v-chip>
      </template>
    </v-data-table>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import hosts from './Hosts.graphql';
import { HostsQuery } from '../generated/graphql';

function getState(host: HostsQuery['hosts'][0]) {
  if (host.lastBackupState) {
    return host.lastBackupState;
  } else {
    if (host.lastBackup) {
      if (!host.lastBackup.complete) {
        return 'failed';
      }
      return 'idle';
    } else {
      return null;
    }
  }
}

@Component({
  apollo: {
    hosts: {
      query: hosts,
      update: ({ hosts }: HostsQuery) =>
        hosts.map((host) => ({
          name: host.name,
          lastBackupNumber: host.lastBackup?.number,
          lastBackupAge: host.lastBackup && new Date().getTime() - new Date(host.lastBackup?.startDate).getTime(),
          lastBackupSize: host.lastBackup?.fileSize,
          state: getState(host),
        })),
      fetchPolicy: 'network-only',
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
  ];
  hosts = [];

  getColor(state: string) {
    switch (state) {
      case 'waiting':
      case 'active':
        return 'blue';
      case 'failed':
        return 'red';
      case 'completed':
        return 'green';
      case 'delayed':
        return 'yellow';
    }
  }

  navigateTo(hostname: string) {
    this.$router.push(`/backups/${hostname}`);
  }
}
</script>
