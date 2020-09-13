<template>
  <v-container>
    <v-card>
      <v-data-table
        v-model="selection"
        show-select
        :headers="headers"
        :items="backups"
        :items-per-page="10"
        item-key="number"
        show-expand
        :single-expand="true"
        :sort-by.sync="sortBy"
        :sort-desc.sync="sortDesc"
        :loading="$apollo.queries.backups.loading"
        loading-text="Loading... Please wait"
      >
        <template slot="item.startDate" slot-scope="props">
          {{ props.item.startDate | date }}
        </template>
        <template slot="item.duration" slot-scope="props">
          {{ (props.item.duration / (1000 * 60)) | formatNumber }}
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
        <template slot="item.complete" slot-scope="props">
          <v-simple-checkbox v-model="props.item.complete" disabled></v-simple-checkbox>
        </template>
        <template v-slot:expanded-item="{ headers, item }">
          <td :colspan="headers.length">
            <v-card class="mx-auto transparent" flat>
              <v-card-title>Size repartition</v-card-title>
              <BackupChartSize class="mx-auto chart" :backup="item"></BackupChartSize>

              <v-card-actions>
                <v-btn text :to="`/backups/${hostname}/${item.number}`">Browse</v-btn>
                <v-btn-toggle borderless multiple>
                  <v-menu offset-y>
                    <template v-slot:activator="{ on }">
                      <v-btn text v-on="on"> Show Log </v-btn>
                    </template>
                    <v-list>
                      <v-list-item :to="`/backups/${hostname}/${item.number}/logs/backup.log`">
                        <v-list-item-title>Transfert Logs</v-list-item-title>
                      </v-list-item>
                      <v-list-item :to="`/backups/${hostname}/${item.number}/logs/backup.error.log`">
                        <v-list-item-title>Error Logs</v-list-item-title>
                      </v-list-item>
                    </v-list>
                  </v-menu>
                </v-btn-toggle>
              </v-card-actions>
            </v-card>
          </td>
        </template>
      </v-data-table>
      <v-card-actions>
        <v-btn class="primary ml-12" text @click="deleteBackup()">Delete</v-btn>
        <v-btn class="primary ml-1" text @click="createBackup()">Launch backup</v-btn>
      </v-card-actions>
    </v-card>

    <template v-for="(job, key) in jobCreated">
      <v-snackbar v-model="jobCreated[key]" color="info" :timeout="5000" :key="key">
        Launch a backup with job id {{ key }}.
        <v-btn color="secondary" text @click="jobCreated[key] = false"> Close </v-btn>
      </v-snackbar>
    </template>
    <template v-for="(job, key) in jobRemoved">
      <v-snackbar v-model="jobRemoved[key]" color="error" :timeout="5000" :key="key">
        Remove the backup with job id {{ key }}.
        <v-btn color="primary" text @click="jobRemoved[key] = false"> Close </v-btn>
      </v-snackbar>
    </template>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import backups from './Backups.graphql';
import {
  BackupsQuery,
  CreateBackupMutation,
  CreateBackupMutationVariables,
  RemoveBackupMutation,
  RemoveBackupMutationVariables,
} from '../generated/graphql';
import BackupChartSize from '../components/BackupChartSize';
import createBackup from './BackupsCreate.graphql';
import removeBackup from './BackupsRemove.graphql';

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
        backups.map((backup) => ({
          ...backup,
          duration: backup?.endDate ? backup.endDate - backup.startDate : null,
        })),
      fetchPolicy: 'network-only',
    },
  },
  components: {
    BackupChartSize,
  },
})
export default class Backups extends Vue {
  @Prop({
    required: true,
  })
  hostname!: string;
  selection: BackupsQuery['backups'] = [];

  sortBy = 'number';
  sortDesc = true;

  jobCreated: Record<number, boolean> = {};
  jobRemoved: Record<number, boolean> = {};

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
    { text: 'Complete', value: 'complete' },
  ];
  backups: BackupsQuery['backups'] = [];

  async deleteBackup() {
    for (const backup of this.selection) {
      const mutation = await this.$apollo.mutate<RemoveBackupMutation, RemoveBackupMutationVariables>({
        mutation: removeBackup,
        variables: {
          hostname: this.hostname,
          number: backup.number,
        },
      });

      if (mutation.data) {
        Vue.set(this.jobRemoved, mutation.data.removeBackup.id, true);
      }
    }
  }

  async createBackup() {
    const mutation = await this.$apollo.mutate<CreateBackupMutation, CreateBackupMutationVariables>({
      mutation: createBackup,
      variables: {
        hostname: this.hostname,
      },
    });
    if (mutation.data) {
      Vue.set(this.jobCreated, mutation.data.createBackup.id, true);
    }
  }
}
</script>

<style scoped>
.transparent {
  background-color: transparent;
  border-color: transparent !important;
}

.chart {
  height: 250px;
}
</style>
