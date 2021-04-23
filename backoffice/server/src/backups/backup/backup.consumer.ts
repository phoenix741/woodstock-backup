import { Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { CHUNK_SIZE, FileManifest, joinBuffer, Manifest, ManifestService } from '@woodstock/shared';
import { Job } from 'bull';
import { constants as constantsFs, createReadStream } from 'fs';
import { readdir } from 'fs/promises';
import * as Long from 'long';
import { concat, defer, from, Observable } from 'rxjs';
import { distinct, filter, map, mergeMap, reduce, switchMap } from 'rxjs/operators';

import { ApplicationConfigService } from '../../config/application-config.service';
import { HostsService } from '../../hosts/hosts.service';
import { PoolService } from '../../storage/pool/pool.service';
import { BackupTask } from '../../tasks/tasks.dto';
import { BackupsService } from '../backups.service';
import { BackupService } from './backup.service';

@Processor('queue')
export class BackupConsumer {
  private logger = new Logger(BackupConsumer.name);

  constructor(
    private manifestService: ManifestService,
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

      const loadIndex$ = this.manifestService.loadIndex(manifest);

      const createIndex$ = loadIndex$.pipe(
        switchMap((index) =>
          this.walkObservable(backupPath).pipe(
            mergeMap((manifest) => from((manifest.chunks || []).map((chunk, index) => ({ chunk, manifest, index })))),
            distinct((chunk) => chunk.chunk.toString('hex')),
            mergeMap((chunk) => from(this.copyChunk(chunk.manifest, chunk.index))),
            distinct((manifest) => manifest.path.toString('hex')), // FIXME: Can be have wrong chunk in the journal
            map((manifest) => this.manifestService.toAddJournalEntry(manifest, true)),
            this.manifestService.writeJournalEntry(() => manifest),
            reduce((index, journalEntry) => {
              if (journalEntry.manifest.path) {
                const entry = index.getEntry(journalEntry.manifest.path);
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
            map((file) => this.manifestService.toRemoveJournalEntry(file.path)),
            this.manifestService.writeJournalEntry(() => manifest),
          ),
        ),
      );

      const compact$ = defer(() => this.manifestService.compact(manifest));

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
      const start = CHUNK_SIZE * chunkNumber;
      const file = createReadStream(fileManifest.path, {
        start: start,
        end: start + CHUNK_SIZE - 1,
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
