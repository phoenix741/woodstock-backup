<template>
  <v-container>
    <v-row v-if="!isFetching">
      <v-col cols="12">
        <PoolUsageNbChunkChartsCard :nb-chunk-range="result?.statistics.poolUsage?.nbChunkRange ?? []">
        </PoolUsageNbChunkChartsCard>
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
          color="primary-darken-1"
          :used="result?.statistics.poolUsage?.nbRef ?? 0"
          :yesterday="result?.statistics.poolUsage?.nbRefLastMonth ?? 0"
        ></text-size-card>
      </v-col>
      <v-col cols="3">
        <text-size-card
          title="Chunks"
          icon="mdi-checkerboard"
          color="secondary-darken-1"
          :used="result?.statistics.poolUsage?.nbChunk ?? 0"
          :yesterday="result?.statistics.poolUsage?.nbChunkLastMonth ?? 0"
        ></text-size-card>
      </v-col>
      <v-col cols="12">
        <PoolUsageCompressedSizeChartsCard
          :compressed-size-range="result?.statistics.poolUsage?.compressedSizeRange ?? []"
        >
        </PoolUsageCompressedSizeChartsCard>
      </v-col>
    </v-row>
    <v-row v-else>
      <v-col cols="12" class="text-center">
        <v-progress-circular indeterminate></v-progress-circular>
      </v-col>
    </v-row>
    <v-fab app color="primary" size="large" icon>
      <v-icon>mdi-plus</v-icon>
      <v-speed-dial v-model="speedDialOpen" activator="parent">
        <LaunchRefcnt
          button-icon="mdi-check"
          color="error"
          button-text="Check and fix the reference count"
          action="Check and fix the reference count"
          description="Check and fix the reference count of backups, hosts and pool"
          :with-fix="true"
          :mutate="fsckPoolCallback"
        >
        </LaunchRefcnt>
        <LaunchRefcnt
          button-icon="mdi-delete"
          color="info"
          button-text="Cleanup unused content"
          action="Cleanup unused content"
          description="This will remove all chunks that are not referenced by any backup."
          :with-fix="false"
          :mutate="cleanupPoolCallback"
        >
        </LaunchRefcnt>
      </v-speed-dial>
    </v-fab>
  </v-container>
</template>

<script lang="ts" setup>
import PoolUsageCompressedSizeChartsCard from '@/components/pool/PoolUsageCompressedSizeChartsCard.vue';
import PoolUsageNbChunkChartsCard from '@/components/pool/PoolUsageNbChunkChartsCard.vue';
import SpaceUsageCard from '@/components/pool/SpaceUsageCard.vue';
import TextSizeCard from '@/components/pool/TextSizeCard.vue';
import { useFragment } from '@/generated';
import type { JobPoolResponseFragment } from '@/generated/graphql';
import { JobPoolResponseFragmentNode, usePool } from '@/utils/pool';
import { usePoolStats } from '@/utils/stats';
import type { GraphQLFormattedError } from 'graphql';
import LaunchRefcnt from './dialogs/LaunchRefcnt.vue';
import { ref } from 'vue';

const { result, isFetching } = usePoolStats();

const { cleanupPool, fsckPool } = usePool();

const speedDialOpen = ref(false);

async function cleanupPoolCallback(): Promise<{
  response?: JobPoolResponseFragment;
  errors?: readonly GraphQLFormattedError[];
}> {
  const { data, errors } = (await cleanupPool()) ?? {};
  const response = useFragment(JobPoolResponseFragmentNode, data?.cleanupPool);
  return {
    response,
    errors,
  };
}

async function fsckPoolCallback(
  fix = false,
  verifyChunks = false,
): Promise<{ response?: JobPoolResponseFragment; errors?: readonly GraphQLFormattedError[] }> {
  const { data, errors } = (await fsckPool({ fix, verifyChunks })) ?? {};
  const response = useFragment(JobPoolResponseFragmentNode, data?.checkAndFixPool);
  return {
    response,
    errors,
  };
}
</script>
