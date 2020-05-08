import { Injectable } from '@nestjs/common';
import * as fs from 'fs';
import { join } from 'path';
import * as mkdirp from 'mkdirp';
import * as Rsync from 'rsync';
import { Observable, Subject } from 'rxjs';
import * as tmp from 'tmp';

import { BackupLogger } from '../logger/BackupLogger.logger';
import { ToolsService } from '../server/tools.service';
import { compactObject } from '../utils/lodash';
import { BackupProgression } from './interfaces/options';
import { BackupContext, BackupOptions } from './interfaces/rsync-backup-options';
import { CommandParameters } from '../server/tools.model';

tmp.setGracefulCleanup();

const PROGRESS_XFR = /.*\(xfr#(\d+),\s+\w+-chk=(\d+)\/(\d+)\).*/;
const PROGRESS_INFO = /\s+([\d,]+)\s+(\d+)%\s+([\d.]+)(\wB)\/s\s+(\d+:\d{1,2}:\d{1,2})\s*/;

const sizes = ['bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];

export interface RSyncBackupOptions extends BackupOptions {
  rsync: boolean;
  username?: string;
  checksum?: boolean;
}

export interface RSyncdBackupOptions extends BackupOptions {
  rsyncd: boolean;
  authentification: boolean;
  username?: string;
  password?: string;
  checksum?: boolean;
}

@Injectable()
export class RSyncCommandService {
  constructor(private toolsService: ToolsService) {}

  /**
   * Launch a RSync backup
   *
   * @param host Host to backup (resolved)
   * @param sharePath The source path
   * @param destination The destination path
   * @param options Options of the backup
   * @returns Is the backup partial
   */
  backup(
    params: CommandParameters,
    sharePath: string,
    options: RSyncBackupOptions | RSyncdBackupOptions,
  ): Observable<BackupProgression> {
    const progression = new Subject<BackupProgression>();
    progression.next(new BackupProgression(0));

    tmp.file({ detachDescriptor: true }, async (err, path, fd, cleanupCallback) => {
      if (err) {
        return progression.thrownError(err);
      }
      try {
        const isRsyncVersionGreaterThan31 = true;

        const rsync = new Rsync();

        rsync
          .flags('vD')
          .set('super')
          .set('recursive')
          .set('protect-args')
          .set('numeric-ids')
          .set('perms')
          .set('owner')
          .set('group')
          .set('devices')
          .set('specials')
          .set('times')
          .set('links')
          .set('hard-links')
          .set('delete')
          .set('delete-excluded')
          .set('one-file-system')
          .set('partial')
          .set('stats')
          .set('inplace')

          .set('log-format', 'log: %o %i %B %8U,%8G %9l %f%L');

        if (options.checksum) {
          rsync.set('checksum');
        }

        // If not is root
        if (!(process.getuid && process.getuid() === 0)) {
          rsync.set('fake-super');
        }

        if (isRsyncVersionGreaterThan31) {
          rsync.set('info', 'progress2');
        }

        if (options.timeout) {
          rsync.set('timeout', '' + options.timeout);
        }

        const includes = new Set<string>();
        const excludes = new Set<string>(options.excludes || []);

        for (let include of options.includes || []) {
          include = `/${include.replace(/\/$/, '')}`.replace(/\/\/+/g, '/');
          if (include === '/') {
            includes.add(include);
            continue;
          }

          const [, ...elts] = include.split('/');
          let path = '';
          for (const elt of elts) {
            excludes.add(`${path}/*`);
            path = `${path}/${elt}`;
            includes.add(path);
          }
        }

        if (includes.size) {
          rsync.include(Array.from(includes));
        }
        if (excludes.size) {
          rsync.exclude(Array.from(excludes));
        }

        if ((options as RSyncBackupOptions).rsync) {
          rsync.set('rsync-path', await this.toolsService.getTool('rsync'));
          if (params.ip) {
            rsync.source(`${params.ip}:${sharePath}/`);
          } else {
            rsync.source(`${sharePath}/`);
          }
        }

        if ((options as RSyncBackupOptions).rsync) {
          rsync.shell(
            (await this.toolsService.getCommand('rsh', params)) + (options.username ? ' -l ' + options.username : ''),
          );
        }

        if ((options as RSyncdBackupOptions).rsyncd) {
          let authentification = '';
          if ((options as RSyncdBackupOptions).authentification) {
            authentification = `${options.username}@`;
            await fs.promises.writeFile(path, (options as RSyncdBackupOptions).password, { encoding: 'utf-8' });
            rsync.set('password-file', path);
          }
          rsync.source(`${authentification}${params.ip}::${sharePath}`);
        }

        const destination = join(await this.toolsService.getPath('destBackupPath', params), sharePath);
        rsync.destination(destination);

        options.backupLogger.log(`Execute command ${rsync.command()}`, sharePath);

        const context: BackupContext = new BackupContext(sharePath);
        await mkdirp(destination);

        rsync.execute(
          (error, code) => {
            if (error && code !== 24) {
              return progression.error(error);
            }
            progression.next(context);
            progression.complete();
            cleanupCallback();
          },
          (data: Buffer) => this.processOutput(context, options.backupLogger, progression, data),
          (data: Buffer) => this.processOutput(context, options.backupLogger, progression, data, true),
        );
      } catch (err) {
        cleanupCallback();

        progression.thrownError(err);
      }
    });

    return progression.asObservable();
  }

  private processOutput(
    context: BackupContext,
    backupLogger: BackupLogger,
    progression: Subject<BackupProgression>,
    data: Buffer,
    error = false,
  ) {
    data
      .toString()
      .split(/[\n\r]/)
      .reduce((acc: string[], line: string) => {
        const startLine = line.indexOf('log: ');
        if (startLine > 0) {
          acc.push(line.substring(0, startLine));
          acc.push(line.substring(startLine));
        } else {
          acc.push(line);
        }

        return acc;
      }, [])
      .filter((line: string): boolean => !!line)
      .filter((line: string): boolean => {
        if (!line.startsWith('log: ')) {
          const progressionWithXfer = line.match(PROGRESS_XFR);
          if (progressionWithXfer) {
            const [, count, , total] = progressionWithXfer;

            Object.assign(
              context,
              compactObject({
                newFileCount: this.rsyncNumberToInt(count),
                fileCount: this.rsyncNumberToInt(total),
              }),
            );

            progression.next(context);
          }

          const progressionWithoutXfer = line.match(PROGRESS_INFO);
          if (progressionWithoutXfer) {
            const [, transferedFileSize, percent, speed, speedUnit] = progressionWithoutXfer;

            Object.assign(
              context,
              compactObject({
                newFileSize: this.rsyncNumberToInt(transferedFileSize),
                percent: this.rsyncNumberToInt(percent),
                speed: this.rsyncNumberToInt(speed, speedUnit),
              }),
            );

            progression.next(context);
            return false;
          }

          Object.assign(
            context,
            compactObject({
              fileCount: this.getValueOfRegex(line, /Number of files:\s+([\d+,.]+)\s+.*/),
              newFileCount: this.getValueOfRegex(line, /Number of created files:\s+([\d+,.]+)\s+.*/),
              fileSize: this.getValueOfRegex(line, /Total file size:\s+([\d+,.]+)\s+.*/),
              newFileSize: this.getValueOfRegex(line, /Total transferred file size:\s+([\d+,.]+)\s+.*/),
            }),
          );
        }

        return true;
      })
      .forEach((line: string) =>
        error ? backupLogger.error(line, context.sharePath) : backupLogger.log(line, context.sharePath),
      );
  }

  private rsyncNumberToInt(value: string, unit = 'bytes'): number {
    let numberValue = parseInt(value.replace(/,/g, ''), 10);
    numberValue = numberValue * Math.pow(1024, sizes.indexOf(unit));
    return numberValue;
  }

  private getValueOfRegex(line: string, regex: RegExp): number | undefined {
    const match = line.match(regex);
    if (match) {
      return parseInt(match[1].replace(/,/g, ''), 10);
    }
    return undefined;
  }
}
