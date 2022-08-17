import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { InputOutputModule } from '../input-output';
import { ExecuteCommandService } from './execute-command.service';
import { PingService } from './ping.service';
import { ResolveService } from './resolve.service';
import { ToolsService } from './tools.service';

@Module({
  imports: [ApplicationConfigModule, InputOutputModule],
  providers: [ToolsService, ExecuteCommandService, PingService, ResolveService],
  exports: [ToolsService, ExecuteCommandService, PingService, ResolveService],
})
export class CommandsModule {}
