import { HttpService, Logger } from '@nestjs/common';
import { ClientProxyFactory, Transport } from '@nestjs/microservices';
import { promises } from 'fs';
import { credentials } from 'grpc';
import { join } from 'path';
import { concat, from, Observable, of } from 'rxjs';
import { filter, map, mergeMap, reduce, switchMap, concatMap } from 'rxjs/operators';
import { ApplicationConfigService, CHUNK_SIZE } from 'src/config/application-config.service';
import { EntryType } from 'src/storage/backup-manifest/manifest.model';

import { Manifest } from '../storage/backup-manifest/manifest';
import { FileManifest } from '../storage/backup-manifest/manifest.model';
import { PoolService } from '../storage/pool/pool.service';
import wrapObservable from '../utils/wrap-observable';
import { BackupConfiguration } from './models/backups-configuration.model';
import { BackupsStats } from './models/backups-stats.model';
import { FileChunk, GetChunkRequest, WoodstockClientService } from './models/woodstock-client.service';
import { Readable } from 'stream';

/**
 * The goal of this class is to contact the client to backup it.
 *
 * This service use an abstraction that can be gRPC or Http2 to backup.
 */
export class BinaryBackupsService {
  private logger = new Logger(BinaryBackupsService.name);

  private statistics?: BackupsStats;

  constructor(
    private httpService: HttpService,
    private configService: ApplicationConfigService,
    private poolService: PoolService,
    private hostToBackup: string,
    private lastBackupId: number,
    private currentBackupId: number,
  ) {}

  getConfiguration(): BackupConfiguration {
    return {
      operations: {
        tasks: [
          {
            command: '/usr/bin/which which',
          },
          {
            shares: [
              {
                name: '/home',
                excludes: [
                  'phoenix/tensorflow',
                  'phoenix/tmp',
                  'phoenix/.composer',
                  '*node_modules',
                  '*mongodb/db',
                  'phoenix/.ccache',
                  '*mongodb/dump',
                  'phoenix/usr/android-sdk',
                  'phoenix/.cache',
                  'phoenix/.CloudStation',
                  'phoenix/.android',
                  'phoenix/.AndroidStudio*',
                  'phoenix/usr/android-studio',
                  '*.vmdk',
                  'phoenix/.nvm',
                  '*.vdi',
                  'phoenix/.local/share/Trash',
                  'phoenix/VirtualBox VMs',
                  '*mongodb/configdb',
                  'phoenix/.thumbnails',
                  'phoenix/.VirtualBox',
                  'phoenix/.vagrant.d',
                  'phoenix/vagrant',
                  'phoenix/.npm',
                  'phoenix/Pictures',
                  'phoenix/Documents synhronisés',
                  'phoenix/dwhelper',
                  'phoenix/snap',
                  'phoenix/.local/share/flatpak',
                  'phoenix/usr/AndroidSdk',
                  'public/kg/gallery',
                  '*vcpkg',
                ],
              },
              {
                name: '/etc',
              },
            ],
          },
        ],
        finalizedTasks: [
          {
            command: '/usr/bin/which which',
          },
        ],
      },
    };
  }

  async start(): Promise<void> {
    this.statistics = new BackupsStats();

    // Get the configuration for the client
    const configuration = this.getConfiguration();

    const channel_creds = credentials.createSsl(await promises.readFile('../client-sync/certs/server.crt'));

    const client = ClientProxyFactory.create({
      transport: Transport.GRPC,
      options: {
        package: 'woodstock',
        protoPath: join(__dirname, '..', 'storage', 'backup-manifest', 'woodstock.proto'),
        url: this.hostToBackup + ':3657',
        credentials: channel_creds,
      },
    });

    this.logger.log('Connect to the client');

    // Create the connection with the client
    const woodstockClientService = client.getService<WoodstockClientService>('WoodstockClientService');

    //const woodstockClientService = new Http2WoodstockClientService(this.httpService, this.hostToBackup);

    this.logger.log('Prepare the backup and send configuration');
    const prepareResult = await woodstockClientService
      .prepareBackup({ configuration, lastBackupNumber: this.lastBackupId, newBackupNumber: this.currentBackupId })
      .toPromise();
    this.logger.log('The backup need to refresh ' + JSON.stringify(prepareResult));

    if (prepareResult.needRefreshCache && this.lastBackupId) {
    }

    const manifest = new Manifest(`backups.${this.hostToBackup}.${this.currentBackupId}`, this.configService.hostPath);

    const loadIndex$ = manifest.loadIndex();

    await new Promise<void>((resolve, reject) => {
      const createIndex$ = loadIndex$.pipe(
        switchMap((index) =>
          woodstockClientService.launchBackup({ backupNumber: this.currentBackupId }).pipe(
            concatMap((entry) => {
              if (entry.type !== EntryType.REMOVE && entry.type !== EntryType.CLOSE) {
                return from(this.copyManifest(woodstockClientService, entry.manifest)).pipe(
                  map((manifest) => ({
                    type: entry.type,
                    manifest,
                  })),
                );
              } else {
                return of(entry);
              }
            }),
            manifest.writeJournalEntry(),
            reduce((index, journalEntry) => {
              if (journalEntry.type !== EntryType.CLOSE) {
                const path = journalEntry.type === EntryType.REMOVE ? journalEntry.path : journalEntry.manifest.path;
                if (path) {
                  const entry = index.getEntry(path);
                  if (entry) {
                    index.mark(entry);
                  }
                }
              }
              return index;
            }, index),
          ),
        ),
      );

      const addRemoveToIndex$ = createIndex$.pipe(
        switchMap((index) =>
          index.walk().pipe(
            filter((entry) => !entry.markViewed),
            map((file) => Manifest.toRemoveJournalEntry(file.path)),
            manifest.writeJournalEntry(),
          ),
        ),
      );

      const compact$ = manifest.compact();

      const subscription$ = concat(addRemoveToIndex$, compact$).subscribe({
        complete: () => {
          this.logger.log(`The manifest have been compacted`);
          subscription$.unsubscribe();
          resolve();
        },
        error: (err) => {
          this.logger.error(`Can't create the manifest: ${err.message}`);
          subscription$.unsubscribe();
          reject(err);
        },
      });
    });
  }

  private async copyManifest(woodstockClientService: WoodstockClientService, fileManifest: FileManifest) {
    const chunks = fileManifest.chunks || [];
    for (let index = 0; index < chunks.length; index++) {
      await this.copyChunk(woodstockClientService, fileManifest, index);
    }
    return fileManifest;
  }

  private async copyChunk(
    woodstockClientService: WoodstockClientService,
    fileManifest: FileManifest,
    chunkNumber: number,
  ): Promise<FileManifest> {
    if (!fileManifest.chunks) {
      return fileManifest;
    }

    const sha256 = fileManifest.chunks[chunkNumber];
    const wrapper = this.poolService.getChunk(sha256);

    const position = chunkNumber * CHUNK_SIZE;
    const chunk: GetChunkRequest = {
      filename: fileManifest.path,
      position,
      size: Math.min(CHUNK_SIZE, (fileManifest.stats?.size || 0) - CHUNK_SIZE * chunkNumber),
      sha256,
    };

    if (!(await wrapper.exists())) {
      const chunkResult = woodstockClientService.getChunk(chunk);
      try {
        const newChunk = await wrapper.write(Readable.from(chunkToStream(chunkResult)));
        if (!newChunk.equals(sha256)) {
          fileManifest.chunks[chunkNumber] = newChunk;
        }
      } catch (err) {
        this.logger.error(`${fileManifest.path.toString()}:${chunkNumber}`);
      }

      /*
      const chunkResult = await woodstockClientService.getChunk(chunk).toPromise();
      try {
        const newChunk = await wrapper.write(chunkResult);
        if (!newChunk.equals(sha256)) {
          fileManifest.chunks[chunkNumber] = newChunk;
        }
      } catch (err) {
        this.logger.error(`${fileManifest.path.toString()}:${chunkNumber}`);
      }
      */
    }

    return fileManifest;
  }
}

async function* chunkToStream(chunkResult: Observable<FileChunk>) {
  for await (const value of wrapObservable<FileChunk>(chunkResult)) {
    if (value) {
      yield value.data;
    }
  }
}
