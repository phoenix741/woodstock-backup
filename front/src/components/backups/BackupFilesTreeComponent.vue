<template>
  <v-skeleton-loader :loading="isFetching" type="list-item-two-line">
    <v-treeview
      v-model:activated="active"
      v-model:opened="open"
      :items="items"
      :load-children="fetchNodes"
      color="warning"
      density="compact"
      slim
      item-title="displayName"
      item-value="id"
      item-props="props"
      activatable
      transition
      :style="cssVars"
      @click:select="onSelect($event)"
    >
      <template v-slot:prepend="{ item, isOpen }">
        <v-icon v-if="item.node.type == EnumFileType.Directory">
          {{ isOpen ? 'mdi-folder-open' : 'mdi-folder' }}
        </v-icon>
        <v-icon v-else>
          {{ FILE_TYPE_MAPPING[item.node.type] }}
        </v-icon>
      </template>
    </v-treeview>
  </v-skeleton-loader>
</template>

<script setup lang="ts">
import { useBackupsBrowse } from '../../utils/backups';
import { VTreeview } from 'vuetify/labs/VTreeview';
import { computed, ref, watch } from 'vue';
import { unmangle } from '../../utils/file';
import { useApolloClient } from '@vue/apollo-composable';
import { TreeViewNode } from './backups.interface';
import { EnumFileType, FragmentFileDescriptionFragment } from '@/generated/graphql';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
  hiddenFiles?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', node: TreeViewNode): void;
}>();

const active = ref([]);
const open = ref([]);
const items = ref<TreeViewNode[]>([]);

const { client } = useApolloClient();
const { shares, isFetching, browse } = useBackupsBrowse(props.deviceId, props.backupNumber);

const FILE_TYPE_MAPPING = {
  [EnumFileType.RegularFile]: 'mdi-file',
  [EnumFileType.BlockDevice]: 'mdi-harddisk',
  [EnumFileType.CharacterDevice]: 'mdi-console',
  [EnumFileType.Directory]: 'mdi-folder',
  [EnumFileType.Fifo]: 'mdi-pipe',
  [EnumFileType.Socket]: 'mdi-power-socket-eu',
  [EnumFileType.Symlink]: 'mdi-link',
  [EnumFileType.Unknown]: 'mdi-file-question',
};

const cssVars = computed(() => {
  return {
    '--v-treeview-item-hidden-display': props.hiddenFiles ? 'grid' : 'none',
  };
});

watch(shares, (value) => {
  let sharesNode = value?.map((item) => mapShareNode(item));

  items.value = sharesNode ?? [];
});

function mapShareNode(node: FragmentFileDescriptionFragment): TreeViewNode {
  return {
    id: node.path,
    sharePath: unmangle(node.path),
    path: [],
    displayName: unmangle(node.path),
    children: [],
    node,
    props: { 'data-is-hidden': false },
  };
}

function mapFileNode(node: FragmentFileDescriptionFragment, parent: TreeViewNode): TreeViewNode {
  const isHidden = node.path.startsWith('.');
  const path = [...parent.path, node.path];
  return {
    id: encodeId([parent.sharePath, ...path].join('-')),
    sharePath: parent.sharePath,
    path,
    displayName: unmangle(node.path),
    children: node.type === EnumFileType.Directory ? [] : undefined,
    node,
    props: { 'data-is-hidden': isHidden },
  };
}

function isTreeNode(node: unknown): node is TreeViewNode {
  return (node as TreeViewNode).displayName !== undefined;
}

async function fetchNodes(node: unknown) {
  if (!isTreeNode(node)) {
    return;
  }

  const files = await browse(client, node.sharePath, node.path.join('/'));
  node.children?.push(...files.map((file) => mapFileNode(file, node)));
}

function encodeId(id: string) {
  return id.replace(/[^a-zA-Z0-9_-]/g, '-');
}

function onSelect(node: unknown) {
  if (!isTreeNode(node)) {
    return;
  }

  emit('select', node);
}
</script>

<style>
.v-treeview-item[data-is-hidden='true'] {
  display: var(--v-treeview-item-hidden-display, none);
}
</style>
