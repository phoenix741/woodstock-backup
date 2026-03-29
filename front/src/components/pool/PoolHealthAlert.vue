<template>
  <div v-if="poolHealth && !poolHealth.healthy" class="pool-health-alert">
    <v-alert type="error" variant="tonal" prominent border="start" class="mb-4">
      <template v-slot:prepend>
        <v-icon size="large">mdi-alert-circle</v-icon>
      </template>
      <v-alert-title class="text-h6">Pool Integrity Warning: Dirty State Detected</v-alert-title>
      <div class="text-body-1 mt-2">
        The storage pool is in a corrupted state (dirty file detected). This indicates that reference counting
        operations were interrupted during a previous backup or removal.
      </div>
      <div class="text-body-2 mt-2"><strong>Pending operations:</strong> {{ toNumber(poolHealth.pendingCount) }}</div>
      <div class="mt-4">
        <v-btn
          color="primary"
          variant="elevated"
          @click="runFsckRepair"
          :loading="fsckLoading"
          prepend-icon="mdi-wrench"
        >
          Run Repair (fsck --repair)
        </v-btn>
        <v-btn
          color="secondary"
          variant="text"
          class="ml-2"
          href="/docs/troubleshooting/POOL_REPAIR.md"
          target="_blank"
        >
          View Repair Guide
        </v-btn>
      </div>
    </v-alert>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useQuery } from '@vue/apollo-composable';
import gql from 'graphql-tag';
import { toNumber } from '@/components/hosts/hosts.utils';
import { usePool } from '@/utils/pool';

// GraphQL Query
const POOL_HEALTH_QUERY = gql`
  query PoolHealth {
    poolHealth {
      healthy
      isDirty
      pendingCount
    }
  }
`;

// Poll every 30s: cache-and-network avoids "empty" flash between polls
// and removes systematic re-render from network-only null state
const { result, refetch } = useQuery(POOL_HEALTH_QUERY, null, {
  pollInterval: 30000,
  fetchPolicy: 'cache-and-network',
});

const poolHealth = computed(() => result.value?.poolHealth);

// Use existing fsck mutation from pool utils
const { fsckPool } = usePool();
const fsckLoading = ref(false);

const runFsckRepair = async () => {
  fsckLoading.value = true;
  try {
    await fsckPool({ fix: true, verifyChunks: false });
    // Refetch pool health after repair
    await refetch();
  } catch (error) {
    console.error('Failed to run fsck repair:', error);
  } finally {
    fsckLoading.value = false;
  }
};
</script>

<style scoped>
.pool-health-alert {
  margin-bottom: 1rem;
}

code {
  background-color: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}
</style>
