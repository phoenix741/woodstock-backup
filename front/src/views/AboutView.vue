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
        The backups are stored in a compressed and shared pool that allows for efficient deduplication. With a 16MB
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

      <br />
      <h2>System</h2>
      <br />
      <v-table density="compact">
        <thead>
          <tr>
            <th>Service</th>
            <th>Instances</th>
            <th>Version(s)</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="service in services" :key="service.serviceType">
            <td>{{ service.serviceType }}</td>
            <td>{{ service.count }}</td>
            <td>{{ service.versions }}</td>
          </tr>
        </tbody>
      </v-table>
    </div>
  </v-container>
</template>

<script lang="ts" setup>
import { useSystemInfo } from '@/utils/systemInfo';
import { computed } from 'vue';

const { services: rawServices } = useSystemInfo();

const services = computed(() => {
  const byType = new Map<string, Set<string>>();
  for (const service of rawServices.value) {
    const versions = byType.get(service.serviceType) ?? new Set<string>();
    versions.add(service.version);
    byType.set(service.serviceType, versions);
  }

  return Array.from(byType.entries())
    .map(([serviceType, versions]) => ({
      serviceType,
      count: rawServices.value.filter((service) => service.serviceType === serviceType).length,
      versions: Array.from(versions).join(', '),
    }))
    .sort((a, b) => a.serviceType.localeCompare(b.serviceType));
});
</script>
