import { Module } from '@nestjs/common';
import { SharedModule } from '@woodstock/shared';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { BackupService } from './backup/backup.service';
import { LoggerModule } from './logger/log.module';

@Module({
  imports: [SharedModule, LoggerModule],
  controllers: [AppController],
  providers: [AppService, BackupService],
})
export class AppModule {}
