import { BullModule } from '@nestjs/bull';
import { Module, HttpModule } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ConsoleModule } from 'nestjs-console';

import { BackupService } from './backups/backup/backup.service';
import { BackupsCommand } from './backups/backups.command';
import { BackupsService } from './backups/backups.service';
import { ApplicationConfigModule } from './config/application-config.module';
import { ApplicationConfigService } from './config/application-config.service';
import { HostsService } from './hosts/hosts.service';
import { ExecuteCommandService } from './operation/execute-command.service';
import { BullConfigService } from './queue/bull-config.factory';
import { ToolsService } from './server/tools.service';
import { StatsCommand } from './stats/stats.command';
import { StatsService } from './stats/stats.service';
import { BtrfsService } from './storage/btrfs/btrfs.service';
import { StorageModule } from './storage/storage.module';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';
import { LockService } from './utils/lock.service';
import { YamlService } from './utils/yaml.service';
import { BinaryBackupsCommand } from './binary-backups/binary-backups.command';
import { PoolService } from './storage/pool/pool.service';

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
    StorageModule,
    HttpModule,
  ],
  providers: [
    ApplicationConfigService,
    BackupsCommand,
    BinaryBackupsCommand,
    BackupService,
    BackupsService,
    BtrfsService,
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
