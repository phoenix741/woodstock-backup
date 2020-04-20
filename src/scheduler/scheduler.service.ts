import { Injectable, OnModuleInit } from '@nestjs/common';
import { HostsService } from 'src/hosts/hosts.service';
import { InjectQueue } from '@nestjs/bull';
import { Queue } from 'bull';
import { SchedulerConfigService } from './scheduler-config.service';

@Injectable()
export class SchedulerService implements OnModuleInit {
  constructor(@InjectQueue('schedule') private scheduleQueue: Queue<object>, private schedulerConfigService: SchedulerConfigService, private hostService: HostsService) {}

  async startScheduler() {
    const schedule = (await this.schedulerConfigService.getScheduler()).wakeupSchedule;
    await this.removeAll();

    await this.scheduleQueue.add(
      {},
      {
        repeat: {
          cron: schedule,
        },
        removeOnComplete: true,
      },
    );
  }

  async removeAll() {
    const jobs = await this.scheduleQueue.getRepeatableJobs();
    for (const job of jobs) {
      await this.scheduleQueue.removeRepeatable(job);
    }
  }

  async onModuleInit() {
    await this.startScheduler();
  }
}
