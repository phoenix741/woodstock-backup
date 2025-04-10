import { Logger } from '@nestjs/common';
import { BackupContext, JsLogLevel, initLog } from '@woodstock/shared-rs';

export function initializeLog(context: BackupContext) {
  const logger = new Logger('SharedLogged');
  initLog(context, (msg) => {
    if (msg.progress) {
      switch (msg.progress.level) {
        case JsLogLevel.Debug:
          logger.debug(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case JsLogLevel.Trace:
          logger.verbose(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case JsLogLevel.Info:
          logger.log(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case JsLogLevel.Warn:
          logger.warn(
            {
              message: msg.progress.message,
              hostname: msg.progress.hostname,
              backupNumber: msg.progress.backupNumber,
            },
            msg.progress.context,
          );
          break;
        case JsLogLevel.Error:
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
