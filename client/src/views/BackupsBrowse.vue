<template>
  <v-treeview
    v-model="tree"
    :items="items"
    :open="open"
    activatable
    item-key="name"
    open-on-click
    :load-children="fetchPaths"
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
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import backupsBrowse from './BackupsBrowse.graphql';
import { BackupsBrowseQuery } from '../generated/graphql';

interface TreeItem {
  name: string;
  type: string;
  path: string;
  file?: string;
  children?: TreeItem[];
}

function mapToItems(path: string, { backup }: BackupsBrowseQuery): TreeItem[] {
  return backup.files.map(b => {
    const object: TreeItem = {
      name: b.name,
      type: b.type,
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
      update: (query: BackupsBrowseQuery) => mapToItems('/', query),
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

  items = [];
  tree = [];
  open = [];

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

  /*
  items = [
    {
      name: '.git',
    },
    {
      name: 'node_modules',
    },
    {
      name: 'public',
      children: [
        {
          name: 'static',
          children: [
            {
              name: 'logo.png',
              file: 'png',
            },
          ],
        },
        {
          name: 'favicon.ico',
          file: 'png',
        },
        {
          name: 'index.html',
          file: 'html',
        },
      ],
    },
    {
      name: '.gitignore',
      file: 'txt',
    },
    {
      name: 'babel.config.js',
      file: 'js',
    },
    {
      name: 'package.json',
      file: 'json',
    },
    {
      name: 'README.md',
      file: 'md',
    },
    {
      name: 'vue.config.js',
      file: 'js',
    },
    {
      name: 'yarn.lock',
      file: 'txt',
    },
  ];
  */
}
</script>
