import 'winston-daily-rotate-file';

import { Injectable, LoggerService } from '@nestjs/common';
import { mkdirSync } from 'fs';
import * as logform from 'logform';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';
import { ApplicationConfigService, BackupsService } from '@woodstock/shared';

const { combine, timestamp, printf } = format;

function padString(value: string | undefined) {
  if (value === undefined) {
    return '';
  }
  // If value is not string
  if (typeof value !== 'string') {
    value = JSON.stringify(value);
  }
  return value.padEnd(25, ' ');
}

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${padString((info.hostname as string) ?? 'global')}][${padString(info.context as string)}] ${info.level}: ${info.message} ${info.trace ?? ''}`;
});

export interface LogStorage {
  jobId: string;
  operation?: string;
  hostname?: string;
  backupNumber?: number;
}

@Injectable()
export class BackupLogger implements LoggerService {
  #logger: Logger;

  constructor(
    private config: ApplicationConfigService,
    private backupsService: BackupsService,
  ) {}

  #createLogger(jobId: string, operation?: string, hostname?: string, backupNumber?: number) {
    const hostPath = hostname && backupNumber && operation !== 'remove';
    const destinationDirectory = hostPath
      ? this.backupsService.getLogDirectory(hostname, backupNumber ?? 0)
      : this.config.jobPath;

    const filename = hostPath ? `${operation}.log` : `${jobId}-${operation}.log`;
    const errorFilename = hostPath ? `${operation}-error.log` : `${jobId}-${operation}-error.log`;

    mkdirSync(destinationDirectory, { recursive: true });
    return createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: join(destinationDirectory, errorFilename),
          level: 'error',
        }),
        new transports.File({
          filename: join(destinationDirectory, filename),
        }),
      ],
    });
  }

  updateLogger(jobId: string, operation?: string, hostname?: string, backupNumber?: number) {
    this.#logger?.close();
    this.#logger = this.#createLogger(jobId, operation, hostname, backupNumber);
  }

  log(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#logger.info(msg);
  }

  error(message: string | Record<string, unknown>, trace?: string, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#logger.error(msg);
  }

  warn(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#logger.warn(msg);
  }

  debug(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#logger.debug(msg);
  }

  verbose(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#logger.verbose(msg);
  }
}
