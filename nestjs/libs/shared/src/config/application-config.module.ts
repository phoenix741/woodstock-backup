import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigService } from './application-config.service';

@Module({
  imports: [ConfigModule],
  providers: [ApplicationConfigService],
  exports: [ApplicationConfigService],
})
export class ApplicationConfigModule {}
