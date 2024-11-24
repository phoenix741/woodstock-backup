import { Injectable, LoggerService } from '@nestjs/common';
import { mkdirSync } from 'fs';
import * as logform from 'logform';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';
import { AsyncLocalStorage } from 'node:async_hooks';

import { BackupsService } from '../backups';

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

interface LogStorage {
  hostname?: string;
  backupNumber?: number;
  operation?: string;
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

  #getBackupLogger(hostname: string, backupNumber: number, operation: string): Logger {
    const key = `${hostname}-${backupNumber}-${operation}`;
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
          filename: join(destinationDirectory, operation + '-error.log'),
          level: 'error',
        }),
        new transports.File({
          filename: join(destinationDirectory, operation + '.log'),
        }),
      ],
    });

    this.#mapLogger.set(key, logger);
    return logger;
  }

  #getLogger(message: Record<string, unknown>): Logger {
    const storage = logAsyncLocalStorage.getStore();

    const hostname = (message.hostname as string | undefined) ?? storage?.hostname;
    const backupNumber = (message.backupNumber as number | undefined) ?? storage?.backupNumber;
    const operation = (message.operation as string | undefined) ?? storage?.operation;

    if (hostname !== undefined && backupNumber !== undefined && operation !== undefined) {
      return this.#getBackupLogger(hostname, backupNumber, operation);
    }

    return this.#globalLogger;
  }

  useLogger<R, TArgs extends any[]>(
    hostname: string,
    backupNumber: number,
    operation: string,
    callback: (...args: TArgs) => R,
    ...args: TArgs
  ): R {
    return logAsyncLocalStorage.run(
      { hostname, backupNumber, operation },
      (...args) => {
        return callback(...args);
      },
      ...args,
    );
  }

  closeLogger(hostname: string, backupNumber: number, operation: string): void {
    const key = `${hostname}-${backupNumber}-${operation}`;
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
