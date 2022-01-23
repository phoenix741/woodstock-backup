import { Test, TestingModule } from '@nestjs/testing';
import { SharedModule } from '@woodstock/shared';
import { AppService } from './app.service';
import { BackupService } from './backup/backup.service';
import { LogService } from './logger/log.service';

describe('AppService', () => {
  let service: AppService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [SharedModule],
      providers: [AppService, BackupService, LogService],
    }).compile();

    service = module.get<AppService>(AppService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
