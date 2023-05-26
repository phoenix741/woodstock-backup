import { Test, TestingModule } from '@nestjs/testing';
import { BackupOnClientService } from '@woodstock/shared';
import { AppService } from './app.service.js';
import { AuthService } from './auth/auth.service.js';
import { LogService } from './log.service.js';

describe('AppService', () => {
  let service: AppService;

  const backupService = {};
  const logService = {};

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AppService,
        { provide: AuthService, useValue: {} },
        { provide: BackupOnClientService, useValue: backupService },
        { provide: LogService, useValue: logService },
      ],
    }).compile();

    service = module.get<AppService>(AppService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
