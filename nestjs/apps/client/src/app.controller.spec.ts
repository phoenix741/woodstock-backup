import { Test, TestingModule } from '@nestjs/testing';
import { StatusCode } from '@woodstock/shared';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { AuthService } from './auth/auth.service.js';
import { LogService } from './log.service.js';

describe('AppController', () => {
  let appController: AppController;

  const fakeAppService = {
    async executeCommand(sessionId: string, command: string) {
      return {
        code: StatusCode.Ok,
        stdout: command,
        stderr: sessionId,
      };
    },
  };
  const fakeLogService = {};

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
      providers: [
        { provide: AppService, useValue: fakeAppService },
        { provide: AuthService, useValue: fakeAppService },
        { provide: LogService, useValue: fakeLogService },
      ],
    }).compile();

    appController = app.get<AppController>(AppController);
  });

  describe('executeCommand', () => {
    it('Should execute the command !', async () => {
      // WHEN
      const result = await appController.executeCommand({ command: 'Cmd' });

      // THEN
      expect(result).toEqual({
        code: StatusCode.Ok,
        stderr: 'Cmd',
        stdout: undefined,
      });
    });
  });
});
