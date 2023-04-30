<template>
  <v-row>
    <v-col cols="6">
      <v-card height="500px" class="overflow-auto" elevation="0">
        <v-card-text>
          <Tree v-if="!isFetching" :config="config" :nodes="nodes" @nodeOpened="loadNode" @nodeFocus="select">
            <template #before-input="props">
              <v-icon v-if="props.node.icon">{{ props.node.icon }}</v-icon>
            </template>
            <template #loading-slot>
              <v-progress-linear color="primary" indeterminate></v-progress-linear>
            </template>
          </Tree>
        </v-card-text>
      </v-card>
    </v-col>
    <v-col cols="6" v-if="selected && selected?.file">
      <v-table>
        <tbody>
          <tr>
            <td>Path</td>
            <td>{{ selected.text }}</td>
          </tr>
          <tr>
            <td>Type</td>
            <td>{{ selected.file.type }}</td>
          </tr>
          <tr v-if="selected.file.symlink">
            <td>Symlink</td>
            <td>{{ selected.file.symlink }}</td>
          </tr>
          <tr v-if="selected.file.stats?.ownerId">
            <td>Owner</td>
            <td>{{ selected.file.stats.ownerId }}</td>
          </tr>
          <tr v-if="selected.file.stats?.groupId">
            <td>Group</td>
            <td>{{ selected.file.stats.groupId }}</td>
          </tr>
          <tr v-if="selected.file.stats?.mode">
            <td>Mode</td>
            <td>{{ (parseInt(selected.file.stats.mode) & 0o7777).toString(8) }}</td>
          </tr>
          <tr v-if="selected.file.stats?.size">
            <td>Size</td>
            <td>{{ filesize(parseInt(selected.file.stats.size)) }}</td>
          </tr>
          <tr v-if="selected.file.stats?.lastModified">
            <td>Modification Time</td>
            <td>{{ toDateTime(parseInt(selected.file.stats.lastModified)) }}</td>
          </tr>
        </tbody>
        <tfoot>
          <tr>
            <td colspan="2" class="text-right">
              <v-btn color="primary" :href="selectedPath" target="_blank">Download</v-btn>
            </td>
          </tr>
        </tfoot>
      </v-table>
    </v-col>
  </v-row>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import Tree from 'vue3-treeview';
import 'vue3-treeview/dist/style.css';
import { useBackupsBrowse } from '../../utils/backups';
import { toIcon, unmangle } from '../../utils/file';
import { ref } from 'vue';
import { toDateTime } from '../hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { computed } from 'vue';
import { FragmentFileDescriptionFragment } from '@/generated/graphql';

interface Node {
  text: string;
  children: string[];
  icon?: string;
  state?: {
    isLoading?: boolean;
  };
  sharePath?: string;
  path?: string[];
  file?: FragmentFileDescriptionFragment;
}

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
}>();

const { shares, isFetching, browse } = useBackupsBrowse(props.deviceId, props.backupNumber);

const nodes = reactive({
  root: {
    text: '/',
    children: [],
  },
} as Record<string, Node>);

const selected = ref<Node | undefined>(undefined);
const selectedPath = computed(() =>
    `/api/hosts/${props.deviceId}/backups/${props.backupNumber}/files/download?sharePath=${selected.value?.sharePath}&path=${selected.value?.file?.path}`
);

watch(shares, () => {
  nodes.root.children = [];
  shares.value?.forEach((share) => {
    nodes[share.path] = {
      text: unmangle(share.path),
      sharePath: share.path,
      path: [],
      children: [],
    };
    nodes.root.children.push(share.path);
  });
  console.log(nodes);
});

const config = reactive({
  roots: ['root'],
  leaves: ['fakeid'],
  keyboardNavigation: true,
  //checkboxes: true,
  openedIcon: {
    type: 'class',
    class: 'mdi mdi-folder-open',
  },
  closedIcon: {
    type: 'class',
    class: 'mdi mdi-folder',
  },
});

const DIRECTORY = ['DIRECTORY', 'SHARE'];
const SYMBOLIC_LINK = ['SYMBOLIC_LINK'];

const loadNode = async (n: Node) => {
  const { sharePath, path } = n;

  if (n.children && n.children.length > 0) return;

  if (!sharePath) {
    console.error('no share path');
    return;
  }
    // set node loading state to tree
  n.state = n.state ?? {};
  n.state.isLoading = true;

  const files = await browse(sharePath, path?.join('/') ?? '/');

  // fake server call
  for (const file of files) {
    const fileName = unmangle(file.path).split('/').pop();
    const ext = fileName?.split('.').pop();
    const id = file.path;
    const newNode = {
      icon: DIRECTORY.includes(file.type)
        ? undefined
        : SYMBOLIC_LINK.includes(file.type)
        ? 'mdi-file-link'
        : toIcon(ext ?? ''),
      text: SYMBOLIC_LINK.includes(file.type) ? `${fileName} → ${unmangle(file.symlink ?? '')}` : fileName ?? 'no-name',
      children: [],
      state: {},
      sharePath,
      path: file.path.split('/'),

      file,
    };
    nodes[id] = newNode;
    n.children.push(id);

    if (!DIRECTORY.includes(file.type)) {
      config.leaves.push(id);
    }
  }
  n.state.isLoading = false;
};

const select = (n: Node) => {
  selected.value = n;
};
</script>

<style>
.input-wrapper {
  color: #f5f5f5;
}
</style>
