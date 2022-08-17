import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { ManifestModule } from '../manifest';
import { PoolModule } from '../pool';
import { ScannerModule } from '../scanner';
import { FilesService } from './files.service';

@Module({
  imports: [ApplicationConfigModule, ScannerModule, ManifestModule, PoolModule],
  providers: [FilesService],
  exports: [FilesService],
})
export class FilesModule {}
