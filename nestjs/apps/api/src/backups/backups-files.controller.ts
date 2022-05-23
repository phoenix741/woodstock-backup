import {
  ClassSerializerInterceptor,
  Controller,
  Get,
  Headers,
  Param,
  ParseIntPipe,
  Query,
  Res,
  UnsupportedMediaTypeException,
  UseInterceptors,
} from '@nestjs/common';
import { ApiHeader, ApiProduces } from '@nestjs/swagger';
import { FileDescription } from '@woodstock/shared';
import * as archiver from 'archiver';
import { Response } from 'express';
import { BackupsFilesService } from './backups-files.service';

@Controller('hosts/:name/backups/:number/files')
export class BackupsFilesController {
  constructor(private service: BackupsFilesService) {}

  @Get()
  @UseInterceptors(ClassSerializerInterceptor)
  async share(@Param('name') name: string, @Param('number', ParseIntPipe) number: number): Promise<FileDescription[]> {
    return this.service.listShare(name, number);
  }

  @Get()
  @UseInterceptors(ClassSerializerInterceptor)
  async list(
    @Param('name') name: string,
    @Param('number', ParseIntPipe) number: number,
    @Query('sharePath') sharePath: string,
    @Query('path') path: string,
  ): Promise<FileDescription[]> {
    return this.service.list(name, number, sharePath, path);
  }

  @Get('download')
  @ApiHeader({ name: 'content-type', required: false })
  @ApiProduces('application/zip', 'application/x-binary', 'text/plain')
  async download(
    @Param('name') name: string,
    @Param('number') number: number,
    @Query('sharePath') sharePath: string,
    @Query('path') path: string,
    @Res() res: Response,
    @Headers('content-type') type?: string,
  ): Promise<void> {
    let archive: archiver.Archiver;
    switch (type || 'application/zip') {
      case 'application/zip':
        archive = archiver.create('zip');
        break;
      case 'application/x-tar':
        archive = archiver.create('tar');
        break;
      default:
        throw new UnsupportedMediaTypeException(`Unsupported media type: ${type}`);
    }

    res.attachment(`download.zip`);
    archive.pipe(res);

    await this.service.createArchive(archive, name, number, sharePath, path);

    await archive.finalize();
  }
}
