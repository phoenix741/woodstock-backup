import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ConsoleModule } from 'nestjs-console';

import { ApplicationConfigModule } from './config/application-config.module';
import { ApplicationConfigService } from './config/application-config.service';
import { BullConfigService } from './queue/bull-config.factory';
import { StatsCommand } from './stats/stats.command';
import { StatsConsumer } from './stats/stats.consumer';
import { StatsService } from './stats/stats.service';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';
import { HostsService } from './hosts/hosts.service';
import { BackupsService } from './backups/backups.service';
import { YamlService } from './utils/yaml.service';
import { LockService } from './utils/lock.service';
import { ExecuteCommandService } from './operation/execute-command.service';
import { ToolsService } from './server/tools.service';

@Module({
  imports: [
    ConfigModule.forRoot(),
    ApplicationConfigModule,
    BullModule.registerQueueAsync({
      name: 'queue',
      imports: [ApplicationConfigModule],
      useClass: BullConfigService,
    }),
    ConsoleModule,
  ],
  providers: [
    ApplicationConfigService,
    BackupsService,
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
