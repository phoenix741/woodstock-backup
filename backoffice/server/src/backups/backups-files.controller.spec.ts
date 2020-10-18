import { Test, TestingModule } from '@nestjs/testing';
import { Response } from 'express';

import { BackupsFilesController } from './backups-files.controller';
import { BackupsFilesService } from './backups-files.service';
import * as archiver from 'archiver';

jest.mock('archiver');

const mockedArchiver = archiver as jest.Mocked<typeof archiver>;

describe('Backups File Controller', () => {
  let controller: BackupsFilesController;

  const backupsFilesService = {
    listShare: (name: string, number: number) => [
      {
        name,
        dev: number,
      },
    ],
    list: (name: string, number: number, share: string, path: string) => [
      { name, dev: number },
      { name: share },
      { name: path },
    ],
    getFileName: (name: string, number: number, share: string, path: string) => ({
      filename: `${name}/${number}/${share}/${path}`,
      stats: {
        isDirectory() {
          return path === 'directory';
        },
      },
    }),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [{ provide: BackupsFilesService, useValue: backupsFilesService }],
      controllers: [BackupsFilesController],
    }).compile();

    controller = module.get<BackupsFilesController>(BackupsFilesController);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  it('should list available share', async () => {
    expect(await controller.share('name', 10)).toEqual([{ name: 'name', dev: 10 }]);
  });

  it('should list the available share of a path', async () => {
    expect(await controller.list('name', 10, 'share', 'path')).toEqual([
      { name: 'name', dev: 10 },
      { name: 'share' },
      { name: 'path' },
    ]);
  });

  it('should download a file with the name', async () => {
    // GIVEN
    const res = ({
      download: jest.fn(),
    } as unknown) as Response;

    // WHEN
    await controller.download('name', 10, 'share', 'path', res);

    // THEN
    expect(res.download).toHaveBeenCalledWith('name/10/share/path', 'path', { dotfiles: 'allow' });
  });

  it('should download a zip file with the file name', async () => {
    // GIVEN
    const res = ({
      attachment: jest.fn(),
    } as unknown) as Response;

    const mockedArchiverInstance = ({
      pipe: jest.fn(),
      file: jest.fn(),
      finalize: jest.fn(),
    } as unknown) as archiver.Archiver;

    mockedArchiver.create.mockReturnValue(mockedArchiverInstance);

    // WHEN
    await controller.download('name', 10, 'share', 'path', res, 'application/zip');

    // THEN
    expect(res.attachment).toHaveBeenCalledWith('path.zip');
    expect(mockedArchiverInstance.pipe).toHaveBeenCalledWith(res);
    expect(mockedArchiverInstance.file).toHaveBeenCalledWith('name/10/share/path', { name: 'path' });
    expect(mockedArchiverInstance.finalize).toHaveBeenCalledWith();
  });

  it('should download a zip file with the directory', async () => {
    // GIVEN
    const res = ({
      attachment: jest.fn(),
    } as unknown) as Response;

    const mockedArchiverInstance = ({
      pipe: jest.fn(),
      directory: jest.fn(),
      finalize: jest.fn(),
    } as unknown) as archiver.Archiver;

    mockedArchiver.create.mockReturnValue(mockedArchiverInstance);

    // WHEN
    await controller.download('name', 10, 'share', 'directory', res);

    // THEN
    expect(res.attachment).toHaveBeenCalledWith('directory.zip');
    expect(mockedArchiverInstance.pipe).toHaveBeenCalledWith(res);
    expect(mockedArchiverInstance.directory).toHaveBeenCalledWith('name/10/share/directory', 'directory');
    expect(mockedArchiverInstance.finalize).toHaveBeenCalledWith();
  });
});
