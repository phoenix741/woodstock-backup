import { LoggerService } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as logform from 'logform';
import * as mkdirp from 'mkdirp';
import { createLogger, format, Logger, transports } from 'winston';

import { BackupList } from '../backups/backup-list.class';

const { combine, timestamp, printf } = format;

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.context}] ${info.level}: ${info.message}`;
});

export class BackupLogger implements LoggerService {
  private logger: Logger;
  private hostpath: string;

  constructor(configService: ConfigService, hostname: string, number?: number) {
    this.hostpath = configService.get<string>('paths.hostPath', '<defunct>');
    const host = new BackupList(this.hostpath, hostname);
    const destinationDirectory = host.getLogDirectory();

    mkdirp(destinationDirectory);
    this.logger = createLogger({
      level: 'debug',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: host.getLogFile(number, 'error'),
          level: 'error',
        }),
        new transports.File({ filename: host.getLogFile(number) }),
      ],
    });
  }

  log(message: string | object, context?: string) {
    this.logger.info(typeof message === 'string' ? { context, message } : { context, ...message });
  }

  error(message: string | object, trace?: string, context?: string) {
    this.logger.error(typeof message === 'string' ? { context, trace, message } : { context, trace, ...message });
  }

  warn(message: string | object, context?: string) {
    this.logger.warn(typeof message === 'string' ? { context, message } : { context, ...message });
  }

  debug(message: string | object, context?: string) {
    this.logger.debug(typeof message === 'string' ? { context, message } : { context, ...message });
  }

  verbose(message: string | object, context?: string) {
    this.logger.verbose(typeof message === 'string' ? { context, message } : { context, ...message });
  }
}
