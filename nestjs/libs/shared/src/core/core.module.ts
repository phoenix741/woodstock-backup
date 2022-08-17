import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CommandsModule } from '../commands';
import { ApplicationConfigModule } from '../config';
import { FilesModule } from '../files';
import { InputOutputModule } from '../input-output';
import { ManifestModule } from '../manifest';
import { PoolModule } from '../pool';
import { RefcntModule } from '../refcnt';
import { ScannerModule } from '../scanner';
import { StatisticsModule } from '../statistics';

export const MODULES = [
  ApplicationConfigModule,
  CommandsModule,
  ConfigModule,
  FilesModule,
  InputOutputModule,
  ManifestModule,
  PoolModule,
  RefcntModule,
  ScannerModule,
  StatisticsModule,
];

@Module({
  imports: [...MODULES],
  providers: [],
  exports: [...MODULES],
})
export class CoreModule {}
