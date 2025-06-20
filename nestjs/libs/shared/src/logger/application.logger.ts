import { Injectable, LoggerService, Logger as NestLogger } from '@nestjs/common';
import { mkdirSync } from 'fs';
import * as logform from 'logform';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';
import { AsyncLocalStorage } from 'node:async_hooks';

import { BackupsService } from '../backups';

import 'winston-daily-rotate-file';
import { ApplicationConfigService } from '../config';
import { LogContext, useRustLogger } from '@woodstock/shared-rs';
import { loggerCallback } from './console.logger';

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

const logAsyncLocalStorage = new AsyncLocalStorage<LogStorage>();

@Injectable()
export class ApplicationLogger implements LoggerService {
  #globalLogger: Logger;
  #mapLogger: Map<string, Logger> = new Map();

  constructor(
    readonly worker: string,
    private config: ApplicationConfigService,
    private backupsService: BackupsService,
  ) {
    this.#globalLogger = this.#createGlobalLogger(worker);
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

  #getBackupLogger(jobId: string, operation?: string, hostname?: string, backupNumber?: number): Logger {
    if (this.#mapLogger.has(jobId)) {
      return this.#mapLogger.get(jobId)!;
    }

    const destinationDirectory =
      hostname && backupNumber && operation !== 'remove'
        ? this.backupsService.getLogDirectory(hostname, backupNumber ?? 0)
        : this.config.jobPath;

    mkdirSync(destinationDirectory, { recursive: true });
    const logger = createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: join(destinationDirectory, `${jobId}-${operation}-error.log`),
          level: 'error',
        }),
        new transports.File({
          filename: join(destinationDirectory, `${jobId}-${operation}.log`),
        }),
      ],
    });

    this.#mapLogger.set(jobId, logger);
    return logger;
  }

  #getLogger(message: Record<string, unknown>): Logger {
    const storage = logAsyncLocalStorage.getStore();

    const jobId = (message.jobId as string | undefined) ?? storage?.jobId;
    const hostname = (message.hostname as string | undefined) ?? storage?.hostname;
    const backupNumber = (message.backupNumber as number | undefined) ?? storage?.backupNumber;
    const operation = (message.operation as string | undefined) ?? storage?.operation;

    if (jobId !== undefined) {
      return this.#getBackupLogger(jobId, operation, hostname, backupNumber);
    }

    return this.#globalLogger;
  }

  useLogger<R extends object, TArgs extends any[]>(
    options: LogStorage,
    callback: (context: LogContext, ...args: TArgs) => R,
    ...args: TArgs
  ): R | Promise<R> {
    const context = new LogContext();
    console.log('Using logger with options:', context.toString(), options);
    return logAsyncLocalStorage.run(
      options,
      (...args) => {
        const logger = new NestLogger('JobLogger');
        return useRustLogger(
          context,
          async () =>
            (await callback(context, ...args)) ??
            {
              /* Mandatory because napi-rs can't manage unknown as threadsafe return type */
            },
          loggerCallback(logger),
        );
      },
      ...args,
    );
  }

  closeLogger(jobId: string): void {
    if (this.#mapLogger.has(jobId)) {
      this.#mapLogger.get(jobId)!.close();
      this.#mapLogger.delete(jobId);
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
