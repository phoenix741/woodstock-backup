import { Injectable, Logger } from '@nestjs/common';
import * as Rsync from 'rsync';

import { compact } from '../utils/lodash';
import { BackupContext, BackupOptions } from './interfaces/rsync-backup-options';

const PROGRESS_XFR = /.*\(xfr#(\d+),\s+\w+-chk=(\d+)\/(\d+)\).*/;
const PROGRESS_INFO = /\s+([\d,]+)\s+(\d+)%\s+([\d.]+)(\wB)\/s\s+(\d+:\d{1,2}:\d{1,2})\s*/;

const sizes = ['bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];

export interface RSyncBackupOptions extends BackupOptions {
  rsync: boolean;
  username?: string;
}

export interface RSyncdBackupOptions extends BackupOptions {
  rsyncd: boolean;
  authentification: boolean;
  username?: string;
  password?: string;
}

@Injectable()
export class RSyncCommandService {
  private logger = new Logger(RSyncCommandService.name);

  /**
   * Launch a RSync backup
   *
   * @param host Host to backup (resolved)
   * @param sharePath The source path
   * @param destination The destination path
   * @param options Options of the backup
   * @returns Is the backup partial
   */
  async backup(host: string | null, sharePath: string, destination: string, options: RSyncBackupOptions | RSyncdBackupOptions): Promise<void> {
    const isRsyncVersionGreaterThan31 = true;

    const rsync = new Rsync();

    if ((options as RSyncBackupOptions).rsync) {
      rsync.shell(`/usr/bin/ssh ${options.username ? '-l ' + options.username : ''} -o stricthostkeychecking=no -o userknownhostsfile=/dev/null -o batchmode=yes -o passwordauthentication=no`);
    }

    rsync
      .flags('vD')
      .set('super')
      .set('recursive')
      .set('protect-args')
      .set('numeric-ids')
      .set('perms')
      .set('owner')
      .set('group')
      .set('times')
      .set('links')
      .set('hard-links')
      .set('delete')
      .set('delete-excluded')
      .set('one-file-system')
      .set('partial')
      .set('stats')
      .set('checksum')
      .set('log-format', 'log: %o %i %B %8U,%8G %9l %f%L');

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

    if (options.includes.length) {
      rsync.include(options.includes);
    }
    if (options.excludes.length) {
      rsync.exclude(options.excludes);
    }

    if ((options as RSyncBackupOptions).rsync) {
      if (host) {
        rsync.source(`${host}:${sharePath}/`);
      } else {
        rsync.source(`${sharePath}/`);
      }
    }
    if ((options as RSyncdBackupOptions).rsyncd) {
      let authentification = '';
      if ((options as RSyncdBackupOptions).authentification) {
        authentification = `${options.username}:${(options as RSyncdBackupOptions).password}@`;
      }
      rsync.source(`${authentification}${host}::${sharePath}/`);
    }

    rsync.destination(destination);

    this.logger.log({ message: `Execute command ${rsync.command()}`, sharePath, ...options });

    return new Promise((resolve, reject) => {
      const context: BackupContext = new BackupContext(sharePath);
      rsync.execute(
        (error, code) => {
          if (error && code !== 24) {
            return reject(error);
          }
          options.callbackProgress(context);
          resolve();
        },
        (data: any) => this.processOutput(context, options, data),
        (data: any) => this.processOutput(context, options, data, true),
      );
    });
  }

  private processOutput(context: BackupContext, options: RSyncBackupOptions | RSyncdBackupOptions, data: any, error = false) {
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
              compact({
                newFileCount: this.rsyncNumberToInt(count),
                fileCount: this.rsyncNumberToInt(total),
              }),
            );

            options.callbackProgress(context);
          }

          const progressionWithoutXfer = line.match(PROGRESS_INFO);
          if (progressionWithoutXfer) {
            const [, transferedFileSize, percent, speed, speedUnit] = progressionWithoutXfer;

            Object.assign(
              context,
              compact({
                newFileSize: this.rsyncNumberToInt(transferedFileSize),
                percent: this.rsyncNumberToInt(percent),
                speed: this.rsyncNumberToInt(speed, speedUnit),
              }),
            );

            options.callbackProgress(context);
            return false;
          }

          Object.assign(
            context,
            compact({
              fileCount: this.getValueOfRegex(line, /Number of files:\s+([\d+,.]+)\s+.*/),
              newFileCount: this.getValueOfRegex(line, /Number of created files:\s+([\d+,.]+)\s+.*/),
              fileSize: this.getValueOfRegex(line, /Total file size:\s+([\d+,.]+)\s+.*/),
              newFileSize: this.getValueOfRegex(line, /Total transferred file size:\s+([\d+,.]+)\s+.*/),
            }),
          );
        }

        return true;
      })
      .forEach((line: string) => (error ? this.logger.error({ message: line, sharePath: context.sharePath, ...options }) : this.logger.log({ message: line, sharePath: context.sharePath, ...options })));
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
