import { Logger } from '@nestjs/common';
import { BackupContext, LogLevel, initLog } from '@woodstock/shared-rs';

export function initializeLog(context: BackupContext) {
  const logger = new Logger('SharedLogged');
  initLog(context, (msg) => {
    if (msg.progress) {
      switch (msg.progress.level) {
        case LogLevel.Debug:
          logger.debug(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case LogLevel.Trace:
          logger.verbose(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case LogLevel.Info:
          logger.log(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case LogLevel.Warn:
          logger.warn(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case LogLevel.Error:
          logger.error(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
      }
    }
  });
}
