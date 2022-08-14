import { InjectQueue } from '@nestjs/bullmq';
import {
  Controller,
  Delete,
  Get,
  NotFoundException,
  Param,
  ParseBoolPipe,
  ParseIntPipe,
  Post,
  Query,
  Res,
} from '@nestjs/common';
import { ApiCreatedResponse, ApiOkResponse } from '@nestjs/swagger';
import { ApplicationConfigService, Backup, BackupsService, BackupTask, HostsService } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { Response } from 'express';
import { join } from 'path';
import { getLog, tailLog } from '../utils/log-utils.service.js';

@Controller('hosts/:name/backups')
export class BackupController {
  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private applicationConfig: ApplicationConfigService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
  ) {}

  @Get()
  @ApiOkResponse({
    description: 'List all backup available',
    type: [Backup],
  })
  async listBackup(@Param('name') name: string): Promise<Backup[]> {
    if (!(await this.hostsService.getHosts()).includes(name)) {
      throw new NotFoundException(`Can't find the host with the name ${name}`);
    }

    return this.backupsService.getBackups(name);
  }

  @Post()
  @ApiCreatedResponse({ description: 'Ask to add a new backup' })
  async createBackup(@Param('name') name: string): Promise<void> {
    if (!(await this.hostsService.getHosts()).includes(name)) {
      throw new NotFoundException(`Can't find the host with the name ${name}`);
    }

    await this.hostsQueue.add('backup', { host: name, force: true });
  }

  @Delete(':number')
  @ApiOkResponse({
    description: 'Delete a backup',
  })
  async removeBackup(@Param('name') name: string, @Param('number', ParseIntPipe) number: number): Promise<void> {
    if (!(await this.hostsService.getHosts()).includes(name)) {
      throw new NotFoundException(`Can't find the host with the name ${name}`);
    }

    await this.hostsQueue.add('remove_backup', { host: name, number });
  }

  @Get(':number/log/backup.log')
  @ApiOkResponse({
    description: 'Get the application log of the server',
    type: String,
  })
  getApplicationLog(
    @Param('name') name: string,
    @Param('number', ParseIntPipe) number: number,
    @Query('tailable', ParseBoolPipe) tailable: boolean,
    @Res() res: Response,
  ): void {
    if (tailable) {
      tailLog(join(this.applicationConfig.hostPath, name, 'logs', `backup.${number}.log`), res);
    } else {
      getLog(join(this.applicationConfig.hostPath, name, 'logs', `backup.${number}.log`), res);
    }
  }

  @Get(':number/log/backup.error.log')
  @ApiOkResponse({
    description: 'Get the exceptions log of the server',
    type: String,
  })
  getExceptionsLog(
    @Param('name') name: string,
    @Param('number', ParseIntPipe) number: number,
    @Query('tailable', ParseBoolPipe) tailable: boolean,
    @Res() res: Response,
  ): void {
    if (tailable) {
      tailLog(join(this.applicationConfig.hostPath, name, 'logs', `backup.${number}.log`), res);
    } else {
      getLog(join(this.applicationConfig.hostPath, name, 'logs', `backup.${number}.error.log`), res);
    }
  }
}
