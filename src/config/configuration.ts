import { join } from 'path';

export default () => {
  const backupPath = process.env.BACKUP_PATH || '/var/lib/woodstock';
  const configPath = process.env.CONFIG_PATH || join(backupPath, 'config');
  const hostPath = process.env.HOST_PATH || join(backupPath, 'hosts');
  const logPath = process.env.HOST_PATH || join(backupPath, 'log');

  return {
    paths: {
      backupPath,
      configPath,
      configHostPath: join(configPath, 'hosts.yml'),
      schedulerPath: join(configPath, 'scheduler.yml'),
      hostPath,
      logPath,
    },
    redis: {
      port: parseInt(process.env.REDIS_PORT || ''),
      host: process.env.REDIS_HOST,
    },
  };
};
