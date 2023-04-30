import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { LockService } from './lock.service';

@Module({
  imports: [ApplicationConfigModule],
  providers: [LockService],
  exports: [LockService],
})
export class SharedModule {}
