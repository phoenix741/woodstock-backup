import { Module } from '@nestjs/common';
import { CommandsModule } from '../commands';
import { ApplicationConfigModule } from '../config';
import { QueueModule } from '../queue';
import { JobService } from './job.service';
import { LockService } from './lock.service';

@Module({
  imports: [ApplicationConfigModule, QueueModule, CommandsModule],
  providers: [LockService, JobService],
  exports: [LockService, JobService],
})
export class BackupingModule {}
