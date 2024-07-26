import { Test, TestingModule } from '@nestjs/testing';
import { FilesService } from '@woodstock/shared';
import { BackupsFilesService } from './backups-files.service.js';
import { BackupsService } from '@woodstock/shared';

describe('Backups File Service', () => {
  let service: BackupsFilesService;

  const mockBackupsService = {
    getDestinationDirectory: (name: string, number: number) => `destinationDirectory/${name}/${number}`,
  };

  const mockFilesService = {};

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        { provide: BackupsService, useValue: mockBackupsService },
        { provide: FilesService, useValue: mockFilesService },
        BackupsFilesService,
      ],
    }).compile();

    service = module.get<BackupsFilesService>(BackupsFilesService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
