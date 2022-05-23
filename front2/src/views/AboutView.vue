<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <h1 class="text-h1">About Woodstock Backup</h1>

        <p class="text-body-1">
          Woodstock Backup is a software that can be used to backup many hosts
          on a centralized server. With the help of deduplication, the storage
          is minimized.
        </p>

        <p class="text-body-1">
          Woodstock Backup is written in Javascript (NodeJS for the server and
          Vue for the client). The backup are stored in a btrfs volume with
          rsync. Woodstock Backup is open source and licensed with MIT Licence.
        </p>

        <h2 class="text-h2">BackupPC Features</h2>

        <ul>
          <li class="text-body-1">
            A modern interface with accessible API that can be used to launch
            backup,
          </li>
          <li class="text-body-1">
            Access to file can be made through the modern interface, the api, or
            the storage,
          </li>
          <li class="text-body-1">
            Backup of computer with dynamic IP or only intermittenlty connected
            to the network,
          </li>
          <li class="text-body-1">
            Open Source and freely availble under MIT.
          </li>
        </ul>
      </v-col>
    </v-row>
  </v-container>
</template>
