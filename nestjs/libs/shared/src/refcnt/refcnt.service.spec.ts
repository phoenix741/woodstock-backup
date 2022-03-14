import { cp, unlink } from 'fs/promises';
import { count, from, toArray } from 'ix/asynciterable';
import { join } from 'path';
import { ProtoFileManifestJournalEntry } from '../models/object-proto.model';
import { FileManifestJournalEntry } from '../models/woodstock';
import { ProtobufService } from '../services/protobuf.service';
import { ReferenceCount, ReferenceCountFileTypeEnum } from './refcnt.model';
import { RefCntService } from './refcnt.service';

describe('RefCntService', () => {
  let service: RefCntService;

  beforeEach(() => {
    const protobufService = new ProtobufService();
    service = new RefCntService(protobufService);
  });

  it('read a file list/journal and generate a refcount file', async () => {
    const journalFilename = join(__dirname, 'fixtures', 'home.filelist');
    const refcntFilename = join(__dirname, 'REFCNT');

    async function* it() {
      const protobufService = new ProtobufService();
      const it = protobufService.loadFile<FileManifestJournalEntry>(journalFilename, ProtoFileManifestJournalEntry);

      for await (const manifest of it) {
        for (const chunk of manifest.message?.manifest?.chunks || []) {
          yield {
            sha256: chunk,
            refCount: 1,
            size: 4_000_000,
            compressedSize: 4_000_000,
          };
        }
      }
    }

    try {
      await service.writeRefCnt(from(it()), refcntFilename);

      const source = service.readRefCnt(refcntFilename);
      const array = await toArray(source);
      expect(array.length).toBe(2_347_832);
      // expect(array).toMatchSnapshot('refcnt');
    } finally {
      await unlink(refcntFilename);
    }
  }, 120_000);

  it('compact', async () => {
    const refcntFilename = join(__dirname, 'fixtures', 'REFCNT');
    const compactDir = join(__dirname, 'compact');
    const cnt = new ReferenceCount(compactDir, compactDir, compactDir);
    try {
      await cp(refcntFilename, cnt.journalPath);

      expect(await count(service.readRefCnt(cnt.journalPath))).toBe(2_347_832);

      await service.compactRefCnt(cnt);

      for (const [type, path] of Object.entries(cnt.getPaths())) {
        if (type !== ReferenceCountFileTypeEnum.JOURNAL) {
          const source = service.readRefCnt(path);
          const array = await toArray(source);
          expect(array.length).toBe(1_169_737);
        }
      }
      // expect(array).toMatchSnapshot('compact');
    } finally {
      await unlink(cnt.backupPath).catch(() => void 0);
      await unlink(cnt.poolPath).catch(() => void 0);
      await unlink(cnt.hostPath).catch(() => void 0);
    }
  }, 240_000);
});
