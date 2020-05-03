import { Controller, Get, ParseBoolPipe, Query, Res } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { Response } from 'express';
import { join } from 'path';

import { ApplicationConfigService } from '../config/application-config.service';
import { BtrfsCheck } from '../storage/btrfs/btrfs.dto';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { tailLog, getLog } from '../utils/log-utils.service';

@Controller('server')
export class ServerController {
  constructor(public applicationConfig: ApplicationConfigService, public btrfsService: BtrfsService) {}

  @Get('status')
  @ApiOkResponse({
    description: 'Get the status of the server',
    type: BtrfsCheck,
  })
  async getStatus() {
    return this.btrfsService.check();
  }

  @Get('log/application.log')
  @ApiOkResponse({
    description: 'Get the application log of the server',
    type: String,
  })
  getApplicationLog(@Query('tailable', ParseBoolPipe) tailable: boolean, @Res() res: Response) {
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
  getExceptionsLog(@Query('tailable', ParseBoolPipe) tailable: boolean, @Res() res: Response) {
    if (tailable) {
      tailLog(join(this.applicationConfig.logPath, 'exceptions.log'), res);
    } else {
      getLog(join(this.applicationConfig.logPath, 'exceptions.log'), res);
    }
  }
}
