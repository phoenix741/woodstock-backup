import { Test, TestingModule } from '@nestjs/testing';
import { Manifest, ManifestService, RefreshCacheRequest, LaunchBackupRequest, SharedModule } from '@woodstock/shared';
import * as Long from 'long';
import { pipe, Subject } from 'rxjs';
import { tap } from 'rxjs/operators';

import { AppService } from './app.service';
import { StatusCode } from '../../../packages/shared/src/models/query.model';
import { EntryType } from '../../../packages/shared/src/models/manifest.model';

describe('AppService', () => {
  let service: AppService;
  let manifestService: ManifestService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [SharedModule],
      providers: [AppService],
    }).compile();

    manifestService = module.get<ManifestService>(ManifestService);
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
      jest.spyOn(manifestService, 'deleteManifest').mockResolvedValue(undefined);
      jest.spyOn(manifestService, 'writeJournalEntry').mockImplementation((manifest: () => Manifest) => {
        return pipe(
          tap((journalEntry) => {
            expect(manifest()).toBe('/home');
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
    it('should make a backup', (done) => {
      const request = new Subject<LaunchBackupRequest>();
      service.launchBackup(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot('Define refresh cache');
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({
        header: { sharePath: Buffer.from('/home/phoenix/Downloads/test'), lastBackupNumber: -1, newBackupNumber: 0 },
      });
      request.next({ footer: { code: StatusCode.Ok } });
      request.complete();
    }, 30000);
  });
});
