<template>
  <v-app>
    <Menu></Menu>
    <default-view />
    <v-footer
      app
      class="text-caption text-medium-emphasis d-flex justify-space-between align-center"
    >
      <span>
        <a href="https://woodstock.shadoware.org" target="_blank" rel="noopener">Woodstock Backup</a>
        v{{ appVersion }}
      </span>
      <span>
        <a
          href="https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/src/branch/develop/LICENSE"
          target="_blank"
          rel="noopener"
          >MIT License</a
        >
        · <a href="/api-docs" target="_blank" rel="noopener">API</a>
        · <a href="/graphql" target="_blank" rel="noopener">GraphQL</a>
      </span>
    </v-footer>
  </v-app>
</template>

<script lang="ts" setup>
import Menu from '@/components/MenuComponent.vue';
import DefaultView from './ViewLayout.vue';

const appVersion = __APP_VERSION__;
</script>
