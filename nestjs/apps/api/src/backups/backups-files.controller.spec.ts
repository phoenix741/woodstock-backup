import { Test, TestingModule } from '@nestjs/testing';
import * as archiver from 'archiver';
import * as httpMocks from 'node-mocks-http';
import { BackupsFilesController } from './backups-files.controller';
import { BackupsFilesService } from './backups-files.service';

jest.mock('archiver');

const mockedArchiver = archiver as jest.Mocked<typeof archiver>;

describe('Backups File Controller', () => {
  let controller: BackupsFilesController;

  const backupsFilesService = {
    listShare: (name: string, number: number) => [
      {
        name,
        dev: number,
      },
    ],
    list: (name: string, number: number, share: string, path: string) => [
      { name, dev: number },
      { name: share },
      { name: path },
    ],
    createArchive(archive: archiver.Archiver, name: string, number: number, share: string, path: string) {
      archive.append('test', { name: 'test' });
    },
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [{ provide: BackupsFilesService, useValue: backupsFilesService }],
      controllers: [BackupsFilesController],
    }).compile();

    controller = module.get<BackupsFilesController>(BackupsFilesController);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  it('should list available share', async () => {
    expect(await controller.share('name', 10)).toEqual([{ name: 'name', dev: 10 }]);
  });

  it('should list the available share of a path', async () => {
    expect(await controller.list('name', 10, 'share', 'path')).toEqual([
      { name: 'name', dev: 10 },
      { name: 'share' },
      { name: 'path' },
    ]);
  });

  it('should download zip file', async () => {
    // GIVEN
    const res = httpMocks.createResponse();
    const mockedArchiverInstance = {
      pipe: jest.fn(),
      append: jest.fn(),
      finalize: jest.fn(),
    } as unknown as archiver.Archiver;

    mockedArchiver.create.mockReturnValue(mockedArchiverInstance);

    // WHEN
    await controller.download('name', 10, 'share', 'path', res, 'application/zip');

    // THEN
    expect(mockedArchiver.create).toHaveBeenCalledWith('zip');
    expect(res).toMatchSnapshot('res');
    expect(mockedArchiverInstance.pipe).toHaveBeenCalledWith(res);
    expect(mockedArchiverInstance.append).toMatchSnapshot('append');
    expect(mockedArchiverInstance.finalize).toHaveBeenCalledWith();
  });

  it('should download zip file', async () => {
    // GIVEN
    const res = httpMocks.createResponse();
    const mockedArchiverInstance = {
      pipe: jest.fn(),
      append: jest.fn(),
      finalize: jest.fn(),
    } as unknown as archiver.Archiver;

    mockedArchiver.create.mockReturnValue(mockedArchiverInstance);

    // WHEN
    await controller.download('name', 10, 'share', 'path', res, 'application/x-tar');

    // THEN
    expect(mockedArchiver.create).toHaveBeenCalledWith('tar');
    expect(res).toMatchSnapshot('res');
    expect(mockedArchiverInstance.pipe).toHaveBeenCalledWith(res);
    expect(mockedArchiverInstance.append).toMatchSnapshot('append');
    expect(mockedArchiverInstance.finalize).toHaveBeenCalledWith();
  });
});
