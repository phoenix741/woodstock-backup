<template>
  <v-card class="mx-auto" flat>
    <v-list-item two-line>
      <v-list-item-avatar tile size="80">
        <v-icon x-large :color="color">{{ icon }}</v-icon>
      </v-list-item-avatar>

      <v-list-item-content>
        <div class="overline mb-4">{{ label }}</div>
        <v-list-item-title class="headline mb-1">{{ value }}</v-list-item-title>
      </v-list-item-content>
    </v-list-item>

    <v-divider></v-divider>
    <v-card-text>
      <v-icon class="mr-2" small>
        mdi-clock-outline
      </v-icon>
      <span class="caption grey--text font-weight-light"><slot></slot></span>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';

@Component
export default class QueueInformation extends Vue {
  @Prop({ required: true })
  color!: string;

  @Prop({ required: true })
  icon!: string;

  @Prop({ required: true })
  label!: string;

  @Prop({ required: true, type: Number })
  value!: number;
}
</script>
