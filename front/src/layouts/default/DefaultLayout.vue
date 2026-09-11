<template>
  <v-app>
    <Menu></Menu>
    <default-view />
    <v-footer
      app
      class="app-footer text-caption text-medium-emphasis d-flex flex-wrap justify-space-between align-center ga-2"
    >
      <span class="d-flex align-center ga-1">
        <a class="footer-link font-weight-medium" href="https://woodstock.shadoware.org" target="_blank" rel="noopener"
          >Woodstock Backup</a
        >
        <span>v{{ appVersion }}</span>
      </span>
      <span class="d-flex flex-wrap align-center ga-4">
        <a
          class="footer-link"
          href="https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/src/branch/develop/LICENSE"
          target="_blank"
          rel="noopener"
          >MIT License</a
        >
        <a class="footer-link" href="/api-docs" target="_blank" rel="noopener">API</a>
        <a class="footer-link" href="/graphql" target="_blank" rel="noopener">GraphQL</a>
      </span>
    </v-footer>
  </v-app>
</template>

<script lang="ts" setup>
import Menu from '@/components/MenuComponent.vue';
import DefaultView from './ViewLayout.vue';

const appVersion = __APP_VERSION__;
</script>

<style scoped>
.app-footer {
  row-gap: 4px;
}

.footer-link {
  color: inherit;
  text-decoration: none;
}

.footer-link:hover,
.footer-link:focus-visible {
  text-decoration: underline;
}
</style>
