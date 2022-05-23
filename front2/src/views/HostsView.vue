<script lang="ts" setup>
import { useQuery } from "villus";
import { computed } from "vue";
import { useRouter } from "vue-router";
import { HostsDocument, type HostsQuery } from "../graphql";
import { formatAge } from "../filters/formatDate";
import { formatFilesize } from "../filters/formatFilesize";
import { getColor } from "../utils/hosts.utils";
import HostRepartitionChartsComponent from "@/components/metrics/cards/hosts/HostRepartitionChartsComponent.vue";
import HostSuccessFailureChartsComponent from "@/components/metrics/cards/hosts/HostSuccessFailureChartsComponent.vue";

function getState(host: HostsQuery["hosts"][0]) {
  if (host.lastBackupState) {
    return host.lastBackupState;
  } else {
    if (host.lastBackup) {
      if (!host.lastBackup.complete) {
        return "failed";
      }
      return "idle";
    } else {
      return null;
    }
  }
}

const { data } = useQuery({
  query: HostsDocument,
});

const router = useRouter();

const hosts = computed(() => {
  return (data?.value?.hosts || []).map((host) => ({
    name: host.name,
    lastBackupNumber: host.lastBackup?.number,
    lastBackupAge:
      host.lastBackup &&
      new Date().getTime() - new Date(host.lastBackup?.startDate).getTime(),
    lastBackupSize: parseInt(host.lastBackup?.fileSize || "0"),
    lastBackupCompressedSize: parseInt(
      host.lastBackup?.compressedFileSize || "0"
    ),
    state: getState(host),
    configuration: host.configuration,
  }));
});

const countByState = computed(() => {
  const cbs = hosts.value.reduce((acc, host) => {
    const state = host.state || "unknown";
    acc[state] = (acc[state] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  return Object.keys(cbs).map((state) => ({ name: state, value: cbs[state] }));
});

function navigateTo(hostname: string) {
  router.push(`/backups/${hostname}`);
}
</script>

<template>
  <v-container>
    <v-row>
      <v-col cols="9">
        <HostRepartitionChartsComponent
          v-if="data?.statistics.hosts?.length"
          :hosts="data?.statistics.hosts || []"
        ></HostRepartitionChartsComponent>
      </v-col>
      <v-col cols="3">
        <HostSuccessFailureChartsComponent
          :countByState="countByState"
        ></HostSuccessFailureChartsComponent>
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="12">
        <v-table>
          <thead>
            <tr>
              <th class="text-left">Host</th>
              <th class="text-left">Last backup number</th>
              <th class="text-left">Last backup age (days)</th>
              <th class="text-left">Last backup size (Gb)</th>
              <th class="text-left">Last backup compressed size (Gb)</th>
              <th class="text-left">State</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in hosts"
              :key="item.name"
              @click="navigateTo(item.name)"
            >
              <td>{{ item.name }}</td>
              <td>{{ item.lastBackupNumber }}</td>
              <td>{{ formatAge(item.lastBackupAge) }}</td>
              <td>{{ formatFilesize(item.lastBackupSize) }}</td>
              <td>{{ formatFilesize(item.lastBackupCompressedSize) }}</td>
              <td>
                <v-chip v-if="item.state" :color="getColor(item.state)" dark>
                  {{ item.state }}
                </v-chip>
                <v-chip v-if="!item.configuration?.schedule?.activated" dark
                  >disabled</v-chip
                >
              </td>
            </tr>
          </tbody>
        </v-table>
      </v-col>
    </v-row>
  </v-container>
</template>
