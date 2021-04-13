import { Test, TestingModule } from '@nestjs/testing';
import * as mock from 'mock-fs';

import { SharePathService } from '../utils/share-path.service';
import { BackupsFilesService } from './backups-files.service';
import { BackupsService } from './backups.service';

describe('Backups File Service', () => {
  let service: BackupsFilesService;

  const mockBackupsService = {
    getDestinationDirectory: (name: string, number: number) => `destinationDirectory/${name}/${number}`,
  };
  const mockSharePathService = {
    unmangle: (name: string) => name,
    mangle: (name: string) => name,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        { provide: BackupsService, useValue: mockBackupsService },
        { provide: SharePathService, useValue: mockSharePathService },
        BackupsFilesService,
      ],
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
    expect(listShare[0]).toMatchInlineSnapshot(
      {
        ino: expect.any(Number),
      },
      `
      Object {
        "atime": 1970-01-01T00:00:00.001Z,
        "atimeMs": 1,
        "birthtime": 1970-01-01T00:00:00.001Z,
        "birthtimeMs": 1,
        "blksize": 4096,
        "blocks": 1,
        "ctime": 1970-01-01T00:00:00.001Z,
        "ctimeMs": 1,
        "dev": 8675309,
        "gid": 0,
        "ino": Any<Number>,
        "mode": 16895,
        "mtime": 1970-01-01T00:00:00.001Z,
        "mtimeMs": 1,
        "name": "dir1",
        "nlink": 2,
        "rdev": 0,
        "size": 1,
        "type": "DIRECTORY",
        "uid": 0,
      }
    `,
    );

    expect(listShare[1]).toMatchInlineSnapshot(
      {
        ino: expect.any(Number),
      },
      `
      Object {
        "atime": 1970-01-01T00:00:00.001Z,
        "atimeMs": 1,
        "birthtime": 1970-01-01T00:00:00.001Z,
        "birthtimeMs": 1,
        "blksize": 4096,
        "blocks": 1,
        "ctime": 1970-01-01T00:00:00.001Z,
        "ctimeMs": 1,
        "dev": 8675309,
        "gid": 0,
        "ino": Any<Number>,
        "mode": 33206,
        "mtime": 1970-01-01T00:00:00.001Z,
        "mtimeMs": 1,
        "name": "file1",
        "nlink": 1,
        "rdev": 0,
        "size": 17,
        "type": "REGULAR_FILE",
        "uid": 0,
      }
    `,
    );
  });

  it('should list all files of a directory', async () => {
    // GIVEN
    mock({
      'destinationDirectory/name/10/share/toto': {
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
    expect(listFiles[0]).toMatchInlineSnapshot(
      {
        ino: expect.any(Number),
      },
      `
      Object {
        "atime": 1970-01-01T00:00:00.001Z,
        "atimeMs": 1,
        "birthtime": 1970-01-01T00:00:00.001Z,
        "birthtimeMs": 1,
        "blksize": 4096,
        "blocks": 1,
        "ctime": 1970-01-01T00:00:00.001Z,
        "ctimeMs": 1,
        "dev": 8675309,
        "gid": 0,
        "ino": Any<Number>,
        "mode": 16895,
        "mtime": 1970-01-01T00:00:00.001Z,
        "mtimeMs": 1,
        "name": "dir1",
        "nlink": 2,
        "rdev": 0,
        "size": 1,
        "type": "DIRECTORY",
        "uid": 0,
      }
    `,
    );

    expect(listFiles[1]).toMatchInlineSnapshot(
      {
        ino: expect.any(Number),
      },
      `
      Object {
        "atime": 1970-01-01T00:00:00.001Z,
        "atimeMs": 1,
        "birthtime": 1970-01-01T00:00:00.001Z,
        "birthtimeMs": 1,
        "blksize": 4096,
        "blocks": 1,
        "ctime": 1970-01-01T00:00:00.001Z,
        "ctimeMs": 1,
        "dev": 8675309,
        "gid": 0,
        "ino": Any<Number>,
        "mode": 33206,
        "mtime": 1970-01-01T00:00:00.001Z,
        "mtimeMs": 1,
        "name": "file1",
        "nlink": 1,
        "rdev": 0,
        "size": 17,
        "type": "REGULAR_FILE",
        "uid": 0,
      }
    `,
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
      'destinationDirectory/name/10/share/toto': {
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
    expect(infos).toMatchInlineSnapshot(
      {
        stats: {
          ino: expect.any(Number),
        },
      },
      `
      Object {
        "filename": "destinationDirectory/name/10/share/toto/file1",
        "stats": Object {
          "atime": 1970-01-01T00:00:00.001Z,
          "atimeMs": 1,
          "birthtime": 1970-01-01T00:00:00.001Z,
          "birthtimeMs": 1,
          "blksize": 4096,
          "blocks": 1,
          "ctime": 1970-01-01T00:00:00.001Z,
          "ctimeMs": 1,
          "dev": 8675309,
          "gid": 0,
          "ino": Any<Number>,
          "mode": 33206,
          "mtime": 1970-01-01T00:00:00.001Z,
          "mtimeMs": 1,
          "nlink": 1,
          "rdev": 0,
          "size": 17,
          "uid": 0,
        },
      }
    `,
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
