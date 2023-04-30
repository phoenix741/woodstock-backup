import { Logger } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import {
  ApplicationConfigService,
  BackupsService,
  bigIntToLong,
  EntryType,
  LogEntry,
  LogLevel,
  Manifest,
  ManifestService,
  PoolRefCount,
  PoolService,
  RefCntService,
} from '@woodstock/shared';
import { constants as constantsFs } from 'fs';
import { fromNodeStream } from 'ix';
import { AsyncSink, from, toArray as toArrayIx } from 'ix/asynciterable';
import * as Long from 'long';
import { lastValueFrom, toArray } from 'rxjs';
import { Readable } from 'stream';
import { setTimeout } from 'timers/promises';
import { BackupClientGrpc, BackupsGrpcContext } from './backup-client-grpc.class.js';
import { BackupClientLocal } from './backup-client-local.class.js';
import { BackupClient } from './backup-client.service.js';

describe('BackupClient', () => {
  const mockApplicationConfigService = {
    poolPath: 'poolPath',
  };

  const mockCientGrpc = {
    authenticate: () => 0,
    streamLog: () => 0,
    downloadFileList: () => 0,
    copyChunk: () => 0,
    close: () => 0,
  };

  const mockBackupService = {
    getDestinationDirectory: (host: string, backup: string) => `${host}-${backup}`,
    getManifest: (host: string, backup: string, share: string) => new Manifest(share, `${host}-${backup}`),
    getManifests: (host: string, backup: string) =>
      ['home', 'etc'].map((share) => new Manifest(share, `${host}-${backup}`)),
    getHostDirectory: (host: string) => host,
    addBackupSharePath: async (_host: string, _n: number, _sharePath: Buffer) => void 0,
  };

  const mockManifestService = {
    writeFileListEntry: () => 0,
    writeJournalEntry: () => 0,
    readFilelistEntries: () => 0,
    compact: () => 0,
    generateRefcntFromManifest: () => 0,
  };

  const mockPoolService = {
    getChunk: () => 0,
  };

  const mockPoolChunkRefCnt = {
    addChunkInformationToRefCnt: async (_s: AsyncIterable<PoolRefCount>, _f: string) => void 0,
    addReferenceCountToRefCnt: async (_s: AsyncIterable<PoolRefCount>, _f: string) => void 0,
    addBackupRefcntTo: async (_hostpath: string, _backupPath: string) => void 0,
    compactAllRefCnt: () => 0,
  };

  function fakeLogger() {
    const logger = new Logger('FakeLogger');
    logger.log = jest.fn();
    logger.debug = jest.fn();
    logger.error = jest.fn();
    logger.verbose = jest.fn();
    logger.warn = jest.fn();
    return logger;
  }

  let backupClient: BackupClient;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        { provide: ApplicationConfigService, useValue: mockApplicationConfigService },
        { provide: BackupClientGrpc, useValue: mockCientGrpc },
        { provide: BackupClientLocal, useValue: mockCientGrpc },
        { provide: BackupsService, useValue: mockBackupService },
        { provide: ManifestService, useValue: mockManifestService },
        { provide: PoolService, useValue: mockPoolService },
        { provide: RefCntService, useValue: mockPoolChunkRefCnt },
        BackupClient,
      ],
    }).compile();

    backupClient = module.get<BackupClient>(BackupClient);
  });

  it('authenticate', async () => {
    // GIVEN
    const streamLog = new AsyncSink<LogEntry>();
    const logger = fakeLogger();
    const ctxt = new BackupsGrpcContext('host', 'ip', 1);

    mockCientGrpc.authenticate = jest.fn().mockResolvedValue({ sessionId: 'sessionId' });
    mockCientGrpc.streamLog = jest.fn().mockReturnValue(from(streamLog));
    mockCientGrpc.close = jest.fn();

    // WHEN
    await backupClient.authenticate(ctxt, logger, logger, 'password');

    streamLog.write({ context: 'context1', level: LogLevel.log, line: 'line1' });
    streamLog.write({ context: 'context2', level: LogLevel.error, line: 'line2' });
    streamLog.write({ context: 'context3', level: LogLevel.warn, line: 'line3' });

    await setTimeout(50);

    backupClient.close(ctxt);
    streamLog.end();

    // THEN
    expect(mockCientGrpc.close).toHaveBeenCalled();
    expect(mockCientGrpc.authenticate).toHaveBeenCalledWith(new BackupsGrpcContext('host', 'ip', 1), 'password');
    expect(mockCientGrpc.streamLog).toMatchSnapshot('mockClientGrpc.streamLog');
    expect(logger.log).toMatchSnapshot('logger.log');
    expect(logger.warn).toMatchSnapshot('logger.warn');
    expect(logger.error).toMatchSnapshot('logger.error');
  });

  it('getFileList', async () => {
    // GIVEN
    const savedFilelist: unknown[] = [];

    const filelist = from([
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file1'),
          stats: { size: bigIntToLong(100n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file2'),
          stats: { size: bigIntToLong(200n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file3'),
          stats: { size: bigIntToLong(300n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file4'),
          stats: { size: bigIntToLong(400n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file5'),
          stats: { size: bigIntToLong(500n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file6'),
          stats: { size: bigIntToLong(600n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
    ]);

    mockCientGrpc.downloadFileList = jest.fn().mockReturnValue(filelist);

    mockManifestService.writeFileListEntry = jest.fn().mockImplementation(async (it, m, cb) => {
      expect(m).toEqual(new Manifest('sharePath', 'host-1'));
      for await (const v of it) {
        savedFilelist.push(await cb(v));
      }
    });

    const ctxt = new BackupsGrpcContext('host', 'ip', 1);

    // WHEN
    const observable = backupClient.getFileList(ctxt, {
      sharePath: Buffer.from('sharePath'),
      includes: [],
      excludes: [],
    });
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
    expect(savedFilelist).toMatchSnapshot('savedFilelist');
    expect(mockCientGrpc.downloadFileList).toMatchSnapshot('downloadFileList');
  });

  it('createBackup', async () => {
    // GIVEN
    const savedJournal: unknown[] = [];
    const refcntHost: unknown[] = [];

    const filelist = from([
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file7'),
          stats: { size: bigIntToLong(100n), mode: Long.fromNumber(constantsFs.S_IFREG) },
          chunks: [Buffer.from('sha256_1')],
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file8'),
          stats: { size: bigIntToLong(200n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file9'),
          stats: { size: bigIntToLong(300n), mode: Long.fromNumber(constantsFs.S_IFREG) },
          chunks: [Buffer.from('sha256_2')],
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file10'),
          stats: { size: bigIntToLong(400n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file11'),
          stats: { size: bigIntToLong(500n), mode: Long.fromNumber(constantsFs.S_IFREG) },
          chunks: [Buffer.from('sha256_3')],
        },
      },
      {
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('file12'),
          stats: { size: bigIntToLong(600n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        },
      },
    ]);
    mockManifestService.readFilelistEntries = jest.fn().mockReturnValue(filelist);

    mockManifestService.writeJournalEntry = jest.fn().mockImplementation(async (it, m, cb) => {
      expect(m).toEqual(new Manifest('sharePath', 'host-1'));
      for await (const v of it) {
        savedJournal.push(await cb(v));
      }
    });

    mockPoolChunkRefCnt.addChunkInformationToRefCnt = jest.fn().mockImplementation(async (it, m) => {
      expect(m).toEqual('host-1/REFCNT.backup');
      for await (const v of it) {
        refcntHost.push(v);
      }
    });

    const wrappers: unknown[] = [];
    let i = 0;
    mockPoolService.getChunk = jest.fn().mockImplementation((sha256) => {
      const wrapper = {
        exists: () => !!sha256,
        getChunkInformation: async () => ({ sha256, size: BigInt(i * 100), compressedSize: BigInt(i * 50) }),
        write: async (readable: Readable) => {
          wrappers.push(await toArrayIx(fromNodeStream(readable)));
          return {
            sha256: sha256 || Buffer.from('newSha256_' + i),
            size: BigInt(i * 100),
            compressedSize: BigInt(i * 50),
          };
        },
      };
      i++;
      return wrapper;
    });

    mockCientGrpc.copyChunk = jest.fn().mockReturnValue(Readable.from(['chunk1', 'chunk2']));

    const ctxt = new BackupsGrpcContext('host', 'ip', 1);

    // WHEN
    const observable = backupClient.createBackup(ctxt, {
      sharePath: Buffer.from('sharePath'),
      includes: [],
      excludes: [],
    });
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
    expect(savedJournal).toMatchSnapshot('savedJournal');
    expect(wrappers).toMatchSnapshot('wrappers');
    expect(mockPoolChunkRefCnt.addChunkInformationToRefCnt).toHaveBeenCalledTimes(1);
    expect(refcntHost).toMatchSnapshot('refcntHost');
  });

  it('compact', async () => {
    // GIVEN
    const fileManifests = [
      {
        path: Buffer.from('file13'),
        stats: { size: bigIntToLong(100n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        chunks: [Buffer.from('sha256_1')],
      },
      { path: Buffer.from('file14'), stats: { size: bigIntToLong(200n), mode: Long.fromNumber(constantsFs.S_IFREG) } },
      {
        path: Buffer.from('file15'),
        stats: { size: bigIntToLong(300n), mode: Long.fromNumber(constantsFs.S_IFREG) },
        chunks: [Buffer.from('sha256_1')],
      },
      { path: Buffer.from('file16'), stats: { size: bigIntToLong(400n), mode: Long.fromNumber(constantsFs.S_IFREG) } },
      { path: Buffer.from('file17'), stats: { size: bigIntToLong(500n), mode: Long.fromNumber(constantsFs.S_IFREG) } },
      { path: Buffer.from('file18'), stats: { size: bigIntToLong(600n), mode: Long.fromNumber(constantsFs.S_IFREG) } },
    ];

    const compactManifest: unknown[] = [];
    mockManifestService.compact = jest.fn().mockImplementation(async (m, cb) => {
      for (const fileManifest of fileManifests) {
        compactManifest.push(await cb(fileManifest));
      }
    });

    mockBackupService.addBackupSharePath = jest.fn();

    const ctxt = new BackupsGrpcContext('host', 'ip', 1);

    // WHEN
    const observable = backupClient.compact(ctxt, Buffer.from('sharePath'));
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
    expect(mockManifestService.compact).toHaveBeenCalledWith(new Manifest('sharePath', 'host-1'), expect.any(Function));
    expect(compactManifest).toMatchSnapshot('compactManifest');
    expect(mockBackupService.addBackupSharePath).toHaveBeenCalledWith('host', 1, Buffer.from('sharePath'));
  });

  it('countRef', async () => {
    // GIVEN
    const sha256 = [
      { sha256: Buffer.from('sha256_1'), refCount: 1 },
      { sha256: Buffer.from('sha256_2'), refCount: 2 },
      { sha256: Buffer.from('sha256_3'), refCount: 3 },
      { sha256: Buffer.from('sha256_4'), refCount: 4 },
      { sha256: Buffer.from('sha256_5'), refCount: 5 },
      { sha256: Buffer.from('sha256_6'), refCount: 6 },
    ];

    mockPoolChunkRefCnt.addReferenceCountToRefCnt = jest.fn().mockResolvedValue(undefined);
    mockBackupService.getManifests = jest
      .fn()
      .mockImplementation((host, backup) => ['home', 'etc'].map((share) => new Manifest(share, `${host}-${backup}`)));
    mockManifestService.generateRefcntFromManifest = jest.fn().mockImplementation((m) => {
      return sha256.map((sha256) => ({
        ...sha256,
        sha256: Buffer.concat([sha256.sha256, Buffer.from(m.manifestPath)]),
      }));
    });
    mockPoolChunkRefCnt.compactAllRefCnt = jest.fn();
    mockPoolChunkRefCnt.addBackupRefcntTo = jest.fn();

    const ctxt = new BackupsGrpcContext('host', 'ip', 1);

    // WHEN
    await backupClient.countRef(ctxt);

    // THEN
    expect(mockPoolChunkRefCnt.addReferenceCountToRefCnt).toMatchSnapshot(
      'mockPoolChunkRefCnt.addReferenceCountToRefCnt',
    );
    expect(mockPoolChunkRefCnt.addBackupRefcntTo).toMatchSnapshot('mockPoolChunkRefCnt.addBackupRefcntTo');
  });
});
