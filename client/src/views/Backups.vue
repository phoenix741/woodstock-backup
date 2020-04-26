<template>
  <v-container>
    <v-card>
      <v-data-table
        show-select
        :headers="headers"
        :items="desserts"
        :items-per-page="15"
      >
        <template slot="item.show" scope="props">
          <td class="align-center">
            <v-btn
              class="secondary"
              rounded
              :to="`/backups/${props.item.host}/${props.item.number}`"
            >
              Browse
            </v-btn>
            <v-btn
              class="ml-1 secondary"
              rounded
              :to="`/backups/${props.item.host}/${props.item.number}/logs`"
            >
              Show Log
            </v-btn>
          </td>
        </template>
      </v-data-table>
      <v-card-actions>
        <v-btn class="primary ml-12" text>Delete</v-btn>
        <v-btn class="primary" text>Launch backup</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";

@Component({})
export default class Hosts extends Vue {
  headers = [
    {
      text: "Backup#",
      align: "start",
      value: "number",
    },
    { text: "Start date", value: "startDate" },
    { text: "Duration", value: "duration" },
    { text: "Files Count", value: "filesCount" },
    { text: "Files Size", value: "filesSize" },
    { text: "Existing Files Count", value: "existingFileCount" },
    { text: "Existing Files Size", value: "existingFileSize" },
    { text: "New Files Count", value: "newFileCount" },
    { text: "New Files Size", value: "newFileSize" },
    { text: "", value: "show", sortable: false },
  ];
  desserts = [
    {
      host: "pc-ulrich",
      number: 1,
      startDate: new Date(),
      duration: 55,
      filesCount: 2131231,
      filesSize: 1231312,
      existingFileCount: 12313,
      existingFileSize: 12312312,
      newFileCount: 12332,
      newFileSize: 123123,
    },
  ];
}
</script>
