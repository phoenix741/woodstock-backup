import { Module } from '@nestjs/common';
import { LogService } from './log.service.js';

@Module({
  providers: [LogService],
  exports: [LogService],
})
export class LoggerModule {}
