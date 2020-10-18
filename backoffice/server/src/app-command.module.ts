import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { SharedModule } from '@woodstock/shared';
import { ConsoleModule } from 'nestjs-console';
import { BackupsCommand } from './backups/backups.command';
import { BackupsService } from './backups/backups.service';
import { PoolService } from './backups/pool/pool.service';
import { ApplicationConfigModule } from './config/application-config.module';
import { ApplicationConfigService } from './config/application-config.service';
import { HostsService } from './hosts/hosts.service';
import { BullConfigService } from './queue/bull-config.factory';
import { ToolsService } from './server/tools.service';
import { StatsCommand } from './stats/stats.command';
import { StatsService } from './stats/stats.service';
import { ExecuteCommandService } from './utils/execute-command.service';
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
    SharedModule,
  ],
  providers: [
    ApplicationConfigService,
    BackupsCommand,
    BackupsService,
    ExecuteCommandService,
    HostConsumerUtilService,
    HostsService,
    LockService,
    StatsCommand,
    StatsService,
    ToolsService,
    YamlService,
    PoolService,
  ],
})
export class AppCommandModule {}
