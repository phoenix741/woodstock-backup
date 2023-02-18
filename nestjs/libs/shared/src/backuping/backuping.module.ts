import { Module } from '@nestjs/common';
import { CommandsModule } from '../commands';
import { ApplicationConfigModule } from '../config';
import { QueueModule } from '../queue';
import { SharedModule } from '../shared/shared.module';
import { JobService } from './job.service';

@Module({
  imports: [ApplicationConfigModule, QueueModule, CommandsModule, SharedModule],
  providers: [JobService],
  exports: [JobService],
})
export class BackupingModule {}
