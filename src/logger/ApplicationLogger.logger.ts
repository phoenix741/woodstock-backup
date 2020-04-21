import { LoggerService, Injectable } from '@nestjs/common';
import * as logform from 'logform';
import * as mkdirp from 'mkdirp';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';

const { combine, timestamp, printf, colorize } = format;

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.context.padEnd(25, ' ')}] ${info.level}: ${info.message}`;
});

@Injectable()
export class ApplicationLogger implements LoggerService {
  private logger: Logger;

  constructor() {
    const logPath = join(process.env.BACKUP_PATH || '', 'log');
    mkdirp(logPath);
    this.logger = createLogger({
      level: 'debug',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.Console({
          format: combine(colorize(), timestamp(), applicationFormat),
        }),
        new transports.File({ filename: join(logPath, 'application.log') }),
      ],
      exceptionHandlers: [new transports.File({ filename: join(logPath, 'exceptions.log') })],
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
