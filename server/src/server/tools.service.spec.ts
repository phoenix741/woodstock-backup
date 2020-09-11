import { Test, TestingModule } from '@nestjs/testing';

import { ToolsService } from './tools.service';
import { ApplicationConfigService } from '../config/application-config.service';
import { YamlService } from '../utils/yaml.service';

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

  it(`should get command for command btrfsCreateSnapshot`, async () => {
    expect(
      await service.getCommand('btrfsCreateSnapshot', {
        hostname: 'pc-test',
        srcBackupNumber: 17,
        destBackupNumber: 20,
        qGroupId: 2,
      }),
    ).toBe('/bin/btrfs subvolume snapshot -i 1/2 hostPath/pc-test/17 hostPath/pc-test/20');
  });

  it(`should get all path`, async () => {
    expect(await service.getPaths({ hostname: 'pc-test', srcBackupNumber: 14, destBackupNumber: 33 })).toEqual({
      destBackupPath: 'hostPath/pc-test/33',
      hostnamePath: 'hostPath/pc-test',
      qgroupHostPath: 'hostPath/pc-test/qgroup',
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
