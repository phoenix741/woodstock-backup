<template>
  <v-container>
    <v-card>
      <v-data-table
        show-select
        :headers="headers"
        :items="backups"
        :items-per-page="15"
        @click:row="navigateTo($event.number)"
      >
        <template slot="item.startDate" slot-scope="props">
          {{ props.item.startDate | date }}
        </template>
        <template slot="item.duration" slot-scope="props">
          {{ (props.item.duration / 3600) | formatNumber }}
        </template>
        <template slot="item.fileSize" slot-scope="props">
          {{ props.item.fileSize | filesize }}
        </template>
        <template slot="item.existingFileSize" slot-scope="props">
          {{ props.item.existingFileSize | filesize }}
        </template>
        <template slot="item.newFileSize" slot-scope="props">
          {{ props.item.newFileSize | filesize }}
        </template>
        <template slot="item.fileCount" slot-scope="props">
          {{ props.item.fileCount | formatNumber }}
        </template>
        <template slot="item.existingFileCount" slot-scope="props">
          {{ props.item.existingFileCount | formatNumber }}
        </template>
        <template slot="item.newFileCount" slot-scope="props">
          {{ props.item.newFileCount | formatNumber }}
        </template>
      </v-data-table>
      <v-card-actions>
        <v-btn class="primary ml-12" text>Delete</v-btn>
        <v-btn class="primary ml-1" text>Launch backup</v-btn>
        <v-menu offset-y>
          <template v-slot:activator="{ on }">
            <v-btn color="primary ml-1" dark v-on="on">
              Show Log
            </v-btn>
          </template>
          <v-list>
            <v-list-item>
              <v-list-item-title>Transfert Logs</v-list-item-title>
            </v-list-item>
            <v-list-item>
              <v-list-item-title>Error Logs</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import backups from './Backups.graphql';
import { BackupsQuery } from '../generated/graphql';

@Component({
  apollo: {
    backups: {
      query: backups,
      variables() {
        return {
          hostname: this.hostname,
        };
      },
      update: ({ backups }: BackupsQuery) =>
        backups.map(backup => ({
          ...backup,
          duration: backup?.endDate ? backup.endDate - backup.startDate : null,
        })),
    },
  },
})
export default class Backups extends Vue {
  @Prop({
    required: true,
  })
  hostname!: string;

  headers = [
    {
      text: 'Backup#',
      align: 'start',
      value: 'number',
    },
    { text: 'Start date', value: 'startDate' },
    { text: 'Duration (minutes)', value: 'duration' },
    { text: 'Files Count', value: 'fileCount' },
    { text: 'Files Size', value: 'fileSize' },
    { text: 'Existing Files Count', value: 'existingFileCount' },
    { text: 'Existing Files Size', value: 'existingFileSize' },
    { text: 'New Files Count', value: 'newFileCount' },
    { text: 'New Files Size', value: 'newFileSize' },
  ];
  backups: BackupsQuery['backups'] = [];

  navigateTo(n: number) {
    this.$router.push(`/backups/${this.hostname}/${n}`);
  }
}
</script>
