import { Controller, Get, Param, Query, Res, Headers } from '@nestjs/common';
import { Response } from 'express';
import * as archiver from 'archiver';
import { basename, relative, join } from 'path';
import * as fs from 'fs';

import { BackupsFilesService } from './backups-files.service';
import { ApiHeader, ApiProduces } from '@nestjs/swagger';

@Controller('hosts/:name/backups/:number/files')
export class BackupsFilesController {
  constructor(private service: BackupsFilesService) {}

  @Get()
  async list(@Param('name') name: string, @Param('number') number: number, @Query('path') path: string) {
    return this.service.list(name, number, path);
  }

  @Get('download')
  @ApiHeader({ name: 'content-type', required: false })
  @ApiProduces('application/zip', 'application/x-binary', 'text/plain')
  async download(
    @Param('name') name: string,
    @Param('number') number: number,
    @Query('path') path: string,
    @Res() res: Response,
    @Headers('content-type') type?: string,
  ) {
    const infos = await this.service.getFileName(name, number, path);

    if (type === 'application/zip' || infos.stats.isDirectory()) {
      const archive = archiver('zip');
      res.attachment(`${basename(infos.filename)}.zip`);
      archive.pipe(res);
      if (infos.stats.isDirectory()) {
        archive.directory(infos.filename, path);
      } else {
        archive.file(infos.filename, { name: basename(infos.filename) });
      }
      archive.finalize();
    } else {
      res.download(infos.filename, basename(infos.filename), { dotfiles: 'allow' });
    }
  }
}
