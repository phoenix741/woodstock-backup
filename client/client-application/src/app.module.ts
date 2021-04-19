import { Module } from '@nestjs/common';
import { SharedModule } from '@woodstock/shared';

import { AppController } from './app.controller';
import { LogService } from './log.service';
import { AppService } from './app.service';

@Module({
  imports: [SharedModule],
  controllers: [AppController],
  providers: [LogService, AppService],
})
export class AppModule {}
