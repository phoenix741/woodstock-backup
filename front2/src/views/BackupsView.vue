<script lang="ts" setup>
import { useMutation, useQuery } from "villus";
import { useRouter } from "vue-router";
import { computed, ref } from "vue";
import BackupUsageCardComponent from "../components/metrics/cards/backups/BackupUsageCardComponent.vue";
import { formatDate, formatDuration } from "../filters/formatDate";
import { formatFilesize } from "../filters/formatFilesize";
import { formatNumber, formatPercent } from "../filters/formatNumber";
import { BackupsDocument, CreateBackupDocument } from "../graphql";

const props = defineProps<{
  hostname: string;
}>();

const jobCreated = ref(false);
const jobCreatedError = ref(false);
const jobCreatedErrorMessage = ref("");

const variables = computed(() => ({
  hostname: props.hostname,
}));

const { data } = useQuery({
  query: BackupsDocument,
  variables,
});

const router = useRouter();

const { execute } = useMutation(CreateBackupDocument);

const backups = computed(() => {
  return (data?.value?.backups || []).map((backup) => ({
    ...backup,

    fileSize: parseInt(backup.fileSize || "0"),
    newFileSize: parseInt(backup.newFileSize || "0"),
    existingFileSize: parseInt(backup.existingFileSize || "0"),

    compressedFileSize: parseInt(backup.compressedFileSize || "0"),
    newCompressedFileSize: parseInt(backup.newCompressedFileSize || "0"),
    existingCompressedFileSize: parseInt(
      backup.existingCompressedFileSize || "0"
    ),

    duration: backup.endDate ? backup.endDate - backup.startDate : null,
  }));
});

async function createBackup() {
  const { data, error } = await execute({ hostname: props.hostname });
  if (data) {
    jobCreated.value = true;
  }
  if (error) {
    jobCreatedError.value = true;
    jobCreatedErrorMessage.value = error.message;
  }
}

function navigateTo(n: number) {
  router.push(`/backups/${props.hostname}/${n}`);
}
</script>

<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <BackupUsageCardComponent
          :backups="data?.backups || []"
        ></BackupUsageCardComponent>
      </v-col>

      <v-col cols="12">
        <v-card>
          <v-table>
            <thead>
              <tr>
                <th class="text-left">Backup#</th>
                <th class="text-left">Start date</th>
                <th class="text-left">Duration (minutes)</th>
                <th class="text-left">Files Count</th>
                <th class="text-left">Files Size</th>
                <th class="text-left">Compression ratio</th>

                <th class="text-left">Existing Files Count</th>
                <th class="text-left">Existing Files Size</th>

                <th class="text-left">New Files Count</th>
                <th class="text-left">New Files Size</th>
                <th class="text-left">New Compression ratio</th>

                <th class="text-left">Completed</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in backups"
                :key="item.number"
                @click="navigateTo(item.number)"
              >
                <td>{{ item.number }}</td>
                <td>{{ formatDate(item.startDate) }}</td>
                <td>{{ formatDuration(item.duration) }}</td>
                <td>{{ formatNumber(item.fileCount) }}</td>
                <td>{{ formatFilesize(item.fileSize) }}</td>
                <td>
                  {{
                    formatPercent(
                      (item.compressedFileSize / item.fileSize) * 100
                    )
                  }}
                </td>
                <td>{{ formatNumber(item.existingFileCount) }}</td>
                <td>{{ formatFilesize(item.existingFileSize) }}</td>
                <td>{{ formatNumber(item.newFileCount) }}</td>
                <td>{{ formatFilesize(item.newFileSize) }}</td>
                <td>
                  {{
                    formatPercent(
                      (item.newCompressedFileSize / item.newFileSize) * 100
                    )
                  }}
                </td>
                <td>
                  <v-checkbox
                    hide-details
                    v-model="item.complete"
                    disabled
                  ></v-checkbox>
                </td>
              </tr>
            </tbody>
          </v-table>
          <v-divider></v-divider>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn
              color="primary"
              variant="contained-text"
              class="ml-1"
              @click="createBackup()"
              >Launch backup</v-btn
            >
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>

  <v-snackbar v-model="jobCreated" color="info" :timeout="5000">
    Backup launched with success.
  </v-snackbar>

  <v-snackbar v-model="jobCreatedError" color="error" :timeout="5000">
    {{ jobCreatedErrorMessage }}
  </v-snackbar>
</template>
