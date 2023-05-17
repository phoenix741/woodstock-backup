import { Test, TestingModule } from '@nestjs/testing';
import archiver from 'archiver';
import { constants as constantsFs } from 'fs';
import { count, from, toArray } from 'ix/asynciterable';
import * as Long from 'long';
import { Readable } from 'stream';
import { BackupsService } from '../../../core/src/config';
import { ManifestService } from '../manifest';
import { PoolService } from '../../../server/src/pool';
import { FileBrowserService } from '../scanner';
import { FilesService } from './files.service.js';

describe('FilesService', () => {
  let service: FilesService;

  const mockBackupsService = {
    getDestinationDirectory: jest.fn(),
    getManifest: jest.fn(),
  };

  const mockFileBrowserService = {
    getFilesFromDirectory: jest.fn(),
  };

  const mockManifestService = {
    readManifestEntries: jest.fn(),
  };

  const mockPoolService = {
    getChunk: jest.fn(),
  };

  const MANIFEST = [
    { path: Buffer.from('d0/ax'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
    { path: Buffer.from('d0'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
    { path: Buffer.from('d1/dd1/ddd1'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
    { path: Buffer.from('d1/dd1/ddd2'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    { path: Buffer.from('d1/dd1/ddd3'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    { path: Buffer.from('d1/dd2/ddd1'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
    { path: Buffer.from('d1/dd2/ddd1/abcd'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    { path: Buffer.from('d1/dd2/ddd1/efgh'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    {
      path: Buffer.from('d1/dd2/ddd1/ijkl'),
      symlink: Buffer.from('symlink'),
      stats: { mode: Long.fromNumber(constantsFs.S_IFLNK) },
    },
    { path: Buffer.from('d1/dd2/ddd1/mnop'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    { path: Buffer.from('d1/dd2/ddd2'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
    { path: Buffer.from('d1/dd2/ddd3'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    { path: Buffer.from('d2/dd1/ddd1'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
    { path: Buffer.from('d2/dd1/ddd1'), stats: { mode: Long.fromNumber(constantsFs.S_IFREG) } },
    { path: Buffer.from('d3/dd1/ddd1'), stats: { mode: Long.fromNumber(constantsFs.S_IFDIR) } },
  ];

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        FilesService,
        { provide: BackupsService, useValue: mockBackupsService },
        { provide: FileBrowserService, useValue: mockFileBrowserService },
        { provide: ManifestService, useValue: mockManifestService },
        { provide: PoolService, useValue: mockPoolService },
      ],
    }).compile();

    service = module.get<FilesService>(FilesService);
  });

  describe('listShares', () => {
    it('list share from a directory', async () => {
      // GIVEN
      mockBackupsService.getDestinationDirectory = jest.fn(() => 'destinationDirectory');
      mockFileBrowserService.getFilesFromDirectory = jest.fn(() =>
        from([
          { name: Buffer.from('file1.manifest') },
          { name: Buffer.from('file2.journal') },
          { name: Buffer.from('file3.manifest') },
          { name: Buffer.from('file4.filelist') },
        ]),
      );

      // WHEN
      const res = await toArray(service.listShares('hostname', 1));

      // THEN
      expect(res.map((v) => v.toString('utf-8'))).toMatchSnapshot('res');
      expect(mockBackupsService.getDestinationDirectory).toHaveBeenCalledWith('hostname', 1);
      expect(mockFileBrowserService.getFilesFromDirectory).toHaveBeenCalledWith(Buffer.from('destinationDirectory'));
    });
  });

  describe('searchFiles', () => {
    it('search files in directory', async () => {
      // GIVEN
      mockBackupsService.getManifest = jest.fn(() => 'destinationDirectory');
      mockManifestService.readManifestEntries = jest.fn(() => from(MANIFEST));

      // WHEN
      const res = await toArray(service.searchFiles('hostname', 1, Buffer.from('sharePath'), Buffer.from('d1/dd2')));

      // THEN
      expect(res.map((v) => ({ ...v, path: v.path.toString('utf-8') }))).toMatchSnapshot('res');
    });
  });

  describe('readFileStream', () => {
    it('read file stream is special file', async () => {
      // GIVEN

      // WHEN
      const res = service.readFileStream({
        path: Buffer.from('d1/dd2/ddd1/abcd'),
        stats: { mode: Long.fromNumber(constantsFs.S_IFIFO) },
        xattr: {},
        acl: [],
        chunks: [],
      });

      // THEN
      expect(await count(res)).toBe(0);
    });

    it('read file stream regular file', async () => {
      // GIVEN
      mockPoolService.getChunk = jest.fn((chunk) => ({
        read: () => Readable.from(chunk),
      }));

      // WHEN
      const res = service.readFileStream({
        path: Buffer.from('d1/dd2/ddd1/abcd'),
        stats: { mode: Long.fromNumber(constantsFs.S_IFREG) },
        xattr: {},
        acl: [],
        chunks: [
          Buffer.from('chunks1'),
          Buffer.from('chunks2'),
          Buffer.from('chunks3'),
          Buffer.from('chunks4'),
          Buffer.from('chunks5'),
        ],
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
      mockManifestService.readManifestEntries = jest.fn(() => from(MANIFEST));
      const mockedArchiverInstance = {
        append: jest.fn(),
        symlink: jest.fn(),
      } as unknown as archiver.Archiver;
      service.readFileStream = jest.fn().mockImplementation((m) => m.path.toString());

      // WHEN
      await service.createArchive(
        mockedArchiverInstance,
        'hostname',
        1,
        Buffer.from('sharePath'),
        Buffer.from('d1/dd2'),
      );

      // THEN
      expect(mockedArchiverInstance.append).toMatchSnapshot('append');
      expect(mockedArchiverInstance.symlink).toMatchSnapshot('symlink');
      expect(service.readFileStream).toMatchSnapshot('readFileStream');
    });
  });
});
