import { LoggerService } from '@nestjs/common';
import { mkdirSync } from 'fs';
import * as logform from 'logform';
import { createLogger, format, Logger, transports } from 'winston';
import { BackupsService } from '../backups';
import { join } from 'path';

const { combine, timestamp, printf } = format;

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.hostname}][${info.context}] ${info.level}: ${info.message} ${
    info.trace ? info.trace : ''
  }`;
});

export class BackupLogger implements LoggerService {
  #logger: Logger;

  constructor(
    backupsService: BackupsService,
    private hostname: string,
    number?: number,
    clientSide = false,
  ) {
    const destinationDirectory = backupsService.getLogDirectory(hostname, number ?? 0);

    mkdirSync(destinationDirectory, { recursive: true });
    this.#logger = createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: join(destinationDirectory, clientSide ? 'error-client' : 'error'),
          level: 'error',
        }),
        new transports.File({
          filename: join(destinationDirectory, clientSide ? 'client' : 'log'),
        }),
      ],
    });
  }

  close(): void {
    this.#logger.close();
  }

  log(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.info(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }

  error(message: string | Record<string, unknown>, trace?: string, context?: string): void {
    this.#logger.error(
      typeof message === 'string'
        ? { context, trace, message, hostname: this.hostname }
        : { context, trace, ...message, hostname: this.hostname },
    );
  }

  warn(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.warn(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }

  debug(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.debug(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }

  verbose(message: string | Record<string, unknown>, context?: string): void {
    this.#logger.verbose(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }
}
