<template>
  <v-skeleton-loader :loading="isFetching" type="list-item-two-line">
    <v-treeview
      v-model:activated="active"
      v-model:opened="open"
      :items="items"
      :load-children="fetchNodes"
      :custom-key-filter="{ isHidden: filterFn }"
      :filter-keys="['isHidden']"
      filter-mode="every"
      color="warning"
      density="compact"
      item-title="displayName"
      item-value="id"
      activatable
      open-on-click
      transition
    >
    </v-treeview>
  </v-skeleton-loader>
</template>

<script setup lang="ts">
import { useBackupsBrowse } from '../../utils/backups';
import { VTreeview } from 'vuetify/labs/VTreeview';
import { ref, watch } from 'vue';
import { unmangle } from '../../utils/file';
import { useApolloClient } from '@vue/apollo-composable';
import { TreeViewNode } from './backups.interface';
import { EnumFileType, FragmentFileDescriptionFragment } from '@/generated/graphql';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
  hiddenFiles?: boolean;
}>();

const active = ref([]);
const open = ref([]);
const items = ref<TreeViewNode[]>([]);

const { client } = useApolloClient();
const { shares, isFetching, browse } = useBackupsBrowse(props.deviceId, props.backupNumber);

watch(shares, (value) => {
  let sharesNode = value?.map((item) => mapShareNode(item));

  items.value = sharesNode ?? [];
});

function mapShareNode(node: FragmentFileDescriptionFragment): TreeViewNode {
  return {
    sharePath: unmangle(node.path),
    path: [],
    displayName: unmangle(node.path),
    children: [],
    isHidden: false,
  };
}

function mapFileNode(node: FragmentFileDescriptionFragment, parent: TreeViewNode): TreeViewNode {
  const isHidden = node.path.startsWith('.');
  return {
    sharePath: parent.sharePath,
    path: [...parent.path, node.path],
    displayName: unmangle(node.path),
    children: node.type === EnumFileType.Directory ? [] : undefined,
    isHidden,
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

function filterFn(value: unknown, search: unknown, { raw }: unknown) {
  if (!isTreeNode(raw)) {
    return false;
  }
  console.log('filterFn', value, search, raw, props.hiddenFiles ? true : !raw.isHidden);

  return props.hiddenFiles ? true : !raw.isHidden;
}
</script>
