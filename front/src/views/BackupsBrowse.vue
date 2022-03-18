<template>
  <div class="fill-height">
    <v-container class="fill-height" fluid v-if="$apollo.queries.items.loading">
      <v-row align="center" justify="center">
        <v-col class="text-center">
          <v-progress-circular :size="50" color="primary" indeterminate></v-progress-circular>
        </v-col>
      </v-row>
    </v-container>
    <v-treeview
      v-if="!$apollo.queries.items.loading"
      :items="items"
      activatable
      @update:active="openTreeItem($event)"
      item-key="fullPath"
      :load-children="fetchPaths"
      return-object
    >
      <template v-slot:prepend="{ item, open }">
        <v-icon v-if="!item.file">
          {{ open ? 'mdi-folder-open' : 'mdi-folder' }}
        </v-icon>
        <v-icon v-else>
          {{ item.file | toIcon }}
        </v-icon>
      </template>
      <template v-slot:label="{ item }">
        {{ item.namePath | unmangle }}
      </template>
    </v-treeview>
    <v-bottom-sheet v-model="sheet">
      <v-sheet class="text-center" v-if="selected">
        <v-simple-table>
          <template v-slot:default>
            <tbody>
              <tr>
                <td>Path</td>
                <td>{{ selected.fullPath | unmangle }}</td>
              </tr>
              <tr>
                <td>Type</td>
                <td>{{ selected.type }}</td>
              </tr>
              <tr v-if="selected.symlink">
                <td>Symlink</td>
                <td>{{ selected.symlink | unmangle }}</td>
              </tr>
              <tr>
                <td>Owner</td>
                <td>{{ selected.stats.ownerId }}</td>
              </tr>
              <tr>
                <td>Group</td>
                <td>{{ selected.stats.groupId }}</td>
              </tr>
              <tr>
                <td>Mode</td>
                <td>{{ (parseInt(selected.stats.mode || '0') & 0o7777).toString(8) }}</td>
              </tr>
              <tr>
                <td>Size</td>
                <td>{{ parseInt(selected.stats.size || '0') | filesize }}</td>
              </tr>
              <tr>
                <td>Modification Time</td>
                <td>{{ parseInt(selected.stats.lastModified || '0') | date }}</td>
              </tr>
            </tbody>
          </template>
        </v-simple-table>
        <v-btn class="mt-6" text color="red" @click="sheet = !sheet">close</v-btn>
        <v-btn
          class="mt-6"
          text
          color="primary"
          :href="
            '/api/hosts/' +
            hostname +
            '/backups/' +
            number +
            '/files/download?sharePath=' +
            selected.sharePath +
            '&path=' +
            selected.searchPath
          "
          >download</v-btn
        >
      </v-sheet>
    </v-bottom-sheet>
  </div>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import backupsBrowse from './BackupsBrowse.graphql';
import shareBrowse from './ShareBrowse.graphql';
import { BackupsBrowseQuery, FragmentFileDescriptionFragment, SharesBrowseQuery } from '@/generated/graphql';

interface TreeItem extends FragmentFileDescriptionFragment {
  file?: string;
  path: string;
  sharePath?: string;
  fullPath: string;
  namePath?: string;
  searchPath?: string;
  children?: TreeItem[];
}

function mapShareToItems(query: SharesBrowseQuery): TreeItem[] {
  const files = query.backup.shares;
  return files.map((b) => ({
    ...b,
    namePath: b.path,
    searchPath: '',
    sharePath: b.path,
    fullPath: b.path,
    children: [],
  }));
}

function mapBackupToItems(currentItem: TreeItem, query: BackupsBrowseQuery): TreeItem[] {
  const files = query.backup.files;
  return files.map((b) => {
    const object: TreeItem = {
      ...b,
      namePath: b.path.split('%2F').pop(),
      searchPath: b.path,
      sharePath: currentItem.sharePath,
      fullPath: [currentItem.sharePath, b.path].join('%2F'),
    };

    if (!['DIRECTORY', 'SHARE'].includes(b.type)) {
      object.file = b.path.split('.').pop();
    } else {
      object.children = [];
    }
    return object;
  });
}

@Component({
  apollo: {
    items: {
      query: shareBrowse,
      variables() {
        return {
          hostname: this.hostname,
          number: parseInt(this.number),
        };
      },
      update: (query: SharesBrowseQuery) => mapShareToItems(query),
      fetchPolicy: 'network-only',
    },
  },
})
export default class BackupBrowse extends Vue {
  @Prop({
    required: true,
  })
  hostname!: string;
  @Prop({
    required: true,
  })
  number!: string;
  selected: TreeItem = {} as TreeItem;

  items: TreeItem[] = [];
  sheet = false;

  async fetchPaths(item: TreeItem) {
    if (!item.children) {
      return;
    }

    const { data } = await this.$apollo.query<BackupsBrowseQuery>({
      query: backupsBrowse,
      variables: {
        hostname: this.hostname,
        number: parseInt(this.number),
        sharePath: item.sharePath,
        path: item.searchPath,
      },
    });

    const items = mapBackupToItems(item, data);
    item.children.push(...items);
  }

  openTreeItem(item: TreeItem[]) {
    if (!item?.length) return;
    this.selected = item[0];
    this.sheet = true;
  }
}
</script>
