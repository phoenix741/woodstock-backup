<script setup lang="ts">
import { RouterView } from "vue-router";
import { useClient } from "villus";
import MenuComponent from "./components/MenuComponent.vue";

useClient({
  url: "http://" + location.host + "/graphql",
});
</script>

<template>
  <v-app>
    <v-app-bar color="primary">
      <v-toolbar-title>Woodstock backup</v-toolbar-title>

      <v-spacer></v-spacer>

      <template v-slot:extension>
        <MenuComponent></MenuComponent>
      </template>
    </v-app-bar>
    <v-main>
      <RouterView />
    </v-main>
  </v-app>
</template>
