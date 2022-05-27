import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule } from '@woodstock/shared';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { BackupService } from './backup/backup.service';
import { ClientConfigService } from './config/client.config';
import { GlobalModule } from './global.module';
import { LoggerModule } from './logger/log.module';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, SharedModule, LoggerModule],
  controllers: [AppController],
  providers: [ClientConfigService, AppService, BackupService],
})
export class AppModule {}
