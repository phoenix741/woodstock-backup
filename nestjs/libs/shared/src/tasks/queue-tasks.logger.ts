import { LoggerService } from '@nestjs/common';
import { Job } from 'bullmq';

export class JobLogger implements LoggerService {
  constructor(private job: Job<unknown>) {}

  log(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? message : JSON.stringify(message);

    this.job.log(`[LOG] ${context} ${msg}`);
  }

  error(message: string | Record<string, unknown>, trace?: string, context?: string): void {
    const msg = typeof message === 'string' ? message : JSON.stringify(message);

    this.job.log(`[ERROR] ${context} ${msg} ${trace}`);
  }

  warn(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? message : JSON.stringify(message);

    this.job.log(`[WARN] ${context} ${msg}`);
  }

  debug(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? message : JSON.stringify(message);

    this.job.log(`[DEBUG] ${context} ${msg}`);
  }

  verbose(message: string | Record<string, unknown>, context?: string): void {
    const msg = typeof message === 'string' ? message : JSON.stringify(message);

    this.job.log(`[VERBOSE] ${context} ${msg}`);
  }
}
