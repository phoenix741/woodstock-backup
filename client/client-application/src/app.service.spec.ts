import { Test, TestingModule } from '@nestjs/testing';
import {
  Manifest,
  ManifestService,
  RefreshCacheRequest,
  LaunchBackupRequest,
  SharedModule,
  IndexManifest,
  EntryType,
  StatusCode,
  FileReader,
} from '@woodstock/shared';
import * as Long from 'long';
import { Observable, pipe, Subject, throwError } from 'rxjs';
import { tap, switchMap } from 'rxjs/operators';
import { AppService } from './app.service';
import * as path from 'path';

describe('AppService', () => {
  let service: AppService;
  let manifestService: ManifestService;
  let fileReader: FileReader;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [SharedModule],
      providers: [AppService],
    }).compile();

    manifestService = module.get<ManifestService>(ManifestService);
    fileReader = module.get<FileReader>(FileReader);
    service = module.get<AppService>(AppService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('#refreshCache', () => {
    it('should create a manifest file from scratch', (done) => {
      let count = 0;
      jest.spyOn(manifestService, 'deleteManifest').mockResolvedValue(undefined);
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation((manifest: () => Manifest) => {
        return pipe(
          tap((journalEntry) => {
            expect(manifest().manifestPath).toBe('/tmp/backups.L2hvbWU=.manifest');
            expect(journalEntry).toMatchSnapshot('Define journal entry');
            count++;
          }),
        );
      });

      const request = new Subject<RefreshCacheRequest>();
      service.refreshCache(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Define refresh cache');
          expect(count).toBe(6);
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({ header: { sharePath: Buffer.from('/home') } });
      request.next({ manifest: { path: Buffer.from('/test1'), stats: { size: Long.fromNumber(100) } } });
      request.next({ manifest: { path: Buffer.from('/test2'), stats: { size: Long.fromNumber(200) } } });
      request.next({ manifest: { path: Buffer.from('/test3'), stats: { size: Long.fromNumber(300) } } });
      request.next({ manifest: { path: Buffer.from('/test4'), stats: { size: Long.fromNumber(400) } } });
      request.next({ manifest: { path: Buffer.from('/test5'), stats: { size: Long.fromNumber(500) } } });
      request.next({ manifest: { path: Buffer.from('/test6'), stats: { size: Long.fromNumber(600) } } });
      request.complete();
    });

    it('should have an error when no header', (done) => {
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation((manifest: () => Manifest) => {
        return pipe(
          tap((journalEntry) => {
            expect(manifest()).toBe('/tmp/backups.L2hvbWUvcGhvZW5peC9Eb3dubG9hZHMvdGVzdA==.manifest');
            expect(journalEntry).toMatchSnapshot('Define journal entry');
          }),
        );
      });

      const request = new Subject<RefreshCacheRequest>();
      service.refreshCache(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Define refresh cache');
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({ manifest: { path: Buffer.from('/test1'), stats: { size: Long.fromNumber(100) } } });
      request.complete();
    });
  });

  describe('#launchBackup', () => {
    beforeEach(() => {
      const mockIndex = new IndexManifest();
      jest.spyOn(manifestService, 'exists').mockResolvedValue(false);
      jest.spyOn(manifestService, 'loadIndex').mockImplementation((manifest: Manifest) => {
        return new Observable((subscribe) => {
          expect(manifest.manifestPath).toBe('/tmp/backups.L2hvbWUvcGhvZW5peC9Eb3dubG9hZHMvdGVzdA==.manifest');
          subscribe.next(mockIndex);
          subscribe.complete();
        });
      });
      jest.spyOn(fileReader, 'getFiles').mockImplementation((index: IndexManifest) => {
        return new Observable((subscribe) => {
          expect(index).toBe(mockIndex);
          subscribe.next({
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
          });
          subscribe.complete();
        });
      });
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation((manifest: () => Manifest) => {
        return pipe(
          tap((journalEntry) => {
            expect(manifest().manifestPath).toBe('/tmp/backups.L2hvbWUvcGhvZW5peC9Eb3dubG9hZHMvdGVzdA==.manifest');
            expect(journalEntry).toMatchSnapshot('Define journal entry');
          }),
        );
      });
      jest.spyOn(manifestService, 'compact').mockImplementation((manifest: Manifest) => {
        expect(manifest.manifestPath).toBe('/tmp/backups.L2hvbWUvcGhvZW5peC9Eb3dubG9hZHMvdGVzdA==.manifest');
        return new Observable((subscribe) => {
          subscribe.next({
            path: Buffer.from('/42093_X_Hot Rod.pdf'),
          });
          subscribe.complete();
        });
      });
    });

    it('should make a backup', (done) => {
      const request = new Subject<LaunchBackupRequest>();
      service.launchBackup(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Launch backup (normal)');
          if (val.response?.diskReadFinished) {
            request.next({ footer: { code: StatusCode.Ok } });
            request.complete();
          }
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({
        header: { sharePath: Buffer.from('/home/phoenix/Downloads/test'), lastBackupNumber: -1, newBackupNumber: 0 },
      });
    });

    it('should make a backup with update', (done) => {
      const request = new Subject<LaunchBackupRequest>();
      service.launchBackup(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Launch backup (with update)');
          if (val.response?.diskReadFinished) {
            request.next({
              entry: {
                type: EntryType.MODIFY,
                manifest: {
                  path: Buffer.from('/42093_X_Hot Rod.pdf'),
                },
              },
            });
            request.next({ footer: { code: StatusCode.Ok } });
            request.complete();
          }
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({
        header: { sharePath: Buffer.from('/home/phoenix/Downloads/test'), lastBackupNumber: -1, newBackupNumber: 0 },
      });
    });

    it('should make a backup with error (from server)', (done) => {
      const request = new Subject<LaunchBackupRequest>();
      service.launchBackup(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Launch backup (with error from server)');
          if (val.response?.diskReadFinished) {
            request.next({ footer: { code: StatusCode.Failed } });
            request.complete();
          }
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({
        header: { sharePath: Buffer.from('/home/phoenix/Downloads/test'), lastBackupNumber: -1, newBackupNumber: 0 },
      });
    });

    it('should make a backup with error (from client)', (done) => {
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation((manifest: () => Manifest) => {
        return pipe(switchMap(() => throwError('Error from test')));
      });

      const request = new Subject<LaunchBackupRequest>();
      service.launchBackup(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Launch backup (with error)');
          if (val.response?.diskReadFinished) {
            request.next({ footer: { code: StatusCode.Ok } });
            request.complete();
          }
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({
        header: { sharePath: Buffer.from('/home/phoenix/Downloads/test'), lastBackupNumber: -1, newBackupNumber: 0 },
      });
    });
  });
});
