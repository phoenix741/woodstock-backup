<template>
  <v-app id="inspire">
    <v-navigation-drawer v-model="drawer" app>
      <v-list>
        <v-list-item>
          <v-list-item-content>
            <v-list-item-title class="title text-center">Woodstock Admin</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
      </v-list>

      <v-divider></v-divider>

      <v-list dense>
        <v-list-item link to="/dashboard">
          <v-list-item-action>
            <v-icon>mdi-view-dashboard</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Dashboard</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        <v-list-item link to="/hosts">
          <v-list-item-action>
            <v-icon>mdi-laptop</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Hosts</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        <NavigationBarTasks></NavigationBarTasks>
        <!--
        <v-list-item link to="/reports">
          <v-list-item-action>
            <v-icon>mdi-file-chart</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Reports</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        -->
        <NavigationBarLogs></NavigationBarLogs>
      </v-list>
    </v-navigation-drawer>

    <v-app-bar app color="primary" dark>
      <v-app-bar-nav-icon @click.stop="drawer = !drawer" />
      <v-toolbar-title>Woodstock Backup</v-toolbar-title>

      <v-spacer></v-spacer>

      <RunningTasksMenu></RunningTasksMenu>
    </v-app-bar>

    <v-main>
      <router-view></router-view>
    </v-main>
  </v-app>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import RunningTasksMenu from './components/RunningTasksMenu.vue';
import NavigationBarTasks from './components/NavigationBarTasks.vue';
import NavigationBarLogs from './components/NavigationBarLogs.vue';

@Component({
  components: {
    RunningTasksMenu,
    NavigationBarTasks,
    NavigationBarLogs,
  },
})
export default class App extends Vue {
  drawer = false;
}
</script>

<style scoped>
.v-app-bar {
  background: linear-gradient(88deg, #318be3, #04befe);
}

.v-main {
  background: rgb(246, 248, 250);
}
</style>
