<template>
  <v-app-bar>
    <v-app-bar-title>
      <v-btn variant="text" to="/devices">Devices</v-btn>
      <v-btn variant="text" to="/tasks/active">Tasks</v-btn>
      <v-btn variant="text" to="/pool">Pool</v-btn>
      <v-btn variant="text" to="/events">Events</v-btn>
    </v-app-bar-title>
    <v-btn variant="text" to="/about">About</v-btn>
    <v-btn icon @click="toggleTheme">
      <v-icon :color="isDarkTheme ? 'primary' : 'primary lighten-4'">
        {{ isDarkTheme ? 'mdi-weather-night' : 'mdi-weather-sunny' }}
      </v-icon>
    </v-btn>
  </v-app-bar>
</template>

<script lang="ts" setup>
import { useTheme } from 'vuetify';
import { computed, onMounted } from 'vue';

const theme = useTheme();
const isDarkTheme = computed(() => theme.global.current.value.dark);

// Initialise le thème en fonction de la préférence du système
onMounted(() => {
  // Vérifie si l'utilisateur a déjà une préférence stockée
  const storedTheme = localStorage.getItem('theme');

  if (storedTheme) {
    theme.global.name.value = storedTheme;
  } else {
    // Sinon, utiliser la préférence du système
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    theme.global.name.value = prefersDark ? 'dark' : 'light';
  }
});

function toggleTheme() {
  const newTheme = theme.global.current.value.dark ? 'light' : 'dark';
  theme.global.name.value = newTheme;

  // Sauvegarder la préférence de l'utilisateur
  localStorage.setItem('theme', newTheme);
}
</script>
