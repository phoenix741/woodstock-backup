import { Injectable, LoggerService } from '@nestjs/common';
import { mkdirSync } from 'fs';
import * as logform from 'logform';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';
import { AsyncLocalStorage } from 'node:async_hooks';

import { BackupsService } from '../backups';

import 'winston-daily-rotate-file';

const { combine, timestamp, printf, colorize } = format;

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${(info.hostname ?? 'global').padEnd(25, ' ')}][${(info.context ?? '').padEnd(25, ' ')}] ${info.level}: ${info.message} ${info.trace ?? ''}`;
});

interface LogStorage {
  hostname?: string;
  backupNumber?: number;
}

const logAsyncLocalStorage = new AsyncLocalStorage<LogStorage>();

@Injectable()
export class ApplicationLogger implements LoggerService {
  #globalLogger: Logger;
  #mapLogger: Map<string, Logger> = new Map();

  constructor(
    readonly worker: string,
    private backupsService: BackupsService,
  ) {
    this.#globalLogger = this.#createGlobalLogger(worker);
  }

  #createGlobalLogger(worker: string): Logger {
    const logPath = join(process.env.BACKUP_PATH || '', 'log');
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

  #getBackupLogger(hostname: string, backupNumber: number): Logger {
    const key = `${hostname}-${backupNumber}`;
    if (this.#mapLogger.has(key)) {
      return this.#mapLogger.get(key)!;
    }

    const destinationDirectory = this.backupsService.getLogDirectory(hostname, backupNumber ?? 0);

    mkdirSync(destinationDirectory, { recursive: true });
    const logger = createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: join(destinationDirectory, 'error'),
          level: 'error',
        }),
        new transports.File({
          filename: join(destinationDirectory, 'log'),
        }),
      ],
    });

    this.#mapLogger.set(key, logger);
    return logger;
  }

  #getLogger(message: Record<string, unknown>): Logger {
    const storage = logAsyncLocalStorage.getStore();

    let hostname = (message.hostname as string | undefined) ?? storage?.hostname;
    let backupNumber = (message.backupNumber as number | undefined) ?? storage?.backupNumber;

    if (hostname !== undefined && backupNumber !== undefined) {
      return this.#getBackupLogger(hostname, backupNumber);
    }

    return this.#globalLogger;
  }

  useLogger<R, TArgs extends any[]>(
    hostname: string,
    backupNumber: number,
    callback: (...args: TArgs) => R,
    ...args: TArgs
  ): R {
    return logAsyncLocalStorage.run(
      { hostname, backupNumber },
      (...args) => {
        return callback(...args);
      },
      ...args,
    );
  }

  closeLogger(hostname: string, backupNumber: number): void {
    const key = `${hostname}-${backupNumber}`;
    if (this.#mapLogger.has(key)) {
      this.#mapLogger.get(key)!.close();
      this.#mapLogger.delete(key);
    }
  }

  log(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#getLogger(msg).info(msg);
  }

  error(message: string | Record<string, unknown>, trace?: string, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#getLogger(msg).error(msg);
  }

  warn(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#getLogger(msg).warn(msg);
  }

  debug(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#getLogger(msg).debug(msg);
  }

  verbose(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? { context, message } : { context, ...message };
    this.#getLogger(msg).verbose(msg);
  }
}
