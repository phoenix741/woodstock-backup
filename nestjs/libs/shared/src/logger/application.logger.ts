import { Injectable, LoggerService } from '@nestjs/common';
import { mkdirSync } from 'fs';
import * as logform from 'logform';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';

import 'winston-daily-rotate-file';

const { combine, timestamp, printf, colorize } = format;

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
export class ApplicationLogger implements LoggerService {
  #logger: Logger;

  constructor(readonly worker: string) {
    this.#logger = this.#createGlobalLogger(worker);
  }

  #createGlobalLogger(worker: string): Logger {
    const logPath = join(process.env.BACKUP_PATH || '', 'logs');
    mkdirSync(logPath, { recursive: true });

    const options = {
      datePattern: 'YYYY-MM-DD',
      zippedArchive: true,
      maxSize: '2m', // Config
      maxFiles: '31d', // Config
      createSymlink: true,
    };

    return createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.Console({
          format: combine(colorize({ all: true }), timestamp(), applicationFormat),
        }),
        new transports.DailyRotateFile({
          filename: join(logPath, `application-${worker}-%DATE%.log`),
          symlinkName: `application-${worker}.log`,
          ...options,
        }),
      ],
      exceptionHandlers: [
        new transports.Console({
          format: combine(colorize({ all: true }), timestamp(), applicationFormat),
        }),
        new transports.DailyRotateFile({
          filename: join(logPath, `application-${worker}-%DATE%.log`),
          symlinkName: `application-${worker}.log`,
          ...options,
        }),
      ],
      //exitOnError: false,
    });
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
