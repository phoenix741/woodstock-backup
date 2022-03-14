import { Test, TestingModule } from '@nestjs/testing';
import { BackupsService } from '@woodstock/backoffice-shared';
import * as mock from 'mock-fs';
import { BackupsFilesService } from './backups-files.service';

describe('Backups File Service', () => {
  let service: BackupsFilesService;

  const mockBackupsService = {
    getDestinationDirectory: (name: string, number: number) => `destinationDirectory/${name}/${number}`,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [{ provide: BackupsService, useValue: mockBackupsService }, BackupsFilesService],
    }).compile();

    service = module.get<BackupsFilesService>(BackupsFilesService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should list all share of a directory', async () => {
    // GIVEN
    mock({
      'destinationDirectory/name/10': {
        file1: mock.file({
          content: 'file content here',
          atime: new Date(1),
          ctime: new Date(1),
          mtime: new Date(1),
          birthtime: new Date(1),
          uid: 0,
          gid: 0,
        }),
        dir1: mock.directory({
          atime: new Date(1),
          ctime: new Date(1),
          mtime: new Date(1),
          birthtime: new Date(1),
          uid: 0,
          gid: 0,
          items: {
            /** empty directory */
          },
        }),
      },
    });

    // WHEN
    const listShare = await service.listShare('name', 10);

    // THEN
    expect(listShare[0]).toMatchSnapshot(
      {
        ino: expect.any(Number),
      },
      'listShare[0]',
    );

    expect(listShare[1]).toMatchSnapshot(
      {
        ino: expect.any(Number),
      },
      'listShare[1]',
    );
  });

  it('should list all files of a directory', async () => {
    // GIVEN
    mock({
      'destinationDirectory/name/10/%2Fshare/toto': {
        file1: mock.file({
          content: 'file content here',
          atime: new Date(1),
          ctime: new Date(1),
          mtime: new Date(1),
          birthtime: new Date(1),
          uid: 0,
          gid: 0,
        }),
        dir1: mock.directory({
          atime: new Date(1),
          ctime: new Date(1),
          mtime: new Date(1),
          birthtime: new Date(1),
          uid: 0,
          gid: 0,
          items: {
            /** empty directory */
          },
        }),
      },
    });

    // WHEN
    const listFiles = await service.list('name', 10, '/share', '/toto');

    // THEN
    expect(listFiles[0]).toMatchSnapshot(
      {
        ino: expect.any(Number),
      },
      'listFiles[0]',
    );

    expect(listFiles[1]).toMatchSnapshot(
      {
        ino: expect.any(Number),
      },
      'listFiles[1]',
    );
  });

  it('should throw exception when list of file is not absolute ', async () => {
    expect.assertions(1);
    await expect(service.list('name', 10, '/share', 'toto')).rejects.toThrow(
      'Only absolute path can be used to serach for directory',
    );
  });

  it('should throw exception when list of share is not absolute ', async () => {
    expect.assertions(1);
    await expect(service.list('name', 10, 'share', '/toto')).rejects.toThrow(
      'Only absolute path can be used to serach for directory',
    );
  });

  it('should get the file stat of a filename', async () => {
    // GIVEN
    mock({
      'destinationDirectory/name/10/%2Fshare/toto': {
        file1: mock.file({
          content: 'file content here',
          atime: new Date(1),
          ctime: new Date(1),
          mtime: new Date(1),
          birthtime: new Date(1),
          uid: 0,
          gid: 0,
        }),
      },
    });

    // WHEN
    const infos = await service.getFileName('name', 10, '/share', '/toto/file1');

    // THEN
    expect(infos).toMatchSnapshot(
      {
        stats: {
          ino: expect.any(Number),
        },
      },
      'service.getFileName',
    );
  });

  it('should throw exception when list of file is not absolute ', async () => {
    expect.assertions(1);
    await expect(service.list('name', 10, '/share', 'toto')).rejects.toThrow(
      'Only absolute path can be used to serach for directory',
    );
  });

  it('should throw exception when list of share is not absolute ', async () => {
    expect.assertions(1);
    await expect(service.list('name', 10, 'share', '/toto')).rejects.toThrow(
      'Only absolute path can be used to serach for directory',
    );
  });

  afterEach(() => {
    mock.restore();
  });
});
