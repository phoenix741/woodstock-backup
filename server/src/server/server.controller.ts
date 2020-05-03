import { Controller, Get, ParseBoolPipe, Query, Res } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { spawn } from 'child_process';
import { Response } from 'express';
import { join } from 'path';

import { ApplicationConfigService } from '../config/application-config.service';
import { BtrfsCheck } from '../storage/btrfs/btrfs.dto';
import { BtrfsService } from '../storage/btrfs/btrfs.service';

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
      this.tailLog('application.log', res);
    } else {
      this.getLog('application.log', res);
    }
  }

  @Get('log/exceptions.log')
  @ApiOkResponse({
    description: 'Get the exceptions log of the server',
    type: String,
  })
  getExceptionsLog(@Query('tailable', ParseBoolPipe) tailable: boolean, @Res() res: Response) {
    if (tailable) {
      this.tailLog('exceptions.log', res);
    } else {
      this.getLog('exceptions.log', res);
    }
  }

  tailLog(file: string, res: Response) {
    res.header('Content-Type', 'text/html;charset=utf-8');

    const tail = spawn('tail', ['-f', '-n', '+1', join(this.applicationConfig.logPath, file)]);
    tail.stdout.on('data', data => res.write(data, 'utf-8'));
    tail.stderr.on('data', data => res.write(data, 'utf-8'));
    tail.on('exit', code => res.end(code));
  }

  getLog(file: string, res: Response) {
    res.sendFile(join(this.applicationConfig.logPath, file));
  }
}
