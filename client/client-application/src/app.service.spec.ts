import { Test, TestingModule } from '@nestjs/testing';
import { RefreshCacheRequest, SharedModule } from '@woodstock/shared';
import { Subject } from 'rxjs';
import * as Long from 'long';

import { AppService } from './app.service';

describe('AppService', () => {
  let service: AppService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [SharedModule],
      providers: [AppService],
    }).compile();

    service = module.get<AppService>(AppService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('#refreshCache', () => {
    it('should create a manifest file from scratch', (done) => {
      const request = new Subject<RefreshCacheRequest>();
      service.refreshCache(request).subscribe({
        next: (val) => {
          expect(val).toMatchSnapshot();
        },
        complete: () => done(),
        error: (err) => done(err),
      });

      request.next({ header: { sharePath: Buffer.from('/home') } });
      request.next({ manifest: { path: Buffer.from('/test1'), stats: { size: Long.fromNumber(100) } } });
      request.complete();
    });
  });
});
