import { Metadata } from '@grpc/grpc-js';
import { Test, TestingModule } from '@nestjs/testing';
import { StatusCode } from '@woodstock/shared';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { LogService } from './logger/log.service';

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
      const metadata: Metadata = new Metadata();
      metadata.add('X-Session-Id', 'test');
      expect(appController.executeCommand({ command: 'Cmd' }, metadata)).toEqual({
        code: StatusCode.Ok,
        stdout: 'Cmd',
        stderr: 'test',
      });
    });
  });
});
