import { Type } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import { cp } from 'fs/promises';
import { count, from, toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import { join } from 'path';
import { ProtoFileManifestJournalEntry } from '../shared/woodstock.model.js';
import { FileManifestJournalEntry } from '../shared';
import { ProtobufMessageWithPosition } from '../input-output';
import { ProtobufService } from '../input-output/protobuf.service.js';
import { PoolStatisticsService } from '../statistics';
import { ReferenceCount } from './refcnt.interface.js';
import { RefCntService } from './refcnt.service.js';

async function atomicToSnapshot(args: [string, Type<any>, AsyncIterable<unknown>, boolean]) {
  return {
    filename: args[0],
    type: args[1],
    source: await toArray(
      from(args[2]).pipe(map((v: Record<string, object>) => ({ ...v, sha256: v.sha256.toString() }))),
    ),
    append: args[3],
  };
}

describe('RefCntService', () => {
  let service: RefCntService;

  const statsService = {
    writeStatistics: jest.fn<Promise<void>, unknown[]>(),
  };

  const protobufService = {
    loadFile<T>(filename: string, type: Type): AsyncIterable<ProtobufMessageWithPosition<T>> {
      filename;
      type;
      return from([]);
    },
    async writeFile<O>(path: string, type: Type, source: AsyncIterable<O>, compress: boolean): Promise<void> {
      path;
      type;
      source;
      compress;
    },
    async atomicWriteFile<O>(path: string, type: Type, source: AsyncIterable<O>, compress: boolean): Promise<void> {
      path;
      type;
      source;
      compress;
    },
  };

  beforeEach(async () => {
    statsService.writeStatistics.mockClear();
  });

  describe('With real file', () => {
    beforeEach(async () => {
      const module: TestingModule = await Test.createTestingModule({
        providers: [ProtobufService, RefCntService, { provide: PoolStatisticsService, useValue: statsService }],
      }).compile();

      service = module.get<RefCntService>(RefCntService);
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
        expect(array.length).toBe(1_064);
      } finally {
        await service.deleteRefcnt(refcntFilename);
      }
    }, 120_000);

    it('addBackupRefcntTo', async () => {
      // GIVEN
      const refcntFilename = join(__dirname, 'fixtures', 'REFCNT');
      const compactDir = join(__dirname, 'compact');
      const cnt = new ReferenceCount(compactDir, compactDir, compactDir);
      try {
        await cp(refcntFilename, cnt.backupPath);

        expect(await count(service.readRefCnt(cnt.backupPath))).toBe(1_000);

        // WHEN
        await service.addBackupRefcntTo(cnt.poolPath, cnt.backupPath, cnt.unusedPoolPath);

        // THEN
        const source = service.readRefCnt(cnt.poolPath);
        const array = await toArray(source);
        expect(array.length).toBe(936);
        expect(array.reduce((acc, v) => acc + v.refCount, 0)).toBe(1_000);

        expect(statsService.writeStatistics).toBeCalledTimes(1);
        expect(
          (statsService.writeStatistics as jest.Mock<Promise<void>, [object[], string]>).mock.calls[0][0],
        ).toMatchSnapshot('statsService.writeStatistics');
      } finally {
        await service.deleteRefcnt(cnt.backupPath).catch(() => void 0);
        await service.deleteRefcnt(cnt.poolPath).catch(() => void 0);
      }
    }, 240_000);

    it('cleanup', async () => {
      const refcntFilename = join(__dirname, 'fixtures', 'REFCNT');
      const cleanupDir = join(__dirname, 'cleanup');
      const unusedPath = join(cleanupDir, 'UNUSED');
      const cnt = new ReferenceCount(cleanupDir, cleanupDir, cleanupDir);
      try {
        await cp(refcntFilename, cnt.backupPath);

        await service.removeBackupRefcntTo(cnt.backupPath, refcntFilename, unusedPath);

        const source = service.readRefCnt(cnt.backupPath);
        const array = await toArray(source);
        expect(array.length).toBe(0);

        const unusedSource = service.readUnused(unusedPath);
        const unusedArray = await toArray(unusedSource);
        expect(unusedArray.length).toBe(936);
      } finally {
        await service.deleteRefcnt(cnt.backupPath).catch(() => void 0);
        await service.deleteRefcnt(unusedPath).catch(() => void 0);
      }
    });

    it('cleanup some file', async () => {
      // GIVEN
      const refcntFilename = join(__dirname, 'fixtures', 'pool', 'REFCNT.pool');
      const unusedFilename = join(__dirname, 'fixtures', 'pool', 'REFCNT.unused');
      const backupFilename = join(__dirname, 'fixtures', 'pool', 'REFCNT.backup');

      const cleanupDir = join(__dirname, 'cleanup2');
      const newRefcntPool = join(cleanupDir, 'REFCNT');
      const newUnused = join(cleanupDir, 'UNUSED');

      try {
        await cp(unusedFilename, newUnused);
        await cp(refcntFilename, newRefcntPool);

        const source1 = service.readRefCnt(newRefcntPool);
        const array1 = await toArray(source1);
        expect(array1.length).toBe(518_508);

        const unusedSource1 = service.readUnused(newUnused);
        const unusedArray1 = await toArray(unusedSource1);
        expect(unusedArray1.length).toBe(27_456);

        await service.removeBackupRefcntTo(newRefcntPool, backupFilename, newUnused);

        const source = service.readRefCnt(newRefcntPool);
        const array = await toArray(source);
        expect(array.length).toBe(518_505);

        const unusedSource = service.readUnused(newUnused);
        const unusedArray = await toArray(unusedSource);
        expect(unusedArray.length).toBe(27_459);
      } finally {
        await service.deleteRefcnt(newRefcntPool).catch(() => void 0);
        await service.deleteRefcnt(newUnused).catch(() => void 0);
      }
    }, 240_000);
  });

  describe('With mock', () => {
    beforeEach(async () => {
      const module: TestingModule = await Test.createTestingModule({
        providers: [
          RefCntService,
          { provide: ProtobufService, useValue: protobufService },
          { provide: PoolStatisticsService, useValue: statsService },
        ],
      }).compile();

      service = module.get<RefCntService>(RefCntService);
    });

    it('cleanup some file', async () => {
      // GIVEN
      protobufService.loadFile = jest
        .fn()
        .mockReturnValueOnce(
          from([
            {
              message: {
                sha256: Buffer.from('sha256_1'),
                refCount: 5,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_2'),
                refCount: 5,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_3'),
                refCount: 5,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_10'),
                refCount: 5,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
          ]),
        )
        .mockReturnValueOnce(
          from([
            {
              message: {
                sha256: Buffer.from('sha256_2'),
                refCount: 3,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_3'),
                refCount: 7,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_10'),
                refCount: 5,
                size: 4_000_000,
                compressedSize: 4_000_000,
              },
            },
          ]),
        )
        .mockReturnValueOnce(
          from([
            {
              message: {
                sha256: Buffer.from('sha256_3'),
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_4'),
              },
            },
            {
              message: {
                sha256: Buffer.from('sha256_5'),
              },
            },
          ]),
        );
      protobufService.writeFile = jest.fn();
      protobufService.atomicWriteFile = jest.fn();

      // WHEN
      await service.removeBackupRefcntTo('fileToChange', 'backupPath', 'unusedPath');

      // THEN
      expect(protobufService.atomicWriteFile).toBeCalledTimes(2);
      expect(
        await atomicToSnapshot(
          (protobufService.atomicWriteFile as jest.Mock<Promise<void>, [string, Type, AsyncIterable<unknown>, boolean]>)
            .mock.calls[0],
        ),
      ).toMatchSnapshot('atomicWriteFileUnused');
      expect(
        await atomicToSnapshot(
          (protobufService.atomicWriteFile as jest.Mock<Promise<void>, [string, Type, AsyncIterable<unknown>, boolean]>)
            .mock.calls[1],
        ),
      ).toMatchSnapshot('atomicWriteFileRefCnt');
      expect(statsService.writeStatistics).toBeCalledTimes(1);
      expect(
        (statsService.writeStatistics as jest.Mock<Promise<void>, [object[], string]>).mock.calls[0],
      ).toMatchSnapshot('statsService.writeStatistics');
    });
  });
});
