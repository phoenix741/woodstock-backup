import { Test, TestingModule } from '@nestjs/testing';
import { ApplicationConfigService } from '../../../core/src/config';
import { YamlService } from '../input-output';
import { ToolsService } from './tools.service.js';

jest.mock('child_process');

describe('Tools Service', () => {
  let service: ToolsService;

  const mockApplicationConfigService = {
    configPathOfTools: 'config',
    backupPath: 'backupPath',
    configPath: 'configPath',
    statisticsPath: 'statisticsPath',
    configPathOfHosts: 'configPathOfHosts',
    configPathOfScheduler: 'configPathOfScheduler',
    hostPath: 'hostPath',
    logPath: 'logPath',
    redis: { host: 'hostRedis', port: 6300 },
    logLevel: 'debug',
    toJSON() {
      return mockApplicationConfigService;
    },
  };
  const mockYamlService = {
    loadFile: jest.fn().mockImplementation((path, def) => def),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        { provide: ApplicationConfigService, useValue: mockApplicationConfigService },
        { provide: YamlService, useValue: mockYamlService },
        ToolsService,
      ],
    }).compile();

    service = module.get<ToolsService>(ToolsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it(`should get tool for command df`, async () => {
    expect(await service.getTool('df')).toBe('/bin/df');
  });

  it(`should get command for command statsSpaceUsage`, async () => {
    expect(
      await service.getCommand('statsSpaceUsage', {
        hostname: 'pc-test',
      }),
    ).toBe('/bin/df -k --print-type hostPath');
  });

  it(`should get all path`, async () => {
    expect(await service.getPaths({ hostname: 'pc-test', srcBackupNumber: 14, destBackupNumber: 33 })).toEqual({
      destBackupPath: 'hostPath/pc-test/33',
      hostnamePath: 'hostPath/pc-test',
      srcBackupPath: 'hostPath/pc-test/14',
      backupPath: 'backupPath',
      configPath: 'configPath',
      configPathOfHosts: 'configPathOfHosts',
      configPathOfScheduler: 'configPathOfScheduler',
      configPathOfTools: 'config',
      hostPath: 'hostPath',
      logLevel: 'debug',
      logPath: 'logPath',
      redis: {
        host: 'hostRedis',
        port: 6300,
      },
      statisticsPath: 'statisticsPath',
      toJSON: expect.any(Function),
    });
  });

  it(`should one path`, async () => {
    expect(await service.getPath('srcBackupPath', { hostname: 'pc-test', srcBackupNumber: 14 })).toBe(
      'hostPath/pc-test/14',
    );
  });
});
