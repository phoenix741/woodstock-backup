import { Test, TestingModule } from '@nestjs/testing';
import { from, lastValueFrom, toArray } from 'rxjs';
import { BackupClientProgress } from './backup-client-progress.service.js';
import { BackupsClientService } from './backups-client.service.js';
import { JsEntryType, WoodstockBackupClient } from '@woodstock/shared-rs';

describe('BackupClientProgress', () => {
  const mockBackupClient = {
    createClient: () => 0,
    authenticate: () => 0,
    createBackupDirectory: () => 0,
    executeCommand: () => 0,
    uploadFileList: () => 0,
    downloadFileList: () => 0,
    createBackup: () => 0,
    compact: () => 0,
    countReferences: () => 0,
    saveBackup: () => 0,
    close: () => 0,
  };

  const fakeClient = {} as WoodstockBackupClient;

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
      providers: [{ provide: BackupsClientService, useValue: mockBackupClient }, BackupClientProgress],
    }).compile();

    backupClientProgress = module.get<BackupClientProgress>(BackupClientProgress);
  });

  it('#authenticate', async () => {
    // GIVEN
    mockBackupClient.authenticate = jest.fn().mockResolvedValue(undefined);

    // WHEN
    const result = await backupClientProgress.authenticate(fakeClient, 'password');

    // THEN
    expect(result).toMatchSnapshot('result');
  });

  it('#executeCommand', async () => {
    // GIVEN
    mockBackupClient.executeCommand = jest.fn().mockResolvedValue(undefined);

    // WHEN
    const observable = backupClientProgress.executeCommand(fakeClient, 'command');
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
  });

  it('#getFileList', async () => {
    // GIVEN
    const it = [
      { type: JsEntryType.Add, manifest: { path: Buffer.from('file1'), stats: { size: 100n } } },
      { type: JsEntryType.Add, manifest: { path: Buffer.from('file2'), stats: { size: 200n } } },
      { type: JsEntryType.Add, manifest: { path: Buffer.from('file3'), stats: { size: 300n } } },
      { type: JsEntryType.Add, manifest: { path: Buffer.from('file4'), stats: { size: 400n } } },
      { type: JsEntryType.Add, manifest: { path: Buffer.from('file5'), stats: { size: 500n } } },
      { type: JsEntryType.Remove, manifest: { path: Buffer.from('file6') } },
    ];
    mockBackupClient.downloadFileList = jest.fn().mockReturnValue(from(it));

    // WHEN
    const observable = backupClientProgress.downloadFileList(fakeClient, {
      sharePath: 'sharePath',
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
        type: JsEntryType.Add,
        manifest: {
          path: Buffer.from('file1'),
          stats: { size: 100n, compressedSize: 50n },
        },
      },
      ...createChunks(200n),
      {
        type: JsEntryType.Add,
        manifest: {
          path: Buffer.from('file2'),
          stats: { size: 200n, compressedSize: 100n },
        },
      },
      ...createChunks(300n),
      {
        type: JsEntryType.Add,
        manifest: {
          path: Buffer.from('file3'),
          stats: { size: 300n, compressedSize: 150n },
        },
      },
      ...createChunks(400n),
      {
        type: JsEntryType.Add,
        manifest: {
          path: Buffer.from('file4'),
          stats: { size: 400n, compressedSize: 200n },
        },
      },
      ...createChunks(500n),
      {
        type: JsEntryType.Add,
        manifest: {
          path: Buffer.from('file5'),
          stats: { size: 500n, compressedSize: 250n },
        },
      },
      ...createChunks(600n),
      {
        type: JsEntryType.Add,
        manifest: {
          path: Buffer.from('file6'),
          stats: { size: 600n, compressedSize: 600n },
        },
      },
    ];
    mockBackupClient.createBackup = jest.fn().mockReturnValue(from(it));

    // WHEN
    const observable = backupClientProgress.createBackup(fakeClient, 'sharePath');
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    for (const r of result) {
      expect(r).toMatchSnapshot({ speed: expect.any(Number) }, 'result');
    }
  });
});
