import { Test, TestingModule } from '@nestjs/testing';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { LogService } from './log.service';
import { StatusCode } from '../../../packages/shared/src/models/query.model';

describe('AppController', () => {
  let appController: AppController;

  const fakeAppService = {};
  const fakeLogService = {};

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
      providers: [
        { provide: AppService, useValue: fakeAppService },
        { provide: LogService, useValue: fakeLogService },
      ],
    }).compile();

    appController = app.get<AppController>(AppController);
  });

  describe('executeCommand', () => {
    it('Should execute the command !', () => {
      expect(appController.executeCommand({ command: 'Cmd' })).toEqual({
        code: StatusCode.Failed,
        stdout: '',
        stderr: 'Cmd',
      });
    });
  });
});
