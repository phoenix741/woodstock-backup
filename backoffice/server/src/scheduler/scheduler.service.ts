import { InjectQueue } from '@nestjs/bull';
import { Injectable, OnModuleInit } from '@nestjs/common';
import { Queue } from 'bull';

import { SchedulerConfigService } from './scheduler-config.service';

@Injectable()
export class SchedulerService implements OnModuleInit {
  constructor(
    @InjectQueue('schedule') private scheduleQueue: Queue<unknown>,
    private schedulerConfigService: SchedulerConfigService,
  ) {}

  async startScheduler(): Promise<void> {
    const wakeupSchedule = (await this.schedulerConfigService.getScheduler()).wakeupSchedule;
    const nightlySchedule = (await this.schedulerConfigService.getScheduler()).nightlySchedule;

    await this.removeAll();

    await this.scheduleQueue.add(
      'wakeup',
      {},
      {
        repeat: {
          cron: wakeupSchedule,
        },
        removeOnComplete: true,
      },
    );

    await this.scheduleQueue.add(
      'nightly',
      {},
      {
        repeat: {
          cron: nightlySchedule,
        },
        removeOnComplete: true,
      },
    );
  }

  async removeAll(): Promise<void> {
    const jobs = await this.scheduleQueue.getRepeatableJobs();
    for (const job of jobs) {
      await this.scheduleQueue.removeRepeatable(job);
    }
  }

  async onModuleInit(): Promise<void> {
    await this.startScheduler();
  }
}
