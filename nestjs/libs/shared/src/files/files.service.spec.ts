import { Test, TestingModule } from '@nestjs/testing';
import archiver from 'archiver';
import { constants as constantsFs } from 'fs';
import { count, toArray } from 'ix/asynciterable';
import { FilesService } from './files.service';

import { CoreBackupsService, CoreFilesService, JsFileManifest } from '@woodstock/shared-rs';

describe('FilesService', () => {
  let service: FilesService;

  const mockBackupsService = {
    getBackupSharePaths: jest.fn(),
  };

  const mockFilesService = {
    list: jest.fn(),
    readManifest: jest.fn(),
  };

  const MANIFEST = [
    { path: Buffer.from('d0/ax'), stats: { mode: constantsFs.S_IFDIR } },
    { path: Buffer.from('d0'), stats: { mode: constantsFs.S_IFDIR } },
    { path: Buffer.from('d1/dd1/ddd1'), stats: { mode: constantsFs.S_IFDIR } },
    { path: Buffer.from('d1/dd1/ddd2'), stats: { mode: constantsFs.S_IFREG } },
    { path: Buffer.from('d1/dd1/ddd3'), stats: { mode: constantsFs.S_IFREG } },
    { path: Buffer.from('d1/dd2/ddd1'), stats: { mode: constantsFs.S_IFDIR } },
    { path: Buffer.from('d1/dd2/ddd1/abcd'), stats: { mode: constantsFs.S_IFREG } },
    { path: Buffer.from('d1/dd2/ddd1/efgh'), stats: { mode: constantsFs.S_IFREG } },
    {
      path: Buffer.from('d1/dd2/ddd1/ijkl'),
      symlink: Buffer.from('symlink'),
      stats: { mode: constantsFs.S_IFLNK },
    },
    { path: Buffer.from('d1/dd2/ddd1/mnop'), stats: { mode: constantsFs.S_IFREG } },
    { path: Buffer.from('d1/dd2/ddd2'), stats: { mode: constantsFs.S_IFDIR } },
    { path: Buffer.from('d1/dd2/ddd3'), stats: { mode: constantsFs.S_IFREG } },
    { path: Buffer.from('d2/dd1/ddd1'), stats: { mode: constantsFs.S_IFDIR } },
    { path: Buffer.from('d2/dd1/ddd1'), stats: { mode: constantsFs.S_IFREG } },
    { path: Buffer.from('d3/dd1/ddd1'), stats: { mode: constantsFs.S_IFDIR } },
  ] as JsFileManifest[];

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        FilesService,
        { provide: CoreBackupsService, useValue: mockBackupsService },
        { provide: CoreFilesService, useValue: mockFilesService },
      ],
    }).compile();

    service = module.get<FilesService>(FilesService);
  });

  describe('listShares', () => {
    it('list share from a directory', async () => {
      // GIVEN
      mockBackupsService.getBackupSharePaths = jest.fn(() => ['file1', 'file3']);

      // WHEN
      const res = await service.listShares('hostname', 1);

      // THEN
      expect(res).toMatchSnapshot('res');
      expect(mockBackupsService.getBackupSharePaths).toHaveBeenCalledWith('hostname', 1);
    });
  });

  describe('searchFiles', () => {
    it('search files in directory', async () => {
      // GIVEN
      mockFilesService.list = jest.fn((h, b, s, p, r, cb) => {
        MANIFEST.forEach((m) => cb(null, m));
        cb(null, null);
      });

      // WHEN
      const res = await toArray(service.searchFiles('hostname', 1, 'sharePath', Buffer.from('d1/dd2')));

      // THEN
      expect(res.map((v) => ({ ...v, path: v.path.toString('utf-8') }))).toMatchSnapshot('res');
      expect(mockFilesService.list).toHaveBeenCalledWith(
        'hostname',
        1,
        'sharePath',
        Buffer.from('d1/dd2'),
        false,
        expect.any(Function),
      );
    });
  });

  describe('readFileStream', () => {
    it('read file stream is special file', async () => {
      // GIVEN
      mockFilesService.readManifest = jest.fn((m, cb) => {
        cb(null, null);
      });

      // WHEN
      const res = service.readFileStream({
        path: Buffer.from('d1/dd2/ddd1/abcd'),
        stats: {
          mode: constantsFs.S_IFIFO,
          ownerId: 0,
          groupId: 0,
          size: 0n,
          compressedSize: 0n,
          lastModified: 0,
          lastRead: 0,
          created: 0,
          type: 0,
          rdev: 0n,
          dev: 0n,
          ino: 0n,
          nlink: 0n,
        },
        xattr: [],
        acl: [],
        chunks: [],
        symlink: Buffer.from(''),
        hash: Buffer.from(''),
        metadata: {},
      });

      // THEN
      expect(await count(res)).toBe(0);
    });

    it('read file stream regular file', async () => {
      // GIVEN
      mockFilesService.readManifest = jest.fn((m: JsFileManifest, cb) => {
        m.chunks.forEach((c) => cb(null, c));
        cb(null, null);
      });

      // WHEN
      const res = service.readFileStream({
        path: Buffer.from('d1/dd2/ddd1/abcd'),
        stats: {
          mode: constantsFs.S_IFREG,
          ownerId: 0,
          groupId: 0,
          size: 0n,
          compressedSize: 0n,
          lastModified: 0,
          lastRead: 0,
          created: 0,
          type: 0,
          rdev: 0n,
          dev: 0n,
          ino: 0n,
          nlink: 0n,
        },
        xattr: [],
        acl: [],
        chunks: [
          Buffer.from('chunks1'),
          Buffer.from('chunks2'),
          Buffer.from('chunks3'),
          Buffer.from('chunks4'),
          Buffer.from('chunks5'),
        ],
        symlink: Buffer.from(''),
        hash: Buffer.from(''),
        metadata: {},
      });

      // THEN
      for await (const chunk of res) {
        expect(chunk.toString()).toMatchSnapshot('chunk');
      }
    });
  });

  describe('createArchive', () => {
    let originalReadFileStream: FilesService['readFileStream'];

    beforeEach(() => {
      originalReadFileStream = service.readFileStream;
    });

    afterEach(() => {
      service.readFileStream = originalReadFileStream;
    });

    it('create archive', async () => {
      // GIVEN
      mockFilesService.list = jest.fn((h, b, s, p, r, cb) => {
        MANIFEST.forEach((m) => cb(null, m));
        cb(null, null);
      });
      const mockedArchiverInstance = {
        append: jest.fn(),
        symlink: jest.fn(),
      } as unknown as archiver.Archiver;
      service.readFileStream = jest.fn().mockImplementation((m) => m.path.toString());

      // WHEN
      await service.createArchive(mockedArchiverInstance, 'hostname', 1, 'sharePath', Buffer.from('d1/dd2'));

      // THEN
      expect(mockedArchiverInstance.append).toMatchSnapshot('append');
      expect(mockedArchiverInstance.symlink).toMatchSnapshot('symlink');
      expect(service.readFileStream).toMatchSnapshot('readFileStream');
    });
  });
});
