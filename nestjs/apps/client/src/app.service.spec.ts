import { Test, TestingModule } from '@nestjs/testing';
import { EncryptionService } from '@woodstock/shared';
import { AppService } from './app.service.js';
import { BackupService } from './backup/backup.service.js';
import { ClientConfigService } from './config/client.config.js';
import { LogService } from './logger/log.service.js';

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
