<template>
  <p>
    The latest version is
    <a :href="release.url" target="_blank">{{ release.version }}</a>.
  </p>
</template>

<script setup>
import { ref } from "vue";
import packageJson from "../package.json";

const release = ref({
  version: packageJson.version,
  url: `https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/releases/tag/v${packageJson.version}`,
});
</script>
