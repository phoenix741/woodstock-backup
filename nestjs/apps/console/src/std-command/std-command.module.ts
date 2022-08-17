import { Module } from '@nestjs/common';
import {
  ApplicationConfigModule,
  InputOutputModule,
  MaintenanceModule,
  PoolModule,
  ScannerModule,
} from '@woodstock/shared';
import { BrowserCommand } from './browser.command';
import { PoolCommand } from './pool.command';
import { ProtobufCommand } from './protobuf.command';

@Module({
  imports: [ApplicationConfigModule, ScannerModule, MaintenanceModule, PoolModule, InputOutputModule],
  providers: [BrowserCommand, PoolCommand, ProtobufCommand],
})
export class StdCommandModule {}
