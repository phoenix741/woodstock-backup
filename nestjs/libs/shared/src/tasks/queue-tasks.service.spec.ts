import { Logger } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import { instanceToPlain } from 'class-transformer';
import { lastValueFrom } from 'rxjs';
import { QueueGroupTasks, QueueSubTask, QueueTaskContext, QueueTaskPriority, QueueTasks } from './queue-tasks.model';
import { QueueTasksService } from './queue-tasks.service';

type LocalContext = Record<string, string>;

const allCommands = [
  'connection',
  'init-directory',
  'authentication',
  'pre-command1',
  'pre-command2',
  'pre-command3',
  'share1-filelist',
  'share1-chunks',
  'share1-compact',
  'share2-filelist',
  'share2-chunks',
  'share2-compact',
  'post-command1',
  'post-command2',
  'post-command3',
  'close-connection',
  'refcnt-host',
  'refcnt-pool',
];

describe('QueueTasksService', () => {
  let service: QueueTasksService;
  let task: QueueGroupTasks;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [QueueTasksService],
    }).compile();

    service = module.get<QueueTasksService>(QueueTasksService);
  });

  beforeEach(() => {
    task = new QueueTasks('GLOBAL', { command: 'GLOBAL' })
      .add(
        new QueueGroupTasks('INITIALISATION', { command: 'INITIALISATION' })
          .add(new QueueSubTask('connection', { command: 'connection' }, QueueTaskPriority.INITIALISATION))
          .add(new QueueSubTask('init-directory', { command: 'init-directory' }, QueueTaskPriority.INITIALISATION))
          .add(new QueueSubTask('authentication', { command: 'authentication' }, QueueTaskPriority.INITIALISATION)),
      )
      .add(
        new QueueGroupTasks('PRE_COMMANDS', { command: 'PRE_COMMANDS' })
          .add(new QueueSubTask('pre-command1', { command: 'pre-command1' }, QueueTaskPriority.PRE_PROCESSING))
          .add(new QueueSubTask('pre-command2', { command: 'pre-command2' }, QueueTaskPriority.PRE_PROCESSING))
          .add(new QueueSubTask('pre-command3', { command: 'pre-command3' }, QueueTaskPriority.PRE_PROCESSING)),
      )
      .add(
        new QueueGroupTasks('SHARES', { command: 'SHARES' })
          .add(
            new QueueGroupTasks('SHARE1', { command: 'SHARE1' })
              .add(
                new QueueSubTask('share1-filelist', { command: 'share1-filelist' }, QueueTaskPriority.PRE_PROCESSING),
              )
              .add(new QueueSubTask('share1-chunks', { command: 'share1-chunks' }, QueueTaskPriority.PROCESSING))
              .add(new QueueSubTask('share1-compact', { command: 'share1-compact' }, QueueTaskPriority.FINALISATION)),
          )
          .add(
            new QueueGroupTasks('SHARE2', { command: 'SHARE2' })
              .add(
                new QueueSubTask('share2-filelist', { command: 'share2-filelist' }, QueueTaskPriority.PRE_PROCESSING),
              )
              .add(new QueueSubTask('share2-chunks', { command: 'share2-chunks' }, QueueTaskPriority.PROCESSING))
              .add(new QueueSubTask('share2-compact', { command: 'share2-compact' }, QueueTaskPriority.FINALISATION)),
          ),
      )
      .add(
        new QueueGroupTasks('POST_COMMANDS', { command: 'POST_COMMANDS' })
          .add(new QueueSubTask('post-command1', { command: 'post-command1' }, QueueTaskPriority.PROCESSING))
          .add(new QueueSubTask('post-command2', { command: 'post-command2' }, QueueTaskPriority.PROCESSING))
          .add(new QueueSubTask('post-command3', { command: 'post-command3' }, QueueTaskPriority.PROCESSING)),
      )
      .add(
        new QueueGroupTasks('FINALISATION', { command: 'FINALISATION' })
          .add(new QueueSubTask('close-connection', { command: 'close-connection' }, QueueTaskPriority.POST_PROCESSING))
          .add(new QueueSubTask('refcnt-host', { command: 'refcnt-host' }, QueueTaskPriority.FINALISATION))
          .add(new QueueSubTask('refcnt-pool', { command: 'refcnt-pool' }, QueueTaskPriority.FINALISATION)),
      );
  });

  function fakeLogger() {
    const logger = new Logger('fakeLogger');
    logger.log = jest.fn();
    logger.debug = jest.fn();
    logger.error = jest.fn();
    logger.verbose = jest.fn();
    logger.warn = jest.fn();
    return logger;
  }

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('executeTasks', () => {
    it('success backup', async () => {
      // GIVEN
      const processedTaskOrder: string[] = [];

      const context = new QueueTaskContext({});
      for (const command of allCommands) {
        context.commands.set(command, async () => {
          processedTaskOrder.push(command);
        });
      }

      // WHEN
      await lastValueFrom(service.executeTasks(task, context));

      // THEN
      expect(instanceToPlain(task)).toMatchSnapshot('task');
      expect(processedTaskOrder).toMatchSnapshot('processedTaskOrder');
    });

    it('failed at pre init step', async () => {
      // GIVEN
      const processedTaskOrder: string[] = [];

      const context = new QueueTaskContext({});
      for (const command of allCommands) {
        context.commands.set(command, async (_gc, lc: LocalContext) => {
          processedTaskOrder.push(command);
          if (lc['command'] === 'authentication') {
            throw new Error('Fake error');
          }
        });
      }

      // WHEN
      await lastValueFrom(service.executeTasks(task, context));

      // THEN
      expect(instanceToPlain(task)).toMatchSnapshot('task');
      expect(processedTaskOrder).toMatchSnapshot('processedTaskOrder');
    });

    it('failed at pre progress', async () => {
      // GIVEN
      const processedTaskOrder: string[] = [];

      const context = new QueueTaskContext({});
      for (const command of allCommands) {
        context.commands.set(command, async (_gc, lc: LocalContext) => {
          processedTaskOrder.push(command);
          if (lc['command'] === 'share1-chunks') {
            throw new Error('Fake error');
          }
        });
      }

      // WHEN
      await lastValueFrom(service.executeTasks(task, context));

      // THEN
      expect(instanceToPlain(task)).toMatchSnapshot('task');
      expect(processedTaskOrder).toMatchSnapshot('processedTaskOrder');
    });

    it('failed at finalisation', async () => {
      // GIVEN
      const processedTaskOrder: string[] = [];

      const context = new QueueTaskContext({});
      for (const command of allCommands) {
        context.commands.set(command, async (_gc, lc: LocalContext) => {
          processedTaskOrder.push(command);
          if (lc['command'] === 'refcnt-host') {
            throw new Error('Fake error');
          }
        });
      }

      // WHEN
      await lastValueFrom(service.executeTasks(task, context));

      // THEN
      expect(instanceToPlain(task)).toMatchSnapshot('task');
      expect(processedTaskOrder).toMatchSnapshot('processedTaskOrder');
    });
  });

  describe('serializeBackupTask/deserializeBackupTask', () => {
    it('should serialize/deserialize', () => {
      // GIVEN

      // WHEN
      const serialized = service.serializeBackupTask(task);
      const deserialized = service.deserializeBackupTask(serialized);

      // THEN
      expect(serialized).toMatchSnapshot('serialized');
      expect(deserialized).toMatchSnapshot('deserialized');
      expect(deserialized).toEqual(task);
    });
  });
});
