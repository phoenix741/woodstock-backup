<template>
  <v-app id="inspire">
    <v-navigation-drawer v-model="drawer" app>
      <v-list>
        <v-list-item>
          <v-list-item-content>
            <v-list-item-title class="title text-center"
              >Woodstock Admin</v-list-item-title
            >
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
        <v-list-item link to="/tasks">
          <v-list-item-action>
            <v-icon>mdi-animation</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Running Tasks</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        <v-list-item link to="/reports">
          <v-list-item-action>
            <v-icon>mdi-file-chart</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Reports</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        <v-list-item link to="/logs">
          <v-list-item-action>
            <v-icon>mdi-file-document-outline</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Logs</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-app-bar app color="primary" dark>
      <v-app-bar-nav-icon @click.stop="drawer = !drawer" />
      <v-toolbar-title>Woodstock Backup</v-toolbar-title>

      <v-spacer></v-spacer>

      <v-menu left bottom>
        <template v-slot:activator="{ on }">
          <v-btn icon v-on="on">
            <v-badge color="red" content="6" overlap>
              <v-icon>mdi-animation</v-icon>
            </v-badge>
          </v-btn>
        </template>

        <v-card>
          <v-list>
            <v-list-item>
              <v-list-item-content>
                <v-list-item-title>You have 17 tasks running</v-list-item-title>
              </v-list-item-content>
              <v-list-item-action>
                <v-chip color="primary">
                  View all
                </v-chip>
              </v-list-item-action>
            </v-list-item>
          </v-list>

          <v-divider></v-divider>

          <v-list>
            <v-list-item>
              <v-list-item-avatar>
                <v-progress-circular
                  :value="20"
                  width="3"
                  color="primary"
                ></v-progress-circular>
              </v-list-item-avatar>

              <v-list-item-content>
                <v-list-item-title>pc-ulrich</v-list-item-title>
                <v-list-item-subtitle>5477 files</v-list-item-subtitle>
              </v-list-item-content>
            </v-list-item>

            <v-list-item>
              <v-list-item-avatar>
                <v-progress-circular
                  :value="50"
                  width="3"
                  color="primary"
                ></v-progress-circular>
              </v-list-item-avatar>

              <v-list-item-content>
                <v-list-item-title>pc-eve</v-list-item-title>
                <v-list-item-subtitle>23589 files</v-list-item-subtitle>
              </v-list-item-content>
            </v-list-item>
          </v-list>
        </v-card>
      </v-menu>
    </v-app-bar>

    <v-content>
      <router-view></router-view>
    </v-content>
  </v-app>
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";

@Component({})
export default class App extends Vue {
  drawer = false;
}
</script>

<style scoped>
.v-app-bar {
  background: linear-gradient(88deg, #318be3, #04befe);
}

.v-content {
  background: rgb(246, 248, 250);
}
</style>
