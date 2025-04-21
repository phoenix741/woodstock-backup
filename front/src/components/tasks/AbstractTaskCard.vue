<template>
  <v-sheet rounded="lg">
    <v-expansion-panels>
      <v-expansion-panel :hide-actions="!$slots.details" :readonly="!$slots.details">
        <v-expansion-panel-title>
          <v-container>
            <v-row no-gutters>
              <v-col cols="8">
                <span class="font-weight-light">
                  <v-icon start size="small">mdi-{{ icon }}</v-icon>
                  {{ title }}
                </span>
              </v-col>
              <v-col cols="4">
                <div class="text-right font-weight-light">{{ subtitle }}</div>
              </v-col>
            </v-row>
            <!-- Global Progress -->
            <v-row class="pt-5" no-gutters>
              <v-col cols="12">
                <v-progress-linear :color="progressColor" striped :model-value="progressPercent" :indeterminate="false"
                  height="25">
                  <strong v-if="backupErrorState">
                    ERROR: {{ errorMessage }}
                  </strong>
                  <strong v-else>
                    {{ toPercent(progressPercent) }}
                    <template v-if="progressMessage">- {{ progressMessage }}</template>
                  </strong>
                </v-progress-linear>
              </v-col>
            </v-row>
            <!-- Backup Details -->
            <v-row class="pt-3" no-gutters>
              <v-col cols="12">
                <div class="text-caption text-grey">
                  <slot name="tags"></slot>
                </div>
              </v-col>
            </v-row>
          </v-container>
        </v-expansion-panel-title>

        <v-expansion-panel-text>
          <slot name="details"></slot>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>
  </v-sheet>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import { defineProps } from 'vue';
import { toPercent } from '@/components/hosts/hosts.utils';

const props = defineProps<{
  title: string;
  subtitle?: string;
  icon: string;
  progressPercent: number;
  progressMessage?: string;
  errorMessage?: string;
  backupErrorState?: boolean;
}>();

const progressColor = computed(() => {
  if (props.backupErrorState) {
    return 'error';
  }
  return 'primary';
});

</script>
