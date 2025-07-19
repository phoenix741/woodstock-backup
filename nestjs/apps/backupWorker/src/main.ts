import 'source-map-support/register';

import { SandboxedJob } from 'bullmq';

import { NestFactory } from '@nestjs/core';
import { BackupQueueData, initializeLog, JobBackupData, JobRestoreData } from '@woodstock/shared';

import { AppModule } from './app.module.js';
import { BackupLogger } from './backup.logger.js';
import { HostConsumer } from './tasks/host.consumer.js';

module.exports = async (job: SandboxedJob<BackupQueueData>) => {
  await initializeLog();

  const app = await NestFactory.create(AppModule, {
    bufferLogs: true,
  });

  const logger = app.get(BackupLogger);
  logger.updateLogger(
    job.id,
    job.name,
    (job.data as JobBackupData | JobRestoreData)?.host,
    (job.data as JobBackupData | JobRestoreData)?.number,
  );

  app.useLogger(logger);
  app.flushLogs();

  const hostConsumer = app.get(HostConsumer);
  await hostConsumer.process(job);
};
