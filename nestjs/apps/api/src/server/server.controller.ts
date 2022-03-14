import { Controller, Get, ParseBoolPipe, Query, Res } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { ApplicationConfigService } from '@woodstock/backoffice-shared';
import { Response } from 'express';
import { join } from 'path';
import { getLog, tailLog } from '../utils/log-utils.service';
import { ServerChecks } from './server.dto';
import { ServerService } from './server.service';

@Controller('server')
export class ServerController {
  constructor(public applicationConfig: ApplicationConfigService, public serverService: ServerService) {}

  @Get('status')
  @ApiOkResponse({
    description: 'Get the status of the server',
    type: ServerChecks,
  })
  async getStatus(): Promise<ServerChecks> {
    return this.serverService.check();
  }

  @Get('log/application.log')
  @ApiOkResponse({
    description: 'Get the application log of the server',
    type: String,
  })
  getApplicationLog(@Query('tailable', ParseBoolPipe) tailable: boolean, @Res() res: Response): void {
    if (tailable) {
      tailLog(join(this.applicationConfig.logPath, 'application.log'), res);
    } else {
      getLog(join(this.applicationConfig.logPath, 'application.log'), res);
    }
  }

  @Get('log/exceptions.log')
  @ApiOkResponse({
    description: 'Get the exceptions log of the server',
    type: String,
  })
  getExceptionsLog(@Query('tailable', ParseBoolPipe) tailable: boolean, @Res() res: Response): void {
    if (tailable) {
      tailLog(join(this.applicationConfig.logPath, 'exceptions.log'), res);
    } else {
      getLog(join(this.applicationConfig.logPath, 'exceptions.log'), res);
    }
  }
}
