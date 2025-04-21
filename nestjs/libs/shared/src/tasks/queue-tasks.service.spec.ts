import { Test, TestingModule } from '@nestjs/testing';
import { QueueTasksService } from './queue-tasks.service';
import { JobBackupData } from '../backuping';

describe('QueueTasksService', () => {
  let service: QueueTasksService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [QueueTasksService],
    }).compile();

    service = module.get<QueueTasksService>(QueueTasksService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('serializeBackupTask/deserializeBackupTask', () => {
    it('should serialize/deserialize', () => {
      // GIVEN
      const task: JobBackupData = {
        host: 'test-host',
        config: undefined, // or provide a mock HostConfiguration if needed
        previousNumber: 42,
        number: 43,
        ip: '192.168.1.10',
        startDate: 1717233600000, // e.g., Date.now() or a fixed timestamp
        force: true,
      };

      // WHEN
      const serialized = service.serializeBackupTask(task);
      const deserialized = service.deserializeBackupTask(serialized);

      // THEN
      expect(serialized).toMatchSnapshot('serialized');
      expect(deserialized).toMatchSnapshot('deserialized');
    });
  });
});
