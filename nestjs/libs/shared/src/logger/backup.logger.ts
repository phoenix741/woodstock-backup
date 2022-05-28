import { LoggerService } from '@nestjs/common';
import * as logform from 'logform';
import * as mkdirp from 'mkdirp';
import { createLogger, format, Logger, transports } from 'winston';
import { BackupsService } from '../services';

const { combine, timestamp, printf } = format;

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.hostname}][${info.context}] ${info.level}: ${info.message} ${
    info.trace ? info.trace : ''
  }`;
});

export class BackupLogger implements LoggerService {
  private logger: Logger;

  constructor(backupsService: BackupsService, private hostname: string, number?: number) {
    const destinationDirectory = backupsService.getLogDirectory(hostname);

    mkdirp(destinationDirectory);
    this.logger = createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: backupsService.getLogFile(hostname, number, 'error'),
          level: 'error',
        }),
        new transports.File({ filename: backupsService.getLogFile(hostname, number) }),
      ],
    });
  }

  log(message: string | Record<string, unknown>, context?: string): void {
    this.logger.info(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }

  error(message: string | Record<string, unknown>, trace?: string, context?: string): void {
    this.logger.error(
      typeof message === 'string'
        ? { context, trace, message, hostname: this.hostname }
        : { context, trace, ...message, hostname: this.hostname },
    );
  }

  warn(message: string | Record<string, unknown>, context?: string): void {
    this.logger.warn(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }

  debug(message: string | Record<string, unknown>, context?: string): void {
    this.logger.debug(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }

  verbose(message: string | Record<string, unknown>, context?: string): void {
    this.logger.verbose(
      typeof message === 'string'
        ? { context, message, hostname: this.hostname }
        : { context, ...message, hostname: this.hostname },
    );
  }
}
