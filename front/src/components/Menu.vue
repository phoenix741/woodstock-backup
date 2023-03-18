<template>
  <v-app-bar flat>
    <v-container class="fill-height d-flex align-center">
      <v-btn variant="text" to="/devices">Devices</v-btn>
      <v-btn variant="text" to="/tasks/active">Tasks</v-btn>
      <v-btn variant="text" to="/pool">Pool</v-btn>

      <v-spacer></v-spacer>

      <v-btn variant="text" to="/about">About</v-btn>
    </v-container>
  </v-app-bar>
</template>

<script lang="ts" setup>
//
</script>
