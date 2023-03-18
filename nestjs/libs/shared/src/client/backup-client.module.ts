import { Module } from '@nestjs/common';
import { BackupOnClientService } from './backup-client.service';
import { CommandsModule } from '../commands';
import { ScannerModule } from '../scanner';
import { ManifestModule } from '../manifest';

@Module({
  imports: [CommandsModule, ScannerModule, ManifestModule],
  providers: [BackupOnClientService],
  exports: [BackupOnClientService],
})
export class BackupClientModule {}
