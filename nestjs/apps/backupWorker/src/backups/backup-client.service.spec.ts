import { Logger } from '@nestjs/common';
import { ClientGrpcProxy } from '@nestjs/microservices';
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
  PoolChunkRefCnt,
  PoolService,
  ReferenceCount,
} from '@woodstock/shared';
import { constants as constantsFs } from 'fs';
import { fromNodeStream } from 'ix';
import { AsyncSink, from, pipe, toArray as toArrayIx } from 'ix/asynciterable';
import * as Long from 'long';
import { lastValueFrom, toArray } from 'rxjs';
import { Readable } from 'stream';
import { setTimeout } from 'timers/promises';
import { BackupClientGrpc, BackupsGrpcContext } from './backup-client-grpc.class';
import { BackupClient } from './backup-client.service';

describe('BackupClient', () => {
  const mockApplicationConfigService = {
    poolPath: 'poolPath',
  };

  const mockCientGrpc = {
    authenticate: () => 0,
    streamLog: () => 0,
    downloadFileList: () => 0,
    copyChunk: () => 0,
  };

  const mockBackupService = {
    getDestinationDirectory: (host: string, backup: string) => `${host}-${backup}`,
    getManifest: (host: string, backup: string, share: string) => new Manifest(share, `${host}-${backup}`),
    getHostDirectory: (host: string) => host,
  };

  const mockManifestService = {
    writeFileListEntry: () => 0,
    writeJournalEntry: () => 0,
    readFilelistEntries: () => 0,
    compact: () => 0,
  };

  const mockPoolService = {
    getChunk: () => 0,
  };

  const mockPoolChunkRefCnt = {
    writeJournal: () => 0,
    compact: () => 0,
  };

  const fakeClient = {
    close: () => void 0,
  } as ClientGrpcProxy;

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
        { provide: BackupsService, useValue: mockBackupService },
        { provide: ManifestService, useValue: mockManifestService },
        { provide: PoolService, useValue: mockPoolService },
        { provide: PoolChunkRefCnt, useValue: mockPoolChunkRefCnt },
        BackupClient,
      ],
    }).compile();

    backupClient = module.get<BackupClient>(BackupClient);
  });

  it('authenticate', async () => {
    // GIVEN
    const streamLog = new AsyncSink<LogEntry>();
    const logger = fakeLogger();
    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    mockCientGrpc.authenticate = jest.fn().mockResolvedValue({ sessionId: 'sessionId' });
    mockCientGrpc.streamLog = jest.fn().mockReturnValue(pipe(streamLog));
    fakeClient.close = jest.fn();

    // WHEN
    await backupClient.authenticate(ctxt, logger, 'password');

    streamLog.write({ context: 'context1', level: LogLevel.log, line: 'line1' });
    streamLog.write({ context: 'context2', level: LogLevel.error, line: 'line2' });
    streamLog.write({ context: 'context3', level: LogLevel.warn, line: 'line3' });

    await setTimeout(50);

    backupClient.close(ctxt);
    streamLog.end();

    // THEN
    expect(fakeClient.close).toHaveBeenCalled();
    expect(mockCientGrpc.authenticate).toHaveBeenCalledWith(
      new BackupsGrpcContext('host', 'ip', 1, fakeClient),
      'password',
    );
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

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

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
    const savedJournalChunk: unknown[] = [];

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

    mockPoolChunkRefCnt.writeJournal = jest.fn().mockImplementation(async (it, path, cb) => {
      for await (const v of it) {
        savedJournalChunk.push(await cb(v));
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

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

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
    expect(savedJournalChunk).toMatchSnapshot('savedJournalChunk');
    expect(wrappers).toMatchSnapshot('wrappers');
    expect(mockPoolChunkRefCnt.writeJournal).toHaveBeenCalledWith(
      expect.any(AsyncSink),
      'host-1',
      expect.any(Function),
    );
  });

  it('compact', async () => {
    //
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

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    // WHEN
    const observable = backupClient.compact(ctxt, Buffer.from('sharePath'));
    const result = await lastValueFrom(observable.pipe(toArray()));

    // THEN
    expect(result).toMatchSnapshot('result');
    expect(mockManifestService.compact).toHaveBeenCalledWith(new Manifest('sharePath', 'host-1'), expect.any(Function));
    expect(compactManifest).toMatchSnapshot('compactManifest');
  });

  it('countRef', async () => {
    mockPoolChunkRefCnt.compact = jest.fn().mockResolvedValue(undefined);

    const ctxt = new BackupsGrpcContext('host', 'ip', 1, fakeClient);

    // WHEN
    await backupClient.countRef(ctxt);

    // THEN
    expect(mockPoolChunkRefCnt.compact).toHaveBeenCalledWith(new ReferenceCount('host', 'host-1', 'poolPath'));
  });
});
