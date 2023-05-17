import { LoggerService } from '@nestjs/common';
import { ApplicationConfigService } from '@woodstock/core';
import { Job } from 'bullmq';
import * as logform from 'logform';
import { mkdirp } from 'mkdirp';
import { join } from 'path';
import { createLogger, format, Logger, transports } from 'winston';

const { combine, timestamp, printf } = format;
const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.context}] ${info.level}: ${info.message} ${info.trace ? info.trace : ''}`;
});

export class JobLogger implements LoggerService {
  #logger: Logger;

  constructor(private configService: ApplicationConfigService, job: Job<unknown>) {
    const destinationDirectory = this.configService.jobPath;
    const destinationLog = join(destinationDirectory, job.id ?? 'unknown');

    mkdirp(destinationDirectory);
    this.#logger = createLogger({
      level: process.env.LOG_LEVEL || 'info',
      format: combine(timestamp(), applicationFormat),
      transports: [
        new transports.File({
          filename: destinationLog,
        }),
      ],
    });
  }

  close(): void {
    this.#logger.close();
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
