<template>
  <v-container>
    <div>
      <h1>Woodstock Backup</h1>

      <br />
      <p class="text-justify">
        Woodstock Backup software is a powerful and user-friendly tool designed to protect your network's data. It
        connects to a list of machines and backs up each one incrementally, using a custom schedule for each machine.
      </p>
      <br />

      <p class="text-justify">
        The backups are stored in a compressed and shared pool that allows for efficient deduplication. With a 4MB
        block-level deduplication, Woodstock Backup ensures that only modified data is saved, making it especially
        effective for backing up virtual machines. We also provide tools to verify the integrity of your backups and
        pool.
      </p>
      <br />

      <p class="text-justify">
        With a modern user interface, our software allows you to easily access and view the status of your backups. We
        also offer REST and GraphQL APIs to extend the interface and integrate with other tools.
      </p>
      <br />

      <p class="text-justify">Distributed under the MIT license</p>
    </div>
  </v-container>
</template>

<script lang="ts" setup></script>
