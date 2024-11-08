import { Test, TestingModule } from '@nestjs/testing';
import { FilesService } from './files.service';

import { CoreBackupsService, CoreFilesService } from '@woodstock/shared-rs';

describe('FilesService', () => {
  let service: FilesService;

  const mockBackupsService = {
    getBackupSharePaths: jest.fn(),
  };

  const mockFilesService = {
    createViewer: jest.fn(),
    readFile: jest.fn(),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        FilesService,
        { provide: CoreBackupsService, useValue: mockBackupsService },
        { provide: CoreFilesService, useValue: mockFilesService },
      ],
    }).compile();

    service = module.get<FilesService>(FilesService);
  });

  describe('listShares', () => {
    it('list share from a directory', async () => {
      // GIVEN
      mockBackupsService.getBackupSharePaths = jest.fn(() => ['file1', 'file3']);

      // WHEN
      const res = await service.listShares('hostname', 1);

      // THEN
      expect(res).toMatchSnapshot('res');
      expect(mockBackupsService.getBackupSharePaths).toHaveBeenCalledWith('hostname', 1);
    });
  });
});
