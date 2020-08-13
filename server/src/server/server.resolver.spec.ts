import { Test, TestingModule } from '@nestjs/testing';

import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { ServerResolver } from './server.resolver';
import { ServerService } from './server.service';

jest.mock('child_process');

describe('Server Resolver', () => {
  let resolver: ServerResolver;

  const serverServiceMock = {
    check: jest.fn(),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [{ provide: ServerService, useValue: serverServiceMock }],
      controllers: [ServerResolver],
    }).compile();

    resolver = module.get<ServerResolver>(ServerResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });

  it('should retrieve the btrfs status', async () => {
    // GIVEN
    serverServiceMock.check.mockResolvedValueOnce({ isBtrfsVolume: true });

    // THEN
    expect(await resolver.status()).toEqual({ isBtrfsVolume: true });
  });
});
