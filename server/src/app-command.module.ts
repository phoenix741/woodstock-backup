import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ConsoleModule } from 'nestjs-console';

import { BackupsCommand } from './backups/backups.command';
import { BackupsService } from './backups/backups.service';
import { ApplicationConfigModule } from './config/application-config.module';
import { ApplicationConfigService } from './config/application-config.service';
import { HostsService } from './hosts/hosts.service';
import { ExecuteCommandService } from './operation/execute-command.service';
import { BullConfigService } from './queue/bull-config.factory';
import { ToolsService } from './server/tools.service';
import { StatsCommand } from './stats/stats.command';
import { StatsConsumer } from './stats/stats.consumer';
import { StatsService } from './stats/stats.service';
import { BtrfsService } from './storage/btrfs/btrfs.service';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';
import { LockService } from './utils/lock.service';
import { YamlService } from './utils/yaml.service';

@Module({
  imports: [
    ConfigModule.forRoot(),
    ApplicationConfigModule,
    BullModule.registerQueueAsync(
      {
        name: 'queue',
        imports: [ApplicationConfigModule],
        useClass: BullConfigService,
      },
      {
        name: 'schedule',
        imports: [ApplicationConfigModule],
        useClass: BullConfigService,
      },
    ),
    ConsoleModule,
  ],
  providers: [
    ApplicationConfigService,
    BackupsCommand,
    BackupsService,
    BtrfsService,
    ExecuteCommandService,
    HostConsumerUtilService,
    HostsService,
    LockService,
    StatsConsumer,
    StatsService,
    StatsCommand,
    ToolsService,
    YamlService,
  ],
})
export class AppCommandModule {}
