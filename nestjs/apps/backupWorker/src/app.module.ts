import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { BackupClientGrpc } from './backups/backup-client-grpc.class';
import { BackupClientProgress } from './backups/backup-client-progress.service';
import { BackupClient } from './backups/backup-client.service';
import { GlobalModule } from './global.module';
import { HostConsumer } from './tasks/host.consumer';
import { RemoveService } from './tasks/remove.service';
import { TasksService } from './tasks/tasks.service';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [
    BackupClientProgress,
    BackupClient,
    BackupClientGrpc,
    HostConsumer,
    HostConsumerUtilService,
    TasksService,
    RemoveService,
  ],
})
export class AppModule {}
