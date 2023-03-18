import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { AuthentificationModule, CoreModule } from '@woodstock/shared';
import { BackupClientModule } from '@woodstock/shared/client/backup-client.module.js';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { ClientConfigService } from './config/client.config.js';
import { GlobalModule } from './global.module.js';
import { LoggerModule } from './logger/log.module.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, AuthentificationModule, LoggerModule, BackupClientModule],
  controllers: [AppController],
  providers: [ClientConfigService, AppService],
})
export class AppModule {}
