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
import { ApplicationConfigService } from '@woodstock/shared';
import { Backup, JobBackupData, QueueName } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { Response } from 'express';
import { join } from 'path';
import { getLog, tailLog } from '../utils/log-utils.service.js';
import { BackupsService, HostsService } from '@woodstock/shared';

@Controller('hosts/:name/backups')
export class BackupController {
  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
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
    const filename = join(this.applicationConfig.hostPath, name, '' + number, `log`);
    if (tailable) {
      tailLog(filename, res);
    } else {
      getLog(filename, res);
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
    const filename = join(this.applicationConfig.hostPath, name, '' + number, `error`);
    if (tailable) {
      tailLog(filename, res);
    } else {
      getLog(filename, res);
    }
  }

  @Get(':number/xferLog/:share.log')
  @ApiOkResponse({
    description: 'Get the share log of the server',
    type: String,
  })
  getXFerLog(
    @Param('name') name: string,
    @Param('number', ParseIntPipe) number: number,
    @Param('share') share: string,
    @Res() res: Response,
  ): void {
    res.header('Content-Type', 'text/plain;charset=utf-8');

    this.backupsService.readLog(name, number, share).subscribe({
      next: (data) => res.write(data + '\n'),
      error: (err) => res.status(500).end(err.message),
      complete: () => res.end(),
    });
  }
}
