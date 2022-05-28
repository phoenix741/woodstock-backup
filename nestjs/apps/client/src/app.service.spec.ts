import { Test, TestingModule } from '@nestjs/testing';
import { AppService } from './app.service';
import { BackupService } from './backup/backup.service';
import { LogService } from './logger/log.service';

describe('AppService', () => {
  let service: AppService;

  const backupService = {};
  const logService = {};

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AppService,
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
