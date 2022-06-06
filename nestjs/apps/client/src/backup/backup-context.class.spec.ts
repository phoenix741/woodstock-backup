import { NotFoundException } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import {
  FileBrowserService,
  FileReader,
  IndexManifest,
  Manifest,
  ManifestService,
  ProtobufService,
} from '@woodstock/shared';
import { count, from, toArray } from 'ix/asynciterable';
import * as Long from 'long';
import { join } from 'path';
import { BackupContext } from './backup-context.class';

describe('BackupContext', () => {
  let service: BackupContext;
  let manifestService: ManifestService;
  let fileReader: FileReader;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ManifestService, ProtobufService, FileReader, FileBrowserService],
    }).compile();

    manifestService = module.get<ManifestService>(ManifestService);
    fileReader = module.get<FileReader>(FileReader);
    service = new BackupContext(fileReader, manifestService);
  });

  describe('#RefreshCache', () => {
    test('#RefreshCache should create one manifest from scratch ', async () => {
      let cnt = 0;
      jest.spyOn(manifestService, 'deleteManifest').mockResolvedValue(undefined);
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation(async (entries) => {
        cnt = await count(entries);
      });

      function* myIterator() {
        yield { header: { sharePath: Buffer.from('/home') } };
        yield { fileManifest: { path: Buffer.from('/test1'), stats: { size: Long.fromNumber(100) } } };
        yield { fileManifest: { path: Buffer.from('/test2'), stats: { size: Long.fromNumber(200) } } };
        yield { fileManifest: { path: Buffer.from('/test3'), stats: { size: Long.fromNumber(300) } } };
        yield { fileManifest: { path: Buffer.from('/test4'), stats: { size: Long.fromNumber(400) } } };
        yield { fileManifest: { path: Buffer.from('/test5'), stats: { size: Long.fromNumber(500) } } };
        yield { fileManifest: { path: Buffer.from('/test6'), stats: { size: Long.fromNumber(600) } } };
      }

      const reply = await service.refreshCache(from(myIterator()));

      expect(manifestService.deleteManifest).toHaveBeenCalledTimes(1);
      expect(manifestService.deleteManifest).toHaveBeenCalledWith(new Manifest('backups.%2Fhome', '/tmp'));
      expect(cnt).toBe(6);
      expect(reply).toMatchSnapshot('RefreshCacheReply');
    });

    test('#RefreshCache should create two manifest from scratch ', async () => {
      const cnt: Record<string, number> = {};
      jest.spyOn(manifestService, 'deleteManifest').mockResolvedValue(undefined);
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation(async (entries, manifest: Manifest) => {
        cnt[manifest.journalPath] = await count(entries);
      });

      function* myIterator() {
        yield { header: { sharePath: Buffer.from('/home') } };
        yield { fileManifest: { path: Buffer.from('/test1'), stats: { size: Long.fromNumber(100) } } };
        yield { fileManifest: { path: Buffer.from('/test2'), stats: { size: Long.fromNumber(200) } } };
        yield { fileManifest: { path: Buffer.from('/test3'), stats: { size: Long.fromNumber(300) } } };
        yield { fileManifest: { path: Buffer.from('/test4'), stats: { size: Long.fromNumber(400) } } };
        yield { fileManifest: { path: Buffer.from('/test5'), stats: { size: Long.fromNumber(500) } } };
        yield { fileManifest: { path: Buffer.from('/test6'), stats: { size: Long.fromNumber(600) } } };
        yield {
          header: { sharePath: Buffer.from('/etc') },
          fileManifest: { path: Buffer.from('/test32'), stats: { size: Long.fromNumber(100) } },
        };
        yield { fileManifest: { path: Buffer.from('/test7'), stats: { size: Long.fromNumber(100) } } };
        yield { fileManifest: { path: Buffer.from('/test8'), stats: { size: Long.fromNumber(200) } } };
        yield { fileManifest: { path: Buffer.from('/test9'), stats: { size: Long.fromNumber(300) } } };
      }

      const reply = await service.refreshCache(from(myIterator()));

      expect(manifestService.deleteManifest).toHaveBeenCalledTimes(2);
      expect(manifestService.deleteManifest).toHaveBeenCalledWith(new Manifest('backups.%2Fhome', '/tmp'));
      expect(cnt).toMatchInlineSnapshot(`
              Object {
                "/tmp/backups.%2Fetc.journal": 4,
                "/tmp/backups.%2Fhome.journal": 6,
              }
          `);
      expect(reply).toMatchSnapshot('RefreshCacheReply');
    });

    test('#RefreshCache should create error', async () => {
      const cnt: Record<string, number> = {};
      jest.spyOn(manifestService, 'deleteManifest').mockResolvedValue(undefined);
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation(async (entries, manifest: Manifest) => {
        cnt[manifest.journalPath] = await count(entries);
      });

      function* myIterator() {
        yield { header: { sharePath: Buffer.from('/home') } };
        yield { fileManifest: { path: Buffer.from('/test1'), stats: { size: Long.fromNumber(100) } } };
        throw new NotFoundException('Should be return error');
      }

      const reply = await service.refreshCache(from(myIterator()));

      expect(manifestService.deleteManifest).toHaveBeenCalledTimes(1);
      expect(manifestService.deleteManifest).toHaveBeenCalledWith(new Manifest('backups.%2Fhome', '/tmp'));
      expect(cnt).toMatchInlineSnapshot(`Object {}`);
      expect(reply).toMatchSnapshot('RefreshCacheReply');
    });
  });

  describe('LaunchBackup', () => {
    let mockIndex: IndexManifest;

    beforeEach(() => {
      mockIndex = new IndexManifest();
      jest.spyOn(manifestService, 'loadIndex').mockImplementation(async (manifest: Manifest) => {
        expect(manifest.manifestPath).toBe('/tmp/backups.%2Fhome%2Fphoenix%2FDownloads%2Ftest.manifest');
        return mockIndex;
      });
      jest.spyOn(fileReader, 'getFiles').mockImplementation((index: IndexManifest) =>
        from(
          (function* fileReaderGetFiles(index: IndexManifest) {
            expect(index).toBe(mockIndex);
            const entry = index.getEntry(Buffer.from('/42093_X_Hot Rod.pdf'));
            if (entry) {
              entry.markViewed = true;
            }
            yield {
              chunks: [
                Buffer.from('782199a8de21390a804df2db8cc308c4d8966c10e181a252f96af35c51e9682f', 'hex'),
                Buffer.from('4a8602e2bc9f3eb9f744da31b0e1abf5474023e94a8bba97ecf544f578372fff', 'hex'),
              ],
              path: Buffer.from('/42093_X_Hot Rod.pdf'),
              sha256: Buffer.from('46b475c703c639f21c725daf1eed193ab1ea66a069028b512e0bca60533e037c', 'hex'),
              stats: {
                created: new Long(1271037455, 376),
                groupId: new Long(1000, 0),
                lastModified: new Long(1656647184, 371),
                lastRead: new Long(1063485888, 372),
                mode: new Long(33188, 0),
                ownerId: new Long(1000, 0),
                size: new Long(4503209, 0),
              },
            };
          })(index),
        ),
      );
    });

    test('should make backup', async () => {
      const share = {
        sharePath: Buffer.from('/home/phoenix/Downloads/test'),
        includes: [Buffer.from('*.pdf')],
        excludes: [Buffer.from('*.bak')],
      };
      const it = await toArray(service.launchBackup(share));
      expect(it).toMatchSnapshot('Launch backup (normal)');
    });

    test('should make backup with index (remove elements)', async () => {
      mockIndex.add({
        path: Buffer.from('/42093_X_Hot Rod.pdf'),
        stats: {},
        acl: [],
        xattr: {},
        chunks: [],
      });
      mockIndex.add({
        path: Buffer.from('/fichier supprimé.pdf'),
        stats: {},
        acl: [],
        xattr: {},
        chunks: [],
      });
      const share = {
        sharePath: Buffer.from('/home/phoenix/Downloads/test'),
        includes: [],
        excludes: [],
      };
      const it = await toArray(service.launchBackup(share));
      expect(it).toMatchSnapshot('Launch backup (normal)');
    });

    test('but return an error', async () => {
      jest.spyOn(manifestService, 'loadIndex').mockRejectedValue(new Error("Can't read files"));

      const share = {
        sharePath: Buffer.from('/home/phoenix/Downloads/test'),
        includes: [],
        excludes: [],
      };
      const it = await toArray(service.launchBackup(share));
      expect(it).toMatchSnapshot('Launch backup (normal)');
    });
  });

  describe('#getChunk', () => {
    test('download a file', async () => {
      const testFile = join(__dirname, 'backup-context.class.ts');
      const chunks = service.getChunk({
        filename: Buffer.from(testFile),
        position: Long.fromNumber(0),
        size: Long.fromNumber(100000),
        sha256: Buffer.from('922d7c9eec991ad5eee982292fcb942715be283e4f5f2e18453602f11832ec7f', 'hex'),
        failIfWrongHash: true,
      });

      const allChunks = await toArray(chunks);

      expect(allChunks.map((c) => c.data.toString())).toMatchSnapshot('getChunk');
    });

    test('download a file, wrong hash', async () => {
      const testFile = join(__dirname, 'backup-context.class.ts');
      const chunks = service.getChunk({
        filename: Buffer.from(testFile),
        position: Long.fromNumber(0),
        size: Long.fromNumber(100000),
        sha256: Buffer.from('13', 'hex'),
        failIfWrongHash: true,
      });

      await expect(toArray(chunks)).rejects.toThrow(/The chunk .* should have a sha of 13, but is .*/);
    });

    test('download a file, piece of file', async () => {
      const testFile = join(__dirname, 'backup-context.class.ts');
      const chunks = service.getChunk({
        filename: Buffer.from(testFile),
        position: Long.fromNumber(10),
        size: Long.fromNumber(30),
        sha256: Buffer.from('bbc1eaf9e6f958bfb798e0c572d295ed53b5d4ab9fa7cc2e0d8c71efdc8b090a', 'hex'),
        failIfWrongHash: true,
      });

      const allChunks = await toArray(chunks);

      expect(allChunks.map((c) => c.data.toString())).toMatchSnapshot('getChunk');
    });
  });
});
