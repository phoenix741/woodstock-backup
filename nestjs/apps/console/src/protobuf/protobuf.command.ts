import {
  basenameBuffer,
  compact,
  FileManifest,
  FileManifestJournalEntry,
  PoolRefCount,
  PoolUnused,
  ProtobufService,
  ProtoFileManifest,
  ProtoFileManifestJournalEntry,
  ProtoPoolRefCount,
  ProtoPoolUnused,
  ReferenceCount,
  YAML_SCHEMA,
} from '@woodstock/shared';
import { pipe } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import * as yaml from 'js-yaml';
import { Command, Console } from 'nestjs-console';
import * as ora from 'ora';
import { basename } from 'path';
import { Type } from 'protobufjs';

@Console({
  command: 'protobuf',
})
export class ProtobufCommand {
  private spinner?: ora.Ora;

  constructor(private protobufService: ProtobufService) {}

  @Command({
    command: 'readFile <path>',
    description: 'Read a file in protobuf format',
    options: [
      {
        flags: '-t, --type <type>',
        description: 'Type of the file',
        required: true,
      },
      {
        flags: '--filter-chunks <filter>',
        description: 'Filter the output',
        required: false,
      },
      {
        flags: '--filter-name <filter>',
        description: 'Filter the output',
        required: false,
      },
      {
        flags: '--readable',
        description: 'Output in readable format',
        required: false,
      },
    ],
  })
  async readProtobufFile(
    path: string,
    options: { type: string; filterChunks: string; filterName: string; readable: boolean },
  ): Promise<void> {
    let type: Type;
    switch (options.type) {
      case 'FileManifest':
        type = ProtoFileManifest;
        break;
      case 'FileManifestJournalEntry':
        type = ProtoFileManifestJournalEntry;
        break;
      case 'RefCount':
        type = ProtoPoolRefCount;
        break;
      case 'Unused':
        type = ProtoPoolUnused;
        break;
      default:
        throw new Error(`Unknown type: ${options.type}`);
    }

    const chunks = options.filterChunks && Buffer.from(options.filterChunks, 'hex');
    const name = options.filterName && Buffer.from(options.filterName);

    const file = pipe(
      this.protobufService.loadFile(path, type),
      map((m) => m.message),
      filter((m) => {
        if (!chunks && !name) {
          return true;
        }

        switch (options.type) {
          case 'FileManifest':
            if (chunks) return !!(m as FileManifest).chunks?.includes(chunks);
            if (name) return !!name.equals(basenameBuffer((m as FileManifest).path));
          case 'FileManifestJournalEntry':
            if (chunks) return !!(m as FileManifestJournalEntry).manifest?.chunks?.includes(chunks);
            if (name) return !!name.equals(basenameBuffer((m as FileManifestJournalEntry).manifest?.path));
          case 'RefCount':
            if (chunks) return (m as PoolRefCount).sha256.equals(chunks);
          case 'Unused':
            if (chunks) return (m as PoolUnused).sha256.equals(chunks);
        }
        return false;
      }),
      map((m) => {
        if (!options.readable) {
          return m;
        }

        switch (options.type) {
          case 'FileManifest':
            const fileManifest = m as FileManifest;
            return {
              ...fileManifest,
              path: fileManifest.path.toString(),
              sha256: fileManifest.sha256?.toString('hex'),
              chunks: fileManifest.chunks?.map((c) => c.toString('hex')),
            };
          case 'FileManifestJournalEntry':
            const fileManifestJournalEntry = m as FileManifestJournalEntry;
            return {
              ...fileManifestJournalEntry,
              manifest: {
                ...fileManifestJournalEntry.manifest,
                path: fileManifestJournalEntry.manifest?.path.toString(),
                sha256: fileManifestJournalEntry.manifest?.sha256?.toString('hex'),
                chunks: fileManifestJournalEntry.manifest?.chunks?.map((c) => c.toString('hex')),
              },
            };
          case 'RefCount':
            const poolRefCount = m as PoolRefCount;
            return {
              ...poolRefCount,
              sha256: poolRefCount.sha256?.toString('hex'),
            };
          case 'Unused':
            const poolUnused = m as PoolUnused;
            return {
              ...poolUnused,
              sha256: poolUnused.sha256?.toString('hex'),
            };
        }
      }),
    );

    for await (const obj of file) {
      const str = yaml.dump(compact([obj]), { schema: YAML_SCHEMA });
      process.stdout.write(str);
    }
  }
}
