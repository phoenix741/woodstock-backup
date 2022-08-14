import { Test, TestingModule } from '@nestjs/testing';
import { ApplicationConfigService, BackupsService, RefCntService } from '@woodstock/shared';
import { RemoveService } from './remove.service.js';

describe('RemoveService', () => {
  let service: RemoveService;

  const fakeApplicationConfigService = {
    poolPath: '/tmp/pool',
  };

  const fakeBackupsService = {
    getHostDirectory: (hostname: string) => `/tmp/hosts/${hostname}`,
    getDestinationDirectory: (hostname: string, backupNumber: number) => `/tmp/hosts/${hostname}/${backupNumber}`,
    removeBackup: (_hostname: string, _backupNumber: number) => Promise.resolve(),
  };

  const fakeRefCntService = {
    removeBackupRefcntTo: (_hostPath: string, _backupPath: string) => Promise.resolve(),
  };

  describe('#createStateMachine', () => {
    beforeEach(async () => {
      const module: TestingModule = await Test.createTestingModule({
        providers: [
          RemoveService,
          { provide: ApplicationConfigService, useValue: fakeApplicationConfigService },
          { provide: BackupsService, useValue: fakeBackupsService },
          { provide: RefCntService, useValue: fakeRefCntService },
        ],
      }).compile();

      service = module.get<RemoveService>(RemoveService);
    });

    it('should remove the backup (but not the pool)', async () => {
      // GIVEN
      fakeBackupsService.removeBackup = jest.fn();
      fakeRefCntService.removeBackupRefcntTo = jest.fn();

      // WHEN
      await service.remove('hostname', 1);

      // THEN
      expect(fakeRefCntService.removeBackupRefcntTo).toHaveBeenCalledWith(
        '/tmp/hosts/hostname/REFCNT.host',
        '/tmp/hosts/hostname/1/REFCNT.backup',
      );
      expect(fakeBackupsService.removeBackup).toHaveBeenCalledWith('hostname', 1);
    });
  });
});
