import { Process, Processor } from '@nestjs/bull';
import { Logger, NotFoundException } from '@nestjs/common';
import { Job } from 'bull';

import { BackupsService } from '../backups/backups.service';
import { BackupTask } from '../tasks/tasks.dto';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service';
import { StatsService } from './stats.service';

@Processor('queue')
export class StatsConsumer {
  private logger = new Logger(StatsConsumer.name);

  constructor(
    private statsService: StatsService,
    private hostConsumerUtilService: HostConsumerUtilService,
    private backupsService: BackupsService,
  ) {}

  @Process({ name: 'stats' })
  async calculateStats(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`START: Calculate for backup ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);

    await this.hostConsumerUtilService.lock(job);
    try {
      if (job.data.number !== undefined && job.data.number !== null) {
        await this.calculateStatsForHost(job, job.data.number);
      } else {
        const backupNumbers = (await this.backupsService.getBackups(job.data.host)).map((b) => b.number);
        for (let i = 0; i < backupNumbers.length; i++) {
          await this.calculateStatsForHost(job, backupNumbers[i]);
          job.progress((i + 1) / backupNumbers.length);
          job.update({ host: job.data.host, number: backupNumbers[i] });
        }
      }
    } catch (err) {
      this.logger.error(
        `END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`,
        err.stack,
      );
      throw err;
    } finally {
      await this.hostConsumerUtilService.unlock(job);
    }
    this.logger.debug(`END: Of calculate of the host ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);
  }

  private async calculateStatsForHost(job: Job<BackupTask>, number: number): Promise<void> {
    const backup = await this.backupsService.getBackup(job.data.host, number);
    if (!backup) {
      throw new NotFoundException(`Can't find a backup for ${job.data.host}`);
    }

    const stats = await this.statsService.calculateCompressionSize({
      hostname: job.data.host,
      destBackupNumber: number,
    });
    backup.diskUsageStatistics = stats;

    await this.backupsService.addBackup(job.data.host, backup);
  }
}
