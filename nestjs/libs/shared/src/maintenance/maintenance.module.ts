import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { ManifestModule } from '../manifest';
import { PoolModule } from '../pool';
import { RefcntModule } from '../refcnt';
import { FsckService } from './fsck.service';

@Module({
  imports: [ApplicationConfigModule, ManifestModule, RefcntModule, PoolModule],
  exports: [FsckService],
  providers: [FsckService],
})
export class MaintenanceModule {}
