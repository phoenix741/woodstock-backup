import { Injectable, LoggerService } from '@nestjs/common';
import * as logform from 'logform';
import * as mkdirp from 'mkdirp';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';
import 'winston-daily-rotate-file';

const { combine, timestamp, printf, colorize } = format;

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${(info.context || '').padEnd(25, ' ')}] ${info.level}: ${info.message}`;
});

@Injectable()
export class ApplicationLogger implements LoggerService {
  #logger: Logger;

  constructor(readonly worker?: string, readonly console = true) {
    const logPath = join(process.env.BACKUP_PATH || '', 'log');
    mkdirp(logPath);

    this.#logger = createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        ...(console
          ? [
              new transports.Console({
                format: combine(colorize(), timestamp(), applicationFormat),
              }),
            ]
          : []),
        new transports.DailyRotateFile({
          filename: join(logPath, `application-${worker}-%DATE%.log`),
          datePattern: 'YYYY-MM-DD',
          zippedArchive: true,
          maxSize: '2m', // Config
          maxFiles: '31d', // Config
          createSymlink: true,
          symlinkName: `application-${worker}.log`,
        }),
      ],
      exceptionHandlers: [
        new transports.Console({
          format: combine(colorize(), timestamp(), applicationFormat),
        }),
        new transports.DailyRotateFile({
          filename: join(logPath, `exceptions-${worker}-%DATE%.log`),
          datePattern: 'YYYY-MM-DD',
          zippedArchive: true,
          maxSize: '2m', // Config
          maxFiles: '31d', // Config
          createSymlink: true,
          symlinkName: `exceptions-${worker}.log`,
        }),
      ],
    });
  }

  log(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.info(typeof message === 'string' ? { context, message } : { context, ...message });
  }

  error(message: string | Record<string, unknown>, trace?: string, context?: string): void {
    this.#logger.error(typeof message === 'string' ? { context, trace, message } : { context, trace, ...message });
  }

  warn(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.warn(typeof message === 'string' ? { context, message } : { context, ...message });
  }

  debug(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.debug(typeof message === 'string' ? { context, message } : { context, ...message });
  }

  verbose(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.verbose(typeof message === 'string' ? { context, message } : { context, ...message });
  }
}
