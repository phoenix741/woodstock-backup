import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { Job, Queue } from 'bull';

import { BackupTask } from '../tasks/tasks.dto';
import { HostsService } from 'src/hosts/hosts.service';
import { ApplicationConfigService } from '../config/application-config.service';
import { YamlService } from '../utils/yaml.service';
import { StatsService } from '../stats/stats.service';
import { CommandParameters } from '../server/tools.model';
import { Statistics } from '../stats/stats.model';

const DEFAULT_STATISTICS: Statistics = {
  spaces: [],
  quotas: [],
};

@Processor('schedule')
export class SchedulerConsumer {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private configService: ApplicationConfigService,
    private yamlService: YamlService,
    private statsService: StatsService,
  ) {}

  @Process('wakeup')
  async wakeupJob(job: Job<{}>) {
    this.logger.log(`Wakeup scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    for (const host of await this.hostsService.getHosts()) {
      const hasBackup = (await this.hostsQueue.getJobs(['active', 'delayed', 'waiting'])).find(
        b => b.data.host === host,
      );

      if (!hasBackup) {
        await this.hostsQueue.add('schedule_host', { host }, { removeOnComplete: true });
      }
    }
  }

  @Process('nightly')
  async nightlyJob(job: Job<{}>) {
    this.logger.log(`Nightly scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    const params: CommandParameters = {};
    const space = await this.statsService.getSpace(params);
    const volumes = await this.statsService.getBackupQuota(params);

    const statistics = await this.yamlService.loadFile(this.configService.statisticsPath, DEFAULT_STATISTICS);
    statistics.spaces.push({ timestamp: new Date().getTime(), ...space });
    statistics.quotas.push({ timestamp: new Date().getTime(), volumes });

    await this.yamlService.writeFile(this.configService.statisticsPath, statistics);
  }
}
