import { InjectQueue } from '@nestjs/bullmq';
import { Injectable, OnModuleInit } from '@nestjs/common';
import { DEFAULT_SCHEDULER, SchedulerConfigService } from '@woodstock/shared';
import { QueueName } from '@woodstock/shared';
import { Queue } from 'bullmq';

@Injectable()
export class SchedulerService implements OnModuleInit {
  constructor(
    @InjectQueue(QueueName.SCHEDULE_QUEUE) private scheduleQueue: Queue<unknown>,
    private schedulerConfigService: SchedulerConfigService,
  ) {}

  async #addRepeatableJob(name: string, pattern: string) {
    const repeatableJobs = await this.scheduleQueue.getRepeatableJobs();
    const job = repeatableJobs.find((job) => job.name === name);

    // If same pattern, do nothing
    if (job && job.pattern === pattern) {
      return;
    }

    // If different pattern, remove and add
    if (job) {
      await this.scheduleQueue.removeRepeatableByKey(job.key);
    }

    // Else
    await this.scheduleQueue.add(
      name,
      {},
      {
        repeat: {
          pattern,
        },
      },
    );
  }

  async startScheduler(): Promise<void> {
    const wakeupSchedule =
      (await this.schedulerConfigService.getScheduler()).wakeupSchedule ?? DEFAULT_SCHEDULER.wakeupSchedule;
    const nightlySchedule =
      (await this.schedulerConfigService.getScheduler()).nightlySchedule ?? DEFAULT_SCHEDULER.nightlySchedule;

    if (wakeupSchedule) {
      await this.#addRepeatableJob('wakeup', wakeupSchedule);
    }

    if (nightlySchedule) {
      await this.#addRepeatableJob('nightly', nightlySchedule);
    }
  }

  async onModuleInit(): Promise<void> {
    await this.startScheduler();
  }
}
