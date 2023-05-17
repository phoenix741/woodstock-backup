import { InjectQueue } from '@nestjs/bullmq';
import { Injectable, OnModuleInit } from '@nestjs/common';
import { SchedulerConfigService } from '@woodstock/core';
import { QueueName } from '@woodstock/server';
import { Queue } from 'bullmq';

@Injectable()
export class SchedulerService implements OnModuleInit {
  constructor(
    @InjectQueue(QueueName.SCHEDULE_QUEUE) private scheduleQueue: Queue<unknown>,
    private schedulerConfigService: SchedulerConfigService,
  ) {}

  async startScheduler(): Promise<void> {
    const wakeupSchedule = (await this.schedulerConfigService.getScheduler()).wakeupSchedule;
    const nightlySchedule = (await this.schedulerConfigService.getScheduler()).nightlySchedule;

    await this.removeAll();

    if (wakeupSchedule) {
      await this.scheduleQueue.add(
        'wakeup',
        {},
        {
          repeat: {
            pattern: wakeupSchedule,
          },
        },
      );
    }

    if (nightlySchedule) {
      await this.scheduleQueue.add(
        'nightly',
        {},
        {
          repeat: {
            pattern: nightlySchedule,
          },
        },
      );
    }
  }

  async removeAll(): Promise<void> {
    const jobs = await this.scheduleQueue.getRepeatableJobs();
    for (const job of jobs) {
      await this.scheduleQueue.removeRepeatableByKey(job.key);
    }
  }

  async onModuleInit(): Promise<void> {
    await this.startScheduler();
  }
}
