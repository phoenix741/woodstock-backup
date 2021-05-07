import { Test, TestingModule } from '@nestjs/testing';
import { StatusCode } from '@woodstock/shared';

import { AppController } from './app.controller';
import { AppService } from './app.service';
import { LogService } from './log.service';

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
        code: StatusCode.Ok,
        stdout: 'Cmd',
        stderr: '',
      });
    });
  });
});
