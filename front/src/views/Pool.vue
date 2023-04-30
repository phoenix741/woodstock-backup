<template>
  <v-container>
    <v-row v-if="!isFetching">
      <v-col cols="12">
        <PoolUsageNbChunkChartsCard
          :nb-chunk-range="result?.statistics.poolUsage?.nbChunkRange ?? []"
        ></PoolUsageNbChunkChartsCard>
      </v-col>
      <v-col cols="3">
        <space-usage-card
          title="Used space"
          icon="mdi-harddisk"
          color="primary"
          :used="result?.statistics.diskUsage?.used ?? 0n"
          :buffer="result?.statistics.poolUsage?.unusedSize ?? 0n"
          :total="result?.statistics.diskUsage?.total ?? 0n"
          :yesterday="result?.statistics.diskUsage?.usedLastMonth ?? 0n"
        ></space-usage-card>
      </v-col>
      <v-col cols="3">
        <space-usage-card
          title="Pool space"
          icon="mdi-zip-box"
          color="secondary"
          :used="result?.statistics.poolUsage?.compressedSize ?? 0n"
          :total="result?.statistics.poolUsage?.size ?? 0n"
          :yesterday="result?.statistics.poolUsage?.compressedSizeLastMonth ?? 0n"
        ></space-usage-card>
      </v-col>
      <v-col cols="3">
        <text-size-card
          title="References"
          icon="mdi-dots-grid"
          color="purple"
          :used="result?.statistics.poolUsage?.nbRef ?? 0"
          :yesterday="result?.statistics.poolUsage?.nbRefLastMonth ?? 0"
        ></text-size-card>
      </v-col>
      <v-col cols="3">
        <text-size-card
          title="Chunks"
          icon="mdi-checkerboard"
          color="amber"
          :used="result?.statistics.poolUsage?.nbChunk ?? 0"
          :yesterday="result?.statistics.poolUsage?.nbChunkLastMonth ?? 0"
        ></text-size-card>
      </v-col>
      <v-col cols="12">
        <PoolUsageCompressedSizeChartsCard
          :compressed-size-range="result?.statistics.poolUsage?.compressedSizeRange ?? []"
        ></PoolUsageCompressedSizeChartsCard>
      </v-col>
    </v-row>
    <v-row v-else>
      <v-col cols="12" class="text-center">
        <v-progress-circular indeterminate></v-progress-circular>
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="8" v-if="!isTaskFetching">
        <!-- Loop over all task (active, complete, failed, waiting, ...) -->
        <!-- Task from refcnt pool with update and subscribe  -->
        <v-row v-for="task in tasks" :key="task.jobId">
          <v-col>
            <TaskCard :task="task"></TaskCard>
          </v-col>
        </v-row>
      </v-col>
      <v-col cols="8" v-else class="text-center">
        <v-progress-circular indeterminate></v-progress-circular>
      </v-col>
      <v-col cols="4">
        <v-row>
          <v-col cols="12">
            <LaunchRefcnt
              button-action="Check reference count"
              action="Check and fix the reference count"
              description="Check and fix the reference count of backups, hosts and pool"
              :with-fix="true"
              :mutate="fsckPoolCallback"
            >
            </LaunchRefcnt>
          </v-col>
        </v-row>
        <v-row>
          <v-col cols="12">
            <LaunchRefcnt
              button-action="Verify checksum"
              action="Verify checksum"
              description="Verify the checksum of all chunks in the pool. This operation can take a very very long time."
              :with-fix="false"
              :mutate="verifyChecksumCallback"
            >
            </LaunchRefcnt>
          </v-col>
        </v-row>
        <v-row>
          <v-col cols="12">
            <LaunchRefcnt
              button-action="Cleanup unused content"
              action="Cleanup unused content"
              description="This will remove all chunks that are not referenced by any backup."
              :with-fix="false"
              :mutate="cleanupPoolCallback"
            >
            </LaunchRefcnt>
          </v-col>
        </v-row>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import SpaceUsageCard from '@/components/pool/SpaceUsageCard.vue';
import TextSizeCard from '@/components/pool/TextSizeCard.vue';
import TaskCard from '@/components/tasks/TaskCard.vue';
import { usePoolStats } from '@/utils/stats';
import { useTasks } from '@/utils/tasks';
import { ref } from 'vue';
import { usePool } from '@/utils/pool';
import LaunchRefcnt from './dialogs/LaunchRefcnt.vue';
import { JobResponse } from '@/generated/graphql';
import { GraphQLError } from 'graphql';
import PoolUsageCompressedSizeChartsCard from '@/components/pool/PoolUsageCompressedSizeChartsCard.vue';
import PoolUsageNbChunkChartsCard from '@/components/pool/PoolUsageNbChunkChartsCard.vue';

const { result, isFetching } = usePoolStats();

const { tasks, isFetching: isTaskFetching } = useTasks(
  ref(['active', 'failed', 'completed', 'waiting']),
  ref('refcnt'),
);

const { cleanupPool, fsckPool, verifyChecksum } = usePool();

async function cleanupPoolCallback(): Promise<{ data?: JobResponse; errors?: readonly GraphQLError[] }> {
  const { data, errors } = (await cleanupPool()) ?? {};
  return {
    data: data?.cleanupPool,
    errors,
  };
}

async function fsckPoolCallback(fix = false): Promise<{ data?: JobResponse; errors?: readonly GraphQLError[] }> {
  const { data, errors } = (await fsckPool({ fix })) ?? {};
  return {
    data: data?.checkAndFixPool,
    errors,
  };
}

async function verifyChecksumCallback(): Promise<{ data?: JobResponse; errors?: readonly GraphQLError[] }> {
  const { data, errors } = (await verifyChecksum()) ?? {};
  return {
    data: data?.verifyChecksum,
    errors,
  };
}
</script>
