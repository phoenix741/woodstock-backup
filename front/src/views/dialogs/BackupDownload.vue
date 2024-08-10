<template>
  <v-dialog v-model="isDialogActive" max-width="800">
    <template v-slot:default="{ isActive }">
      <v-card rounded="lg">
        <v-card-title class="d-flex justify-space-between align-center">
          <div class="text-h5 text-medium-emphasis ps-2">Download agent</div>

          <v-btn icon="mdi-close" variant="text" @click="isActive.value = false"></v-btn>
        </v-card-title>

        <v-divider class="mb-4"></v-divider>

        <v-card-text v-if="client == ClientType.Windows">
          <AgentWindowsMD></AgentWindowsMD>
        </v-card-text>
        <v-card-text v-else-if="client == ClientType.Linux || client == ClientType.LinuxLite">
          <v-alert
            v-if="client == ClientType.LinuxLite"
            text="The Lite agent does not handle reading ACLs (Access Control Lists) or XATTRs (Extended Attributes). The advantage of the Lite agent is that it has fewer dependencies on the system, making it easier to install and run in environments with limited resources or permissions."
            title="Agent lite"
            type="info"
            variant="tonal"
          ></v-alert>
          <AgentLinuxMD></AgentLinuxMD>
        </v-card-text>
        <v-card-text v-else-if="client == ClientType.LinuxDeb">
          <AgentLinuxDebianMD></AgentLinuxDebianMD>
        </v-card-text>
        <v-card-text v-else-if="client == ClientType.None">
          <AgentNone></AgentNone>
        </v-card-text>

        <v-divider class="mt-2"></v-divider>

        <v-card-actions class="my-2 d-flex justify-end">
          <v-btn class="text-none" rounded="xl" text="Cancel" @click="isActive.value = false"></v-btn>

          <v-btn
            class="text-none"
            color="primary"
            rounded="xl"
            text="Download"
            variant="flat"
            @click="downloadClientAgent()"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </template>
  </v-dialog>

  <div class="pa-4">
    <v-btn color="primary" prepend-icon="mdi-monitor" append-icon="mdi-menu-down" rounded="pill" divided>
      <div class="text-none font-weight-regular">Download agent</div>

      <v-menu activator="parent" location="bottom end" transition="fade-transition">
        <v-list density="compact" min-width="250" rounded="lg" slim>
          <v-list-item
            prepend-icon="mdi-microsoft-windows"
            title="Download Windows agent"
            link
            @click="openDialog(ClientType.Windows)"
          ></v-list-item>
          <v-list-item
            prepend-icon="mdi-penguin"
            title="Download Linux agent"
            link
            @click="openDialog(ClientType.Linux)"
          ></v-list-item>
          <v-list-item
            prepend-icon="mdi-debian"
            title="Download Linux agent (DEB)"
            link
            @click="openDialog(ClientType.LinuxDeb)"
          ></v-list-item>
          <v-list-item
            prepend-icon="mdi-penguin"
            title="Download lite Linux agent"
            link
            @click="openDialog(ClientType.LinuxLite)"
          ></v-list-item>
          <v-list-item
            prepend-icon="mdi-cog-outline"
            title="Download configuration only"
            link
            @click="openDialog(ClientType.None)"
          ></v-list-item>

          <v-divider class="my-2"></v-divider>

          <v-list-item min-height="24">
            <template v-slot:subtitle>
              <div class="text-caption">Download agent version {{ informations?.woodstockVersion }}</div>
            </template>
          </v-list-item>
        </v-list>
      </v-menu>
    </v-btn>
  </div>
</template>

<script lang="ts" setup>
import { ClientType } from '@/utils/client';
import { ref } from 'vue';
import { useServerInformation } from '@/utils/server';

import AgentLinuxMD from './AgentLinux.md';
import AgentLinuxDebianMD from './AgentLinuxDeb.md';
import AgentWindowsMD from './AgentWindows.md';
import AgentNone from './AgentNone.md';

// Get the prop deviceId from the parent component
const props = defineProps<{ deviceId: string }>();

// Get the version of the agent to download
const { informations } = useServerInformation();

// Get the current operating system from window.navigator
const client = ref(getDefaultClient());

// Dialog state
const isDialogActive = ref(false);

function getDefaultClient() {
  const { userAgent } = window.navigator;
  if (userAgent.indexOf('Win') !== -1) return ClientType.Windows;
  if (userAgent.indexOf('Linux') !== -1) return ClientType.Linux;
  return ClientType.Windows;
}

function openDialog(value: ClientType) {
  console.log('open', arguments);
  client.value = value;
  isDialogActive.value = true;
}

function downloadClientAgent() {
  isDialogActive.value = false;

  // Download client at /api/hosts/{name}/client
  const deviceId = props.deviceId;

  let clientType = client.value;
  if (clientType === ClientType.LinuxDeb) {
    clientType = ClientType.None;
  }

  // Fetch the agent
  fetch(`/api/hosts/${deviceId}/client?client=${client.value}`)
    .then((response) => response.blob())
    .then((blob) => {
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `agent-${client.value}.zip`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
    });
}
</script>
