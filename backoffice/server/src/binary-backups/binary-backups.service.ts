import { Logger } from '@nestjs/common';
import { ClientProxyFactory, Transport } from '@nestjs/microservices';
import {
  CHUNK_SIZE,
  EntryType,
  FileChunk,
  FileManifest,
  GetChunkRequest,
  LaunchBackupHeader,
  LaunchBackupRequest,
  longMin,
  Manifest,
  ManifestService,
  StatusCode,
  WoodstockClientService,
  silence,
} from '@woodstock/shared';
import { promises } from 'fs';
import { credentials } from 'grpc';
import * as Long from 'long';
import { join } from 'path';
import { concat, from, Observable, of, Subject, throwError } from 'rxjs';
import { catchError, endWith, map, mergeMap, startWith, takeWhile, reduce, tap } from 'rxjs/operators';
import { ApplicationConfigService } from 'src/config/application-config.service';
import { Readable } from 'stream';

import { PoolService } from '../storage/pool/pool.service';
import wrapObservable from '../utils/wrap-observable';
import { BackupConfiguration, Task } from './models/backups-configuration.model';
import { BackupsStats } from './models/backups-stats.model';

/**
 * The goal of this class is to contact the client to backup it.
 *
 * This service use an abstraction that can be gRPC or Http2 to backup.
 */
export class BinaryBackupsService {
  private logger = new Logger(BinaryBackupsService.name);

  private statistics?: BackupsStats;

  constructor(
    private manifestService: ManifestService,
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

    const channel_creds = credentials.createSsl(await promises.readFile('../../client/client-sync/certs/server.crt'));

    const client = ClientProxyFactory.create({
      transport: Transport.GRPC,
      options: {
        package: 'woodstock',
        protoPath: join(__dirname, '..', 'woodstock.proto'),
        url: this.hostToBackup + ':3657',
        credentials: channel_creds,
      },
    });

    this.logger.log('Connect to the client');

    // Create the connection with the client
    const woodstockClientService = client.getService<WoodstockClientService>('WoodstockClientService');

    const { tasks, finalizedTasks } = configuration.operations;
    try {
      for (const task of tasks || []) {
        await this.processTask(woodstockClientService, task);
      }
    } finally {
      for (const task of finalizedTasks || []) {
        await this.processTask(woodstockClientService, task);
      }
    }
  }

  private async processTask(woodstockClientService: WoodstockClientService, task: Task): Promise<void> {
    const { command, shares, includes, excludes } = task;
    if (command) {
      const reply = await woodstockClientService
        .executeCommand({
          command,
        })
        .toPromise();

      this.logger.log(reply.stdout);
      if (reply.code) {
        this.logger.error(reply.stderr);
      }
    }

    for (const share of shares || []) {
      const { name, includes: shareIncludes, excludes: shareExcludes } = share;

      await this.createBackup(woodstockClientService, {
        sharePath: Buffer.from(name),
        includes: [...(includes || []), ...(shareIncludes || [])].map((s) => Buffer.from(s)),
        excludes: [...(excludes || []), ...(shareExcludes || [])].map((s) => Buffer.from(s)),
        lastBackupNumber: -1,
        newBackupNumber: 0,
      }).toPromise();
    }
  }

  private createBackup(
    woodstockClientService: WoodstockClientService,
    header: LaunchBackupHeader,
  ): Observable<boolean> {
    const manifest = new Manifest(`backups.${this.hostToBackup}.${this.currentBackupId}`, this.configService.hostPath);

    const fileManifestEntries$ = new Subject<LaunchBackupRequest>();
    const launchBackupRequest$ = concat<LaunchBackupRequest>(
      fileManifestEntries$.pipe(
        startWith({ header }),
        endWith({
          footer: {
            code: StatusCode.Ok,
          },
        }),
        catchError((err) => {
          return of({
            footer: {
              code: StatusCode.Failed,
              message: err.message,
            },
          });
        }),
      ),
    );

    const launchBackup$ = woodstockClientService.launchBackup(launchBackupRequest$).pipe(
      takeWhile(({ response }) => !response?.diskReadFinished),
      tap(({ response }) => {
        if (response && response.code === StatusCode.Failed) {
          throw { needRefreshCache: response.needRefreshCache };
        }
      }),
      mergeMap(({ entry, response }) => {
        if (entry && entry.type !== EntryType.REMOVE) {
          return from(this.copyManifest(woodstockClientService, entry.manifest)).pipe(
            map((manifest) => ({
              response,
              entry: {
                type: entry.type,
                manifest,
              },
            })),
          );
        } else {
          return of({ entry, response });
        }
      }), // FIXME: Merge with concurency
      this.manifestService.writeJournalEntry(
        () => manifest,
        ({ entry }) => entry,
      ),
    );

    const compact$ = this.manifestService.compact(manifest); // FIXME: REFCOUNT

    return concat(launchBackup$, compact$).pipe(
      reduce(() => false, false),
      catchError((err) => {
        if (err && err.needRefreshCache) {
          return of(true);
        }
        return throwError(err);
      }),
    );
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

    const position = Long.fromValue(CHUNK_SIZE).mul(chunkNumber);
    const chunk: GetChunkRequest = {
      filename: fileManifest.path,
      position,
      size: longMin(
        Long.fromValue(CHUNK_SIZE),
        (fileManifest.stats?.size || Long.ZERO).sub(CHUNK_SIZE).mul(chunkNumber),
      ),
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
