import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { AuthentificationModule, CoreModule } from '@woodstock/shared';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { BackupService } from './backup/backup.service.js';
import { ClientConfigService } from './config/client.config.js';
import { GlobalModule } from './global.module.js';
import { LoggerModule } from './logger/log.module.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, AuthentificationModule, LoggerModule],
  controllers: [AppController],
  providers: [ClientConfigService, AppService, BackupService],
})
export class AppModule {}
