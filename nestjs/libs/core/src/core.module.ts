import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ExecuteCommandService } from './commands/execute-command.service';
import { PingService } from './commands/ping.service';
import { ResolveService } from './commands/resolve.service';
import { ToolsService } from './commands/tools.service';
import { ApplicationConfigService } from './config/application-config.service';
import { SchedulerConfigService } from './config/scheduler-config.service';
import { ProtobufService } from './input-output/protobuf.service';
import { YamlService } from './input-output/yaml.service';

const providers = [
  ApplicationConfigService,
  SchedulerConfigService,
  ProtobufService,
  YamlService,
  ToolsService,
  ExecuteCommandService,
  PingService,
  ResolveService,
];

@Module({
  imports: [ConfigModule.forRoot({ isGlobal: true })],
  providers,
  exports: providers,
})
export class CoreModule {}
