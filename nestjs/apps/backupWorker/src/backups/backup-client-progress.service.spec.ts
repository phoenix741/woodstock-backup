import { Logger } from '@nestjs/common';
import { ClientGrpcProxy } from '@nestjs/microservices';
import { Test, TestingModule } from '@nestjs/testing';
import { bigIntToLong, EntryType } from '@woodstock/shared';
import { from, lastValueFrom, toArray } from 'rxjs';
import { BackupsGrpcContext } from './backup-client-grpc.class.js';
import { BackupClientProgress } from './backup-client-progress.service.js';
import { BackupClient } from './backup-client.service.js';

describe('BackupClientProgress', () => {
  const mockBackupClient = {
    authenticate: () => 0,
    executeCommand: () => 0,
    getFileList: () => 0,
    createBackup: () => 0,
    refreshCache: () => 0,
    countRef: () => 0,
    close: () => 0,
  };

  const fakeClient = {} as ClientGrpcProxy;

  let backupClientProgress: BackupClientProgress;

  function createChunks(size: bigint) {
    const SPLIT = 75n;
    const chunks = [];
    for (let i = 0n; i < size / SPLIT; i++) {
      chunks.push({
        sha256: Buffer.from('sha256_' + i),
        size: SPLIT,
        compressedSize: 50n,
      });
    }
    chunks.push({
      sha256: Buffer.from('sha256_' + size / SPLIT),
      size: size % SPLIT,
      compressedSize: 50n,
    });
    return chunks;
  }

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [{ provide: BackupClient, useValue: mockBackupClient }, BackupClientProgress],
    }).compile();

    backupClientProgress = module.get<BackupClientProgress>(BackupClientProgress);
  });

  it('#authenticate', async () => {
    // GIVEN
    mockBackupClient.authenticate = jest.fn().mockResolvedValue(undefined);

    const logger = new Logger('FakeLogger');
    const clientLogger = new Logger('FakeClientLogger');
    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    // WHEN
    const observable = backupClientProgress.authenticate(ctxt, logger, clientLogger, 'password');
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
  });

  it('#executeCommand', async () => {
    // GIVEN
    mockBackupClient.executeCommand = jest.fn().mockResolvedValue(undefined);

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    // WHEN
    const observable = backupClientProgress.executeCommand(ctxt, 'command');
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
  });

  it('#getFileList', async () => {
    // GIVEN
    const it = [
      { type: EntryType.ADD, manifest: { path: Buffer.from('file1'), stats: { size: bigIntToLong(100n) } } },
      { type: EntryType.ADD, manifest: { path: Buffer.from('file2'), stats: { size: bigIntToLong(200n) } } },
      { type: EntryType.ADD, manifest: { path: Buffer.from('file3'), stats: { size: bigIntToLong(300n) } } },
      { type: EntryType.ADD, manifest: { path: Buffer.from('file4'), stats: { size: bigIntToLong(400n) } } },
      { type: EntryType.ADD, manifest: { path: Buffer.from('file5'), stats: { size: bigIntToLong(500n) } } },
      { type: EntryType.REMOVE, manifest: { path: Buffer.from('file6') } },
    ];
    mockBackupClient.getFileList = jest.fn().mockReturnValue(from(it));

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    // WHEN
    const observable = backupClientProgress.getFileList(ctxt, {
      sharePath: Buffer.from('sharePath'),
      includes: [],
      excludes: [],
    });
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
  });

  it('#createBackup', async () => {
    // GIVEN
    const it = [
      ...createChunks(100n),
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file1'),
          stats: { size: bigIntToLong(100n), compressedSize: bigIntToLong(50n) },
        },
      },
      ...createChunks(200n),
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file2'),
          stats: { size: bigIntToLong(200n), compressedSize: bigIntToLong(100n) },
        },
      },
      ...createChunks(300n),
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file3'),
          stats: { size: bigIntToLong(300n), compressedSize: bigIntToLong(150n) },
        },
      },
      ...createChunks(400n),
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file4'),
          stats: { size: bigIntToLong(400n), compressedSize: bigIntToLong(200n) },
        },
      },
      ...createChunks(500n),
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file5'),
          stats: { size: bigIntToLong(500n), compressedSize: bigIntToLong(250n) },
        },
      },
      ...createChunks(600n),
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file6'),
          stats: { size: bigIntToLong(600n), compressedSize: bigIntToLong(600n) },
        },
      },
    ];
    mockBackupClient.createBackup = jest.fn().mockReturnValue(from(it));

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    // WHEN
    const observable = backupClientProgress.createBackup(ctxt, {
      sharePath: Buffer.from('sharePath'),
      includes: [],
      excludes: [],
    });
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    for (const r of result) {
      expect(r).toMatchSnapshot({ speed: expect.any(Number) }, 'result');
    }
  });
});
