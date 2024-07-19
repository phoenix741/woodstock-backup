module.exports = [
  {
    script: 'apps/api/main.js',
    name: 'api',
    cwd: '/app/nestjs',
    exec_mode: 'cluster',
    instances: parseInt(process.env.API_INSTANCES ?? '1'),
  },
  {
    script: 'apps/backupWorker/main.js',
    name: 'backupWorker',
    cwd: '/app/nestjs',
    instances: parseInt(process.env.BACKUP_WORKER_INSTANCES || '1'),
    env: {
      MAX_BACKUP_TASK: 1,
    },
  },
  {
    script: 'apps/refcntWorker/main.js',
    name: 'refcntWorker',
    cwd: '/app/nestjs',
    instances: process.env.DISABLE_REFCNT === 'true' ? 0 : 1,
  },
  {
    script: 'apps/scheduleWorker/main.js',
    name: 'scheduleWorker',
    cwd: '/app/nestjs',
    instances: process.env.DISABLE_SCHEDULER === 'true' ? 0 : 1,
  },
  {
    script: 'apps/statsWorker/main.js',
    name: 'statsWorker',
    cwd: '/app/nestjs',
    instances: process.env.DISABLE_STATS === 'true' ? 0 : 1,
  },
];
