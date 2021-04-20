import { Module } from '@nestjs/common';
import { SharedModule } from '@woodstock/shared';

import { AppController } from './app.controller';
import { AppService } from './app.service';
import { LogService } from './log.service';

@Module({
  imports: [SharedModule],
  controllers: [AppController],
  providers: [LogService, AppService],
})
export class AppModule {}
