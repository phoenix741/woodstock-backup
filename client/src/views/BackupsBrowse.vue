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
      item-key="path"
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
    </v-treeview>
    <v-bottom-sheet v-model="sheet">
      <v-sheet class="text-center" v-if="selected">
        <v-simple-table>
          <template v-slot:default>
            <tbody>
              <tr>
                <td>Path</td>
                <td>{{ selected.path }}</td>
              </tr>
              <tr>
                <td>Type</td>
                <td>{{ selected.type }}</td>
              </tr>
              <tr>
                <td>Owner</td>
                <td>{{ selected.uid }}</td>
              </tr>
              <tr>
                <td>Group</td>
                <td>{{ selected.gid }}</td>
              </tr>
              <tr>
                <td>Mode</td>
                <td>{{ (selected.mode & 0o7777).toString(8) }}</td>
              </tr>
              <tr>
                <td>Size</td>
                <td>{{ selected.size | filesize }}</td>
              </tr>
              <tr>
                <td>Modification Time</td>
                <td>{{ selected.mtime | date }}</td>
              </tr>
            </tbody>
          </template>
        </v-simple-table>
        <v-btn class="mt-6" text color="red" @click="sheet = !sheet">close</v-btn>
        <v-btn
          class="mt-6"
          text
          color="primary"
          :href="'/api/hosts/' + hostname + '/backups/' + number + '/files/download?path=' + selected.path"
          >download</v-btn
        >
      </v-sheet>
    </v-bottom-sheet>
  </div>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import backupsBrowse from './BackupsBrowse.graphql';
import { BackupsBrowseQuery } from '../generated/graphql';

interface TreeItem {
  name: string;
  type: string;
  uid: number;
  gid: number;
  mode: number;
  size: number;
  mtime: number;

  file?: string;
  path: string;
  children?: TreeItem[];
}

function mapToItems(path: string, { backup }: BackupsBrowseQuery): TreeItem[] {
  return backup.files.map(b => {
    const object: TreeItem = {
      ...b,
      path: `${path}/${b.name}`,
    };
    if (b.type !== 'DIRECTORY') {
      object.file = b.name.split('.').pop();
    } else {
      object.children = [];
    }
    return object;
  });
}

@Component({
  apollo: {
    items: {
      query: backupsBrowse,
      variables() {
        return {
          hostname: this.hostname,
          number: parseInt(this.number),
          path: '/',
        };
      },
      update: (query: BackupsBrowseQuery) => mapToItems('', query),
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
        path: item.path,
      },
    });

    const items = mapToItems(item.path, data);
    item.children.push(...items);
  }

  openTreeItem(item: TreeItem[]) {
    if (!item?.length) return;
    this.selected = item[0];
    this.sheet = true;
  }
}
</script>
