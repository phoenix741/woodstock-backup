<template>
  <v-card class="mx-auto" flat>
    <v-list-item two-line>
      <v-list-item-avatar tile size="80">
        <v-icon x-large color="blue">mdi-harddisk</v-icon>
      </v-list-item-avatar>

      <v-list-item-content>
        <div class="overline mb-4">Used space</div>
        <v-list-item-title>{{ used | filesize }} / {{ total | filesize }}</v-list-item-title>
        <v-list-item-subtitle>
          <v-progress-linear :value="(100 * used) / total"></v-progress-linear>
        </v-list-item-subtitle>
      </v-list-item-content>
    </v-list-item>

    <v-divider></v-divider>
    <v-card-text>
      <v-icon class="mr-2" small> mdi-folder-outline </v-icon>
      <span class="caption grey--text font-weight-light"> Shared space: {{ shared | filesize }}</span>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';

@Component({})
export default class SpaceUsageCard extends Vue {
  @Prop({ required: true, type: Number })
  used!: number;

  @Prop({ required: true, type: Number })
  total!: number;

  @Prop({ required: true, type: Number })
  shared!: number;
}
</script>
