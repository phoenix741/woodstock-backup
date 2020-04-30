<template>
  <v-container>
    <v-card>
      <v-data-table show-select :headers="headers" :items="backups" :items-per-page="15">
        <template slot="item.show" slot-scope="props">
          <td class="align-center">
            <v-btn class="secondary" rounded :to="`/backups/${hostname}/${props.item.number}`">
              Browse
            </v-btn>
            <v-btn class="ml-1 secondary" rounded :to="`/backups/${hostname}/${props.item.number}/logs`">
              Show Log
            </v-btn>
          </td>
        </template>
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
        <v-btn class="primary" text>Launch backup</v-btn>
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
    { text: '', value: 'show', sortable: false },
  ];
  backups: BackupsQuery['backups'] = [];
}
</script>
