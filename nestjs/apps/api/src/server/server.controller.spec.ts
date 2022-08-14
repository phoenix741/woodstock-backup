import { Test, TestingModule } from '@nestjs/testing';
import { ApplicationConfigService } from '@woodstock/shared';
import * as childProcess from 'child_process';
import { EventEmitter } from 'events';
import { Response } from 'express';
import { join } from 'path';
import { ServerController } from './server.controller.js';
import { ServerService } from './server.service.js';

jest.mock('child_process');

const mockedChildProcess = childProcess as jest.Mocked<typeof childProcess>;

describe('Server Controller', () => {
  let controller: ServerController;

  const applicationConfigMock = {
    logPath: join('__fixtures__'),
  };

  const serverServiceMock = {
    check: jest.fn(),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        { provide: ApplicationConfigService, useValue: applicationConfigMock },
        { provide: ServerService, useValue: serverServiceMock },
      ],
      controllers: [ServerController],
    }).compile();

    controller = module.get<ServerController>(ServerController);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  it('/server/status should retrieve the btrfs status', async () => {
    // GIVEN
    serverServiceMock.check.mockResolvedValueOnce({ isBtrfsVolume: true });

    // THEN
    expect(await controller.getStatus()).toEqual({ isBtrfsVolume: true });
  });

  it('/server/log/application.log should return log of the server', async () => {
    // GIVEN
    const res: Response = {
      sendFile: jest.fn(),
      header: jest.fn(),
    } as unknown as Response;

    // WHEN
    await controller.getApplicationLog(false, res);

    // THEN
    expect(res.header).toHaveBeenCalledWith('Content-Type', 'text/plain;charset=utf-8');
    expect(res.sendFile).toHaveBeenCalledWith('__fixtures__/application.log');
  });

  it('/server/log/application.log?tailable=true should return log of the server', async () => {
    // GIVEN
    const res: Response = {
      header: jest.fn(),
      write: jest.fn(),
      end: jest.fn(),
    } as unknown as Response;

    const spawnEe = new EventEmitter();
    const stdout = new EventEmitter();
    const stderr = new EventEmitter();

    mockedChildProcess.spawn.mockReturnValue(
      Object.assign(spawnEe, {
        stdout,
        stderr,
      }) as childProcess.ChildProcessWithoutNullStreams,
    );

    // WHEN
    controller.getApplicationLog(true, res);

    // THEN
    stdout.emit('data', 'write data');
    stderr.emit('data', 'write error data');
    spawnEe.emit('exit', 10);

    expect(res.header).toHaveBeenCalledWith('Content-Type', 'text/plain;charset=utf-8');
    expect(res.write).toHaveBeenCalledWith('write data', 'utf-8');
    expect(res.write).toHaveBeenCalledWith('write error data', 'utf-8');
    expect(res.end).toHaveBeenCalledWith(10);
  });

  it('/server/log/exceptions.log should return log of the server', async () => {
    // GIVEN
    const res: Response = {
      sendFile: jest.fn(),
      header: jest.fn(),
    } as unknown as Response;

    // WHEN
    await controller.getExceptionsLog(false, res);

    // THEN
    expect(res.header).toHaveBeenCalledWith('Content-Type', 'text/plain;charset=utf-8');
    expect(res.sendFile).toHaveBeenCalledWith('__fixtures__/exceptions.log');
  });

  it('/server/log/exceptions.log?tailable=true should return log of the server', async () => {
    // GIVEN
    const res: Response = {
      header: jest.fn(),
      write: jest.fn(),
      end: jest.fn(),
    } as unknown as Response;

    const spawnEe = new EventEmitter();
    const stdout = new EventEmitter();
    const stderr = new EventEmitter();

    mockedChildProcess.spawn.mockReturnValue(
      Object.assign(spawnEe, {
        stdout,
        stderr,
      }) as childProcess.ChildProcessWithoutNullStreams,
    );

    // WHEN
    controller.getExceptionsLog(true, res);

    // THEN
    stdout.emit('data', 'write data');
    stderr.emit('data', 'write error data');
    spawnEe.emit('exit', 10);

    expect(res.header).toHaveBeenCalledWith('Content-Type', 'text/plain;charset=utf-8');
    expect(res.write).toHaveBeenCalledWith('write data', 'utf-8');
    expect(res.write).toHaveBeenCalledWith('write error data', 'utf-8');
    expect(res.end).toHaveBeenCalledWith(10);
  });
});
