<template>
  <v-container>
    <v-row>
      <v-col>
        <ArchiveProfileCard :profile-name="profileName" />
      </v-col>
    </v-row>
    <v-row>
      <v-col class="d-flex justify-end">
        <ArchiveRun :profile-name="profileName" />
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import { useRoute } from 'vue-router';

import ArchiveProfileCard from '@/components/archiving/ArchiveProfileCard.vue';
import ArchiveRun from './dialogs/ArchiveRun.vue';

const route = useRoute();

const profileName = Array.isArray(route.params.profileName) ? route.params.profileName[0] : route.params.profileName;
</script>
