import { Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { Job } from 'bull';
import { constants as constantsFs, createReadStream } from 'fs';
import { readdir } from 'fs/promises';
import * as Long from 'long';
import { concat, from, Observable } from 'rxjs';
import { distinct, filter, map, mergeMap, reduce, switchMap } from 'rxjs/operators';

import { ApplicationConfigService, CHUNK_SIZE } from '../../config/application-config.service';
import { HostsService } from '../../hosts/hosts.service';
import { Manifest } from '../../storage/backup-manifest/manifest.model';
import { EntryType, FileManifest } from '../../storage/backup-manifest/object-proto.model';
import { PoolService } from '../../storage/pool/pool.service';
import { BackupTask } from '../../tasks/tasks.dto';
import { joinBuffer } from '../../utils/lodash.utils';
import { BackupsService } from '../backups.service';
import { BackupService } from './backup.service';

@Processor('queue')
export class BackupConsumer {
  private logger = new Logger(BackupConsumer.name);

  constructor(
    private configService: ApplicationConfigService,
    private hostsService: HostsService,
    private backupService: BackupService,
    private backupsService: BackupsService,
    private poolService: PoolService,
  ) {}

  @Process({ name: 'manifest' })
  async createManifestFile(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`Calculation of manifest for ${job.data.host} number ${job.data.number}`);

    if (job.data.ip) {
      await this.browseDirectory(job.data.host, job.data.number || 0, Buffer.from(job.data.ip));
    } else if (job.data.number) {
      await this.browseBackup(job.data.host, job.data.number);
    } else {
      await this.browseHost(job.data.host);
    }
  }

  async browse(): Promise<void> {
    const hosts = await this.hostsService.getHosts();
    for (const host of hosts) {
      await this.browseHost(host);
    }
  }

  async browseHost(hostname: string): Promise<void> {
    const backupNumbers = (await this.backupsService.getBackups(hostname)).map((backup) => backup.number);
    for (const number of backupNumbers) {
      await this.browseBackup(hostname, number);
    }
  }

  async browseBackup(hostname: string, backupNumber: number): Promise<void> {
    const backupPath = this.backupsService.getDestinationDirectory(hostname, backupNumber);
    this.logger.log(`Backup directory is ${backupPath} for ${hostname}/${backupNumber}`);

    await this.browseDirectory(hostname, backupNumber, Buffer.from(backupPath));
  }

  async browseDirectory(hostname: string, backupNumber: number, backupPath: Buffer): Promise<void> {
    return new Promise((resolve, reject) => {
      const manifest = new Manifest(`backups.${hostname}.${backupNumber}`, this.configService.hostPath);

      const loadIndex$ = manifest.loadIndex();

      const createIndex$ = loadIndex$.pipe(
        switchMap((index) =>
          this.walkObservable(backupPath).pipe(
            mergeMap((manifest) => from((manifest.chunks || []).map((chunk, index) => ({ chunk, manifest, index })))),
            distinct((chunk) => chunk.chunk.toString('hex')),
            mergeMap((chunk) => from(this.copyChunk(chunk.manifest, chunk.index))),
            distinct((manifest) => manifest.path.toString('hex')), // FIXME: Can be have wrong chunk in the journal
            map((manifest) => Manifest.toAddJournalEntry(manifest, true)),
            manifest.writeJournalEntry(),
            reduce((index, journalEntry) => {
              if (journalEntry.type === EntryType.CLOSE) {
                return index;
              }

              const path = journalEntry.type === EntryType.REMOVE ? journalEntry.path : journalEntry.manifest.path;
              if (path) {
                const entry = index.getEntry(path);
                if (entry) {
                  index.mark(entry);
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

  private async copyChunk(fileManifest: FileManifest, chunkNumber: number): Promise<FileManifest> {
    if (!fileManifest.chunks) {
      return fileManifest;
    }

    const chunk = fileManifest.chunks[chunkNumber];
    const wrapper = this.poolService.getChunk(chunk);
    if (!(await wrapper.exists())) {
      const start = CHUNK_SIZE.mul(chunkNumber);
      const file = createReadStream(fileManifest.path, {
        start: start.toNumber(),
        end: start.add(CHUNK_SIZE).sub(1).toNumber(),
      });

      const newChunk = await wrapper.write(file);
      if (!newChunk.equals(chunk)) {
        fileManifest.chunks[chunkNumber] = newChunk;
      }
    }

    return fileManifest;
  }

  private walkObservable(backupPath: Buffer) {
    return new Observable<FileManifest>((subscribe) => {
      const go = async () => {
        try {
          await this.walk(backupPath, async (manifest) => {
            if (((manifest?.stats?.mode || Long.ZERO).toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFREG) {
              manifest = await this.backupService.readLocalFile(manifest);
            }
            subscribe.next(manifest);
          });
          subscribe.complete();
        } catch (err) {
          subscribe.error(err);
        }
      };

      go();
    });
  }

  private async walk(backupPath: Buffer, progress: (manifest: FileManifest) => Promise<void>) {
    //this.logger.debug(`Read the directory ${backupPath}/${path}`);
    const list = await readdir(backupPath, { encoding: 'buffer' });
    for (const file of list) {
      try {
        const manifestFile = await this.backupService.createManifestFromLocalFile(joinBuffer(backupPath, file));
        await progress(manifestFile);
        if (((manifestFile?.stats?.mode || Long.ZERO).toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFDIR) {
          await this.walk(joinBuffer(backupPath, file), progress);
        }
      } catch (err) {
        this.logger.error(`Can't process the file ${backupPath}: ${err.message}`); // FIXME: Can't process encoding file
      }
    }
  }
}
