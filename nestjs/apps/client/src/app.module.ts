import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule } from '@woodstock/shared';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { BackupService } from './backup/backup.service.js';
import { ClientConfigService } from './config/client.config.js';
import { GlobalModule } from './global.module.js';
import { LoggerModule } from './logger/log.module.js';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, SharedModule, LoggerModule],
  controllers: [AppController],
  providers: [ClientConfigService, AppService, BackupService],
})
export class AppModule {}
