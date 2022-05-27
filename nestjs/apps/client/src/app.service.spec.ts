import { Test, TestingModule } from '@nestjs/testing';
import { EncryptionService } from '@woodstock/shared';
import { AppService } from './app.service';
import { BackupService } from './backup/backup.service';
import { ClientConfigService } from './config/client.config';
import { LogService } from './logger/log.service';

describe('AppService', () => {
  let service: AppService;

  const backupService = {};
  const logService = {};

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AppService,
        { provide: ClientConfigService, useValue: {} },
        { provide: EncryptionService, useValue: {} },
        { provide: BackupService, useValue: backupService },
        { provide: LogService, useValue: logService },
      ],
    }).compile();

    service = module.get<AppService>(AppService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
