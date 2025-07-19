module.exports = [
  {
    script: 'apps/api/main.js',
    name: 'api',
    cwd: '/app/nestjs',
    exec_mode: 'cluster',
    instances: parseInt(process.env.API_INSTANCES ?? '1'),
    env: {
      MAX_BACKUP_TASK: 1,
    },
  },
  {
    script: 'apps/clientApi/main.js',
    name: 'clientApi',
    cwd: '/app/nestjs',
    exec_mode: 'cluster',
    instances: parseInt(process.env.API_INSTANCES ?? '1'),
  },
  {
    script: 'apps/scheduleWorker/main.js',
    name: 'scheduleWorker',
    cwd: '/app/nestjs',
    instances: process.env.DISABLE_SCHEDULER === 'true' ? 0 : 1,
  },
];
